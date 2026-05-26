use proc_macro2::{
    Group, Ident as Ident2, Punct, Spacing, Span, TokenStream as TokenStream2, TokenTree,
};
use quote::{format_ident, quote};
use syn::LitStr;

pub enum LiveSegment {
    Literal(String),
    Reactive { path: Vec<String>, spec: String },
}

pub fn parse_live_template(s: &str) -> Result<Vec<LiveSegment>, String> {
    let mut segments = Vec::new();
    let mut literal = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '{' if chars.get(i + 1) == Some(&'{') => {
                literal.push('{');
                i += 2;
            }
            '}' if chars.get(i + 1) == Some(&'}') => {
                literal.push('}');
                i += 2;
            }
            '{' => {
                if !literal.is_empty() {
                    segments.push(LiveSegment::Literal(literal.clone()));
                    literal.clear();
                }
                i += 1;
                let mut inner = String::new();
                while i < chars.len() && chars[i] != '}' {
                    inner.push(chars[i]);
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("unclosed { in live! template".into());
                }
                i += 1; // consume '}'

                let (var_part, spec_part) = match inner.find(':') {
                    Some(pos) => (
                        inner[..pos].trim().to_string(),
                        inner[pos + 1..].to_string(),
                    ),
                    None => (inner.trim().to_string(), String::new()),
                };

                if var_part.is_empty() {
                    return Err("empty variable name in live! template".into());
                }

                let path: Vec<String> = var_part.split('.').map(str::to_string).collect();
                segments.push(LiveSegment::Reactive {
                    path,
                    spec: spec_part,
                });
            }
            c => {
                literal.push(c);
                i += 1;
            }
        }
    }

    if !literal.is_empty() {
        segments.push(LiveSegment::Literal(literal));
    }

    Ok(segments)
}

/// Walk a token stream, find every `my.field` pattern, replace each with `(*field)`,
/// and return the list of unique field names found alongside the rewritten stream.
pub fn find_my_fields(tokens: TokenStream2) -> (Vec<String>, TokenStream2) {
    let mut fields: Vec<String> = Vec::new();
    let mut output: Vec<TokenTree> = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(tt) = iter.next() {
        let is_my = matches!(&tt, TokenTree::Ident(id) if id.to_string() == "my");

        if is_my {
            let is_dot = matches!(iter.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '.');
            if is_dot {
                let dot = iter.next().unwrap();
                if let Some(TokenTree::Ident(field_id)) = iter.peek().cloned() {
                    let field_name = field_id.to_string();
                    let field_span = field_id.span();
                    iter.next(); // consume the field ident
                    if !fields.contains(&field_name) {
                        fields.push(field_name.clone());
                    }
                    // Emit (*field)
                    let star: TokenTree = Punct::new('*', Spacing::Alone).into();
                    let ident: TokenTree = Ident2::new(&field_name, field_span).into();
                    let inner: TokenStream2 = vec![star, ident].into_iter().collect();
                    output.push(TokenTree::Group(Group::new(
                        proc_macro2::Delimiter::Parenthesis,
                        inner,
                    )));
                    continue;
                } else {
                    // `my.` but not followed by an ident — emit as-is
                    output.push(tt);
                    output.push(dot);
                    continue;
                }
            }
        }

        match tt {
            TokenTree::Group(g) => {
                let (sub_fields, sub_ts) = find_my_fields(g.stream());
                for f in sub_fields {
                    if !fields.contains(&f) {
                        fields.push(f);
                    }
                }
                output.push(TokenTree::Group(Group::new(g.delimiter(), sub_ts)));
            }
            other => output.push(other),
        }
    }

    (fields, output.into_iter().collect())
}

