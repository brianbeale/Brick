use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::LitStr;

use crate::live::find_my_fields;

pub static CSS_UNITS: &[&str] = &[
    "rem", "em", "px", "vh", "vw", "vmin", "vmax", "ch", "ex", "cm", "mm", "pt", "pc", "fr", "s",
    "ms", "deg", "rad", "turn", "dvh", "svh", "lvh", "dvw", "svw", "lvw",
];

pub struct CssDynValue {
    pub var_name: String,
    pub expr_tokens: TokenStream2,
    pub format_str: String, // e.g. "{:.1}%" or "{}"
}

/// Split `expr:spec` at the first top-level `:` that is not part of `::`.
/// Tracks `{`, `(`, `[` depth so colons inside nested expressions are skipped.
pub fn css_split_at_format_colon(s: &str) -> (String, String) {
    let chars: Vec<char> = s.chars().collect();
    let mut depth = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            ':' if depth == 0 => {
                if i + 1 < chars.len() && chars[i + 1] == ':' {
                    i += 2; // skip ::
                    continue;
                }
                let expr = chars[..i].iter().collect::<String>().trim().to_string();
                let spec = chars[i + 1..].iter().collect::<String>().trim().to_string();
                return (expr, spec);
            }
            _ => {}
        }
        i += 1;
    }
    (s.trim().to_string(), String::new())
}

/// Split `{expr:spec}` inner tokens into (expr_stream, spec_string).
/// The `:` separator is the first top-level colon that is not part of `::`.
pub fn split_expr_spec(tokens: Vec<proc_macro::TokenTree>) -> (proc_macro::TokenStream, String) {
    use proc_macro::TokenTree as PTT;
    let mut depth = 0usize;
    let mut colon_idx: Option<usize> = None;
    for (i, tt) in tokens.iter().enumerate() {
        match tt {
            PTT::Group(_) => depth += 1,
            PTT::Punct(p) if depth == 0 && p.as_char() == ':' => {
                // skip ::
                if matches!(tokens.get(i + 1), Some(PTT::Punct(q)) if q.as_char() == ':') {
                    continue;
                }
                colon_idx = Some(i);
                break;
            }
            _ => {}
        }
    }
    if let Some(idx) = colon_idx {
        let expr: proc_macro::TokenStream = tokens[..idx].iter().cloned().collect();
        let spec: String = tokens[idx + 1..]
            .iter()
            .map(|t| t.to_string())
            .collect::<String>()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        (expr, spec)
    } else {
        let expr: proc_macro::TokenStream = tokens.iter().cloned().collect();
        (expr, String::new())
    }
}

pub fn css_space_needed(s: &str) -> bool {
    !matches!(
        s.chars().last(),
        None | Some(' ') | Some('(') | Some('.') | Some('-') | Some('#')
    )
}

