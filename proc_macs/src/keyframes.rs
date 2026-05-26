use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::css::{CssDynValue, process_css_with_dyn};

/// Parse the stop list inside `{ 0% { ... } 50% { ... } from { ... } to { ... } }`.
fn parse_stops(
    tokens: Vec<proc_macro::TokenTree>,
    dyn_values: &mut Vec<CssDynValue>,
) -> Result<String, String> {
    use proc_macro::{Delimiter, TokenTree as PTT};
    let mut out = String::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            PTT::Literal(l) => {
                let num = l.to_string();
                let has_pct =
                    matches!(tokens.get(i + 1), Some(PTT::Punct(p)) if p.as_char() == '%');
                let brace_idx = if has_pct { i + 2 } else { i + 1 };
                if let Some(PTT::Group(g)) = tokens.get(brace_idx) {
                    if g.delimiter() == Delimiter::Brace {
                        let decls: Vec<PTT> = g.stream().into_iter().collect();
                        let body = process_css_with_dyn(decls, dyn_values, 1, false);
                        out.push_str(&format!("  {}% {{ {} }}\n", num, body));
                        i = brace_idx + 1;
                        continue;
                    }
                }
                return Err(format!(
                    "keyframes!: expected `{}%` followed by a `{{ }}` block",
                    num
                ));
            }
            PTT::Ident(id) => {
                let kw = id.to_string();
                if kw == "from" || kw == "to" {
                    if let Some(PTT::Group(g)) = tokens.get(i + 1) {
                        if g.delimiter() == Delimiter::Brace {
                            let decls: Vec<PTT> = g.stream().into_iter().collect();
                            let body = process_css_with_dyn(decls, dyn_values, 1, false);
                            out.push_str(&format!("  {} {{ {} }}\n", kw, body));
                            i += 2;
                            continue;
                        }
                    }
                    return Err(format!(
                        "keyframes!: `{}` must be followed by a `{{ }}` block",
                        kw
                    ));
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    Ok(out)
}

pub fn expand_keyframes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use proc_macro::{Delimiter, TokenTree as PTT};

    let tokens: Vec<PTT> = input.into_iter().collect();

    // Expect a single `{ stops }` block — no name argument.
    let stop_tokens = match tokens.first() {
        Some(PTT::Group(g)) if g.delimiter() == Delimiter::Brace => {
            g.stream().into_iter().collect::<Vec<PTT>>()
        }
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "keyframes!: expected a `{ }` block of keyframe stops, e.g. keyframes!({ 0% { opacity: 0; } 100% { opacity: 1; } })",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut dyn_values: Vec<CssDynValue> = Vec::new();
    let stops_css = match parse_stops(stop_tokens, &mut dyn_values) {
        Ok(s) => s,
        Err(msg) => {
            return syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into();
        }
    };

    // The name is generated at runtime so each component instance gets a unique identifier.
    // inject_global_style and reactive observers are gated on #[cfg(not(test))].
    let mut not_test_stmts: TokenStream2 = quote! {
        crate::view_components::inject_global_style(
            &format!("@keyframes {} {{\n{}}}", _kf.as_str(), #stops_css)
        );
    };

    if !dyn_values.is_empty() {
        not_test_stmts.extend(quote! {
            let _scope = crate::view_components::get_scope();
        });
        for dyn_val in &dyn_values {
            let var_name = &dyn_val.var_name;
            let expr = &dyn_val.expr_tokens;
            let fmt_str = &dyn_val.format_str;
            not_test_stmts.extend(quote! {
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
    }

    proc_macro::TokenStream::from(quote! {
        {
            let _kf = crate::view_components::KeyframeName::generate();
            #[cfg(not(test))]
            { #not_test_stmts }
            _kf
        }
    })
}