pub fn expand_live(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    // Suppress unused attr warning — live! is a proc_macro (no attr), but for
    // symmetry with expand_compute we accept it.
    let _ = attr;

    // Template form: live!("text with {my.field:.spec}")
    let input_clone: proc_macro2::TokenStream = item.clone();
    if let Ok(lit) = syn::parse2::<LitStr>(input_clone) {
        return live_template_ts(lit);
    }

    // Expression form: live!(if my.field { "a" } else { "b" })
    let (fields, rewritten) = find_my_fields(item);

    if fields.is_empty() {
        return syn::Error::new(
            Span::call_site(),
            "live! expression must reference at least one model field via my.field",
        )
        .to_compile_error();
    }
    if fields.len() > 1 {
        return syn::Error::new(
            Span::call_site(),
            "live! does not support multiple fields; extract a derived Signal with .map() first",
        )
        .to_compile_error();
    }

    let field = format_ident!("{}", fields[0]);
    quote! {
        my.#field.map(move |#field| { (#rewritten).to_string() })
    }
}

fn live_template_ts(lit: LitStr) -> TokenStream2 {
    let template_str = lit.value();

    let segments = match parse_live_template(&template_str) {
        Ok(s) => s,
        Err(msg) => {
            return syn::Error::new(lit.span(), msg).to_compile_error();
        }
    };

    let mut setup: Vec<TokenStream2> = Vec::new();
    let mut fmt_template = String::new();
    let mut fmt_args: Vec<TokenStream2> = Vec::new();
    let mut n = 0usize;

    for segment in &segments {
        match segment {
            LiveSegment::Literal(text) => {
                fmt_template.push_str(&text.replace('{', "{{").replace('}', "}}"));
            }
            LiveSegment::Reactive { path, spec } => {
                let rc_var = format_ident!("_live_rc_{}", n);
                let class_var = format_ident!("_live_class_{}", n);
                let init_var = format_ident!("_live_init_{}", n);
                let html_var = format_ident!("_live_html_{}", n);

                let fmt_str = if spec.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{:{}}}", spec)
                };

                let rc_expr = match path.len() {
                    2 => {
                        let var = format_ident!("{}", path[0]);
                        let field = format_ident!("{}", path[1]);
                        quote! { #var.#field.rc() }
                    }
                    _ => {
                        let ident = format_ident!("{}", path[0]);
                        quote! { #ident.rc() }
                    }
                };

                setup.push(quote! {
                    let #rc_var = #rc_expr;
                    let #class_var = crate::view_components::fresh_span_class();
                    let #init_var = format!(#fmt_str, #rc_var.borrow().read());
                    observe_fmt!(#rc_var, &#class_var, |v| format!(#fmt_str, v));
                    let #html_var = format!(
                        r#"<span class="{}">{}</span>"#,
                        #class_var, #init_var
                    );
                });

                fmt_template.push_str("{}");
                fmt_args.push(quote! { #html_var });
                n += 1;
            }
        }
    }

    quote! {
        &{
            #( #setup )*
            format!(#fmt_template, #( #fmt_args ),*)
        }
    }
}

pub fn expand_compute(_attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let (fields, rewritten) = find_my_fields(item);

    if fields.is_empty() {
        return syn::Error::new(
            Span::call_site(),
            "compute! requires at least one my.field reference",
        )
        .to_compile_error();
    }

    let primary = format_ident!("{}", fields[0]);
    let secondaries: Vec<_> = fields[1..].iter().map(|f| format_ident!("{}", f)).collect();
    let clone_idents: Vec<_> = fields[1..]
        .iter()
        .map(|f| format_ident!("__{}_clone", f))
        .collect();
    let read_idents: Vec<_> = fields[1..]
        .iter()
        .map(|f| format_ident!("__{}_read", f))
        .collect();

    quote! {
        {
            #( let #clone_idents = my.#secondaries.clone(); )*
            my.#primary.map(move |#primary| {
                #( let #read_idents  = #clone_idents.read(); )*
                #( let #secondaries  = &#read_idents; )*
                #rewritten
            })
        }
    }
}