/// CSS token → string, detecting `{expr:spec}` groups in value positions.
/// `depth` = 0 at top level, 1 inside a rule block, etc.
/// `in_value` = true after `:` (property separator) and before `;` at `depth > 0`.
pub fn process_css_with_dyn(
    tokens: Vec<proc_macro::TokenTree>,
    dyn_values: &mut Vec<CssDynValue>,
    depth: usize,
    in_value_in: bool,
) -> String {
    use proc_macro::{Delimiter, TokenTree as PTT};
    let mut out = String::new();
    let mut in_value = in_value_in;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            PTT::Group(g) => {
                let delim = g.delimiter();
                let inner: Vec<PTT> = g.stream().into_iter().collect();
                match delim {
                    Delimiter::None => {
                        out.push_str(&process_css_with_dyn(inner, dyn_values, depth, in_value));
                        i += 1;
                    }
                    Delimiter::Brace if depth > 0 && in_value => {
                        // Reactive value: {expr:spec}[unit]
                        let (expr_ts, spec) = split_expr_spec(inner);

                        // Consume an optional unit suffix (%, px, rem, …)
                        let mut suffix = String::new();
                        i += 1;
                        while i < tokens.len() {
                            match &tokens[i] {
                                PTT::Punct(p) if p.as_char() == '%' => {
                                    suffix.push('%');
                                    i += 1;
                                }
                                PTT::Ident(id) => {
                                    let s = id.to_string();
                                    if CSS_UNITS.iter().any(|&u| u == s.as_str()) {
                                        suffix.push_str(&s);
                                        i += 1;
                                    } else {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }

                        let fmt_str = if spec.is_empty() {
                            format!("{{}}{}", suffix)
                        } else {
                            format!("{{:{}}}{}", spec, suffix)
                        };

                        let index = dyn_values.len();
                        let var_name = format!("--brick-dyn-{}", index);
                        if css_space_needed(&out) {
                            out.push(' ');
                        }
                        out.push_str(&format!("var({})", var_name));

                        let expr_ts2: TokenStream2 = expr_ts.into();
                        dyn_values.push(CssDynValue {
                            var_name,
                            expr_tokens: expr_ts2,
                            format_str: fmt_str,
                        });
                        continue; // i already advanced past group + suffix
                    }
                    Delimiter::Brace => {
                        // CSS rule body (or nested rule)
                        if css_space_needed(&out) {
                            out.push(' ');
                        }
                        out.push_str("{ ");
                        out.push_str(&process_css_with_dyn(inner, dyn_values, depth + 1, false));
                        out.push('}');
                        in_value = false;
                        i += 1;
                    }
                    Delimiter::Parenthesis => {
                        out.push('(');
                        out.push_str(&process_css_with_dyn(inner, dyn_values, depth, in_value));
                        out.push(')');
                        i += 1;
                    }
                    Delimiter::Bracket => {
                        if css_space_needed(&out) {
                            out.push(' ');
                        }
                        out.push('[');
                        out.push_str(&process_css_with_dyn(inner, dyn_values, depth, in_value));
                        out.push(']');
                        i += 1;
                    }
                }
            }
            PTT::Ident(id) => {
                if css_space_needed(&out) {
                    out.push(' ');
                }
                out.push_str(&id.to_string());
                i += 1;
            }
            PTT::Punct(p) => match p.as_char() {
                ':' => {
                    out.push(':');
                    out.push(' ');
                    if depth > 0 {
                        in_value = true;
                    }
                    i += 1;
                }
                ';' => {
                    out.push_str("; ");
                    in_value = false;
                    i += 1;
                }
                '-' => {
                    let prev_alnum = out.chars().last().map_or(false, |c| c.is_alphanumeric());
                    let next_ident = matches!(tokens.get(i + 1), Some(PTT::Ident(_)));
                    if prev_alnum && next_ident {
                        out.push('-');
                    } else {
                        if css_space_needed(&out) {
                            out.push(' ');
                        }
                        out.push('-');
                    }
                    i += 1;
                }
                ',' => {
                    out.push_str(", ");
                    i += 1;
                }
                '.' => {
                    out.push('.');
                    i += 1;
                }
                '!' => {
                    out.push('!');
                    i += 1;
                }
                '#' => {
                    if css_space_needed(&out) {
                        out.push(' ');
                    }
                    out.push('#');
                    i += 1;
                    while i < tokens.len() {
                        match &tokens[i] {
                            PTT::Literal(l) => {
                                out.push_str(&l.to_string());
                                i += 1;
                            }
                            PTT::Ident(id) => {
                                out.push_str(&id.to_string());
                                i += 1;
                                break;
                            }
                            _ => break,
                        }
                    }
                }
                other => {
                    if css_space_needed(&out) {
                        out.push(' ');
                    }
                    out.push(other);
                    i += 1;
                }
            },
            PTT::Literal(l) => {
                let s = l.to_string();
                let is_numeric = s.starts_with(|c: char| c.is_ascii_digit());
                if css_space_needed(&out) && !out.ends_with('-') {
                    out.push(' ');
                }
                if (s.starts_with('"') && s.ends_with('"'))
                    || (s.starts_with('\'') && s.ends_with('\''))
                {
                    out.push_str(&s[1..s.len() - 1]);
                } else {
                    out.push_str(&s);
                }
                if is_numeric {
                    if let Some(PTT::Ident(unit)) = tokens.get(i + 1) {
                        let u = unit.to_string();
                        if CSS_UNITS.iter().any(|&x| x == u.as_str()) {
                            out.push_str(&u);
                            i += 2;
                            continue;
                        }
                    }
                }
                i += 1;
            }
        }
    }
    out.trim().to_string()
}

pub fn expand_style(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens: Vec<proc_macro::TokenTree> = input.into_iter().collect();
    let mut dyn_values: Vec<CssDynValue> = Vec::new();
    let css = process_css_with_dyn(tokens, &mut dyn_values, 0, false);

    let mut stmts = quote! {
        let _scope = crate::view_components::get_scope();
        crate::view_components::inject_scoped_style(#css, &_scope);
    };

    for dyn_val in &dyn_values {
        let var_name = &dyn_val.var_name;
        let expr = &dyn_val.expr_tokens;
        let fmt_str = &dyn_val.format_str;
        stmts.extend(quote! {
            {
                let _scope_css = _scope.clone();
                let _sig = (#expr).clone();
                crate::view_components::push_css_on_mount(::std::boxed::Box::new({
                    let _scope_init = _scope_css.clone();
                    let _sig_init = _sig.clone();
                    move || {
                        crate::view_components::set_css_var_on_scope(
                            &_scope_init, #var_name,
                            &format!(#fmt_str, _sig_init.read()),
                        );
                        let _scope_obs = _scope_init.clone();
                        _sig.add_observer(
                            &format!("brick-css-{}-{}", _scope_init, #var_name),
                            ::std::boxed::Box::new(crate::state_mgmt::observers::Effect::new(
                                move |v| {
                                    crate::view_components::set_css_var_on_scope(
                                        &_scope_obs, #var_name,
                                        &format!(#fmt_str, v),
                                    );
                                }
                            )),
                        );
                    }
                }));
            }
        });
    }

    proc_macro::TokenStream::from(quote! {
        #[cfg(not(test))]
        { #stmts }
    })
}

pub fn expand_css(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lit = match syn::parse::<LitStr>(input) {
        Ok(l) => l,
        Err(_) => {
            return proc_macro::TokenStream::from(
                syn::Error::new(
                    Span::call_site(),
                    "css! expects a string literal, e.g. css!(\"width:{my.field:.1}%\")",
                )
                .to_compile_error(),
            );
        }
    };

    let s = lit.value();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;

    let mut fmt_template = String::new();
    // (rewritten_expr_tokens, format_spec_string, unit_suffix)
    let mut groups: Vec<(TokenStream2, String, String)> = Vec::new();
    let mut all_fields: Vec<String> = Vec::new();

    while i < chars.len() {
        if chars[i] == '{' {
            // Find matching '}' tracking brace depth.
            let mut depth = 1usize;
            let mut j = i + 1;
            while j < chars.len() && depth > 0 {
                if chars[j] == '{' {
                    depth += 1;
                } else if chars[j] == '}' {
                    depth -= 1;
                }
                j += 1;
            }
            let content: String = chars[i + 1..j - 1].iter().collect();

            // Split content at first top-level ':' (not '::') → (expr, spec).
            let (expr_str, spec_str) = css_split_at_format_colon(&content);

            // Consume trailing CSS unit suffix after '}'.
            let mut unit = String::new();
            while j < chars.len() && (chars[j].is_alphabetic() || chars[j] == '%') {
                unit.push(chars[j]);
                j += 1;
            }

            // Parse expr as TokenStream2 and apply find_my_fields.
            let expr_ts: TokenStream2 = match expr_str.parse() {
                Ok(ts) => ts,
                Err(e) => {
                    return proc_macro::TokenStream::from(
                        syn::Error::new(lit.span(), format!("css! expression parse error: {}", e))
                            .to_compile_error(),
                    );
                }
            };
            let (group_fields, rewritten) = find_my_fields(expr_ts);
            for f in &group_fields {
                if !all_fields.contains(f) {
                    all_fields.push(f.clone());
                }
            }

            fmt_template.push_str(&format!("{{:{}}}{}", spec_str, unit));
            groups.push((rewritten, spec_str, unit));
            i = j;
        } else {
            if chars[i] == '}' {
                fmt_template.push_str("}}");
            } else {
                fmt_template.push(chars[i]);
            }
            i += 1;
        }
    }

    if all_fields.is_empty() {
        return proc_macro::TokenStream::from(
            syn::Error::new(
                lit.span(),
                "css! must contain at least one {expr} group; use a plain string for static styles",
            )
            .to_compile_error(),
        );
    }

    let primary = format_ident!("{}", all_fields[0]);
    let secondaries: Vec<_> = all_fields[1..]
        .iter()
        .map(|f| format_ident!("{}", f))
        .collect();
    let clone_idents: Vec<_> = all_fields[1..]
        .iter()
        .map(|f| format_ident!("__{}_clone", f))
        .collect();
    let read_idents: Vec<_> = all_fields[1..]
        .iter()
        .map(|f| format_ident!("__{}_read", f))
        .collect();

    let fmt_args: Vec<TokenStream2> = groups.iter().map(|(ts, _, _)| ts.clone()).collect();

    proc_macro::TokenStream::from(quote! {
        {
            #( let #clone_idents = my.#secondaries.clone(); )*
            my.#primary.map(move |#primary| {
                #( let #read_idents  = #clone_idents.read(); )*
                #( let #secondaries  = &#read_idents; )*
                format!(#fmt_template, #( #fmt_args ),*)
            })
        }
    })
}

pub fn css_tokens_to_string(tokens: Vec<proc_macro::TokenTree>) -> String {
    use proc_macro::{Delimiter, TokenTree};

    let mut out = String::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Group(g) => {
                match g.delimiter() {
                    Delimiter::None => {
                        out.push_str(&css_tokens_to_string(g.stream().into_iter().collect()));
                    }
                    Delimiter::Brace => {
                        if css_space_needed(&out) {
                            out.push(' ');
                        }
                        out.push('{');
                        out.push(' ');
                        out.push_str(&css_tokens_to_string(g.stream().into_iter().collect()));
                        out.push('}');
                    }
                    Delimiter::Parenthesis => {
                        out.push('(');
                        out.push_str(&css_tokens_to_string(g.stream().into_iter().collect()));
                        out.push(')');
                    }
                    Delimiter::Bracket => {
                        if css_space_needed(&out) {
                            out.push(' ');
                        }
                        out.push('[');
                        out.push_str(&css_tokens_to_string(g.stream().into_iter().collect()));
                        out.push(']');
                    }
                }
                i += 1;
            }

            TokenTree::Ident(id) => {
                if css_space_needed(&out) {
                    out.push(' ');
                }
                out.push_str(&id.to_string());
                i += 1;
            }

            TokenTree::Punct(p) => match p.as_char() {
                '-' => {
                    let prev_alnum = matches!(
                        out.chars().last(),
                        Some(c) if c.is_alphanumeric()
                    );
                    let next_ident = matches!(tokens.get(i + 1), Some(TokenTree::Ident(_)));
                    if prev_alnum && next_ident {
                        out.push('-');
                    } else {
                        if css_space_needed(&out) {
                            out.push(' ');
                        }
                        out.push('-');
                    }
                    i += 1;
                }
                ':' => {
                    out.push(':');
                    out.push(' ');
                    i += 1;
                }
                ';' => {
                    out.push_str("; ");
                    i += 1;
                }
                ',' => {
                    out.push_str(", ");
                    i += 1;
                }
                '#' => {
                    if css_space_needed(&out) {
                        out.push(' ');
                    }
                    out.push('#');
                    i += 1;
                    while i < tokens.len() {
                        match &tokens[i] {
                            TokenTree::Literal(l) => {
                                out.push_str(&l.to_string());
                                i += 1;
                            }
                            TokenTree::Ident(id) => {
                                out.push_str(&id.to_string());
                                i += 1;
                                break;
                            }
                            _ => break,
                        }
                    }
                }
                '.' => {
                    out.push('.');
                    i += 1;
                }
                '!' => {
                    out.push('!');
                    i += 1;
                }
                other => {
                    if css_space_needed(&out) {
                        out.push(' ');
                    }
                    out.push(other);
                    i += 1;
                }
            },

            TokenTree::Literal(l) => {
                let s = l.to_string();
                let is_numeric = s.starts_with(|c: char| c.is_ascii_digit());

                if css_space_needed(&out) && !out.ends_with('-') {
                    out.push(' ');
                }

                if (s.starts_with('"') && s.ends_with('"'))
                    || (s.starts_with('\'') && s.ends_with('\''))
                {
                    out.push_str(&s[1..s.len() - 1]);
                } else {
                    out.push_str(&s);
                }

                if is_numeric {
                    if let Some(TokenTree::Ident(unit)) = tokens.get(i + 1) {
                        let u = unit.to_string();
                        if CSS_UNITS.iter().any(|&x| x == u.as_str()) {
                            out.push_str(&u);
                            i += 2;
                            continue;
                        }
                    }
                }
                i += 1;
            }
        }
    }

    out.trim().to_string()
}

pub fn expand_brick_css_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let css = css_tokens_to_string(input.into_iter().collect());
    let lit = proc_macro::Literal::string(&css);
    proc_macro::TokenTree::Literal(lit).into()
}
