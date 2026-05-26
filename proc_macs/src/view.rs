use proc_macro2::{
    Delimiter, Group, Ident as Ident2, TokenStream as TokenStream2, TokenTree as TT,
};
use quote::quote;
use syn::{Ident, ItemFn, Stmt};

use crate::shared::split_by_top_comma;

pub const HTML_CONTAINER_ELEMENTS: &[&str] = &[
    "div", "span", "section", "header", "footer", "main", "nav", "article", "aside", "ul", "ol",
    "li", "form",
];

pub fn is_container_element(id: &Ident2) -> bool {
    let s = id.to_string();
    HTML_CONTAINER_ELEMENTS.iter().any(|&e| e == s)
}

pub fn is_fragment_keyword(id: &Ident2) -> bool {
    id.to_string() == "fragment"
}

pub fn is_css_macro(id: &Ident2) -> bool {
    matches!(id.to_string().as_str(), "style" | "css")
}

pub fn preprocess_view_body(ts: TokenStream2) -> TokenStream2 {
    let mut out = TokenStream2::new();
    let mut iter = ts.into_iter().peekable();
    while let Some(tt) = iter.next() {
        let (container, css_mac, fragment_kw) = if let TT::Ident(ref id) = tt {
            (
                is_container_element(id),
                is_css_macro(id),
                is_fragment_keyword(id),
            )
        } else {
            (false, false, false)
        };

        // CSS macros (style!, css!): emit name + ! + group verbatim, no recursion.
        if css_mac {
            out.extend(std::iter::once(tt));
            if let Some(TT::Punct(p)) = iter.peek() {
                if p.as_char() == '!' {
                    out.extend(std::iter::once(iter.next().unwrap()));
                    if let Some(TT::Group(_)) = iter.peek() {
                        out.extend(std::iter::once(iter.next().unwrap()));
                    }
                }
            }
            continue;
        }

        if fragment_kw {
            if let Some(TT::Group(g)) = iter.peek() {
                if g.delimiter() == Delimiter::Brace {
                    let group = match iter.next() {
                        Some(TT::Group(g)) => g,
                        _ => unreachable!(),
                    };
                    let inner = preprocess_view_body(group.stream());
                    out.extend(expand_fragment_block(inner));
                    continue;
                }
            }
            out.extend(std::iter::once(tt));
            continue;
        }

        if container {
            if let Some(TT::Group(g)) = iter.peek() {
                if g.delimiter() == Delimiter::Brace {
                    let id = match tt {
                        TT::Ident(id) => id,
                        _ => unreachable!(),
                    };
                    let group = match iter.next() {
                        Some(TT::Group(g)) => g,
                        _ => unreachable!(),
                    };
                    let inner = preprocess_view_body(group.stream());
                    out.extend(expand_element_block(&id, inner));
                    continue;
                }
            }
            out.extend(std::iter::once(tt));
            continue;
        }
        match tt {
            TT::Group(g) => {
                let inner = preprocess_view_body(g.stream());
                let mut new_g = Group::new(g.delimiter(), inner);
                new_g.set_span(g.span());
                out.extend(std::iter::once(TT::Group(new_g)));
            }
            other => out.extend(std::iter::once(other)),
        }
    }
    out
}

pub fn expand_element_block(tag: &Ident2, inner: TokenStream2) -> TokenStream2 {
    let is_form = tag.to_string() == "form";
    let items = split_by_top_comma(inner);

    // For form {}, detect `submit(EXPR)` item (first position only).
    if is_form {
        let (submit_opt, remaining_items): (Option<TokenStream2>, Vec<TokenStream2>) = {
            let first_tts: Vec<TT> = items
                .first()
                .map(|ts| ts.clone().into_iter().collect())
                .unwrap_or_default();
            let is_submit = first_tts.len() >= 2
                && matches!(&first_tts[0], TT::Ident(id) if id.to_string() == "submit")
                && matches!(&first_tts[1], TT::Group(g) if g.delimiter() == Delimiter::Parenthesis);
            if is_submit {
                let submit_expr = match &first_tts[1] {
                    TT::Group(g) => g.stream(),
                    _ => unreachable!(),
                };
                (Some(submit_expr), items[1..].to_vec())
            } else {
                (None, items)
            }
        };

        // Skip any class() directive in form (forms use auto-generated brick-form-N class)
        let child_items: Vec<TokenStream2> = remaining_items
            .into_iter()
            .filter(|ts| {
                if ts.is_empty() {
                    return false;
                }
                let tts: Vec<TT> = ts.clone().into_iter().collect();
                let is_class = tts.len() >= 2
                    && matches!(&tts[0], TT::Ident(id) if id.to_string() == "class")
                    && matches!(&tts[1], TT::Group(g) if g.delimiter() == Delimiter::Parenthesis);
                !is_class
            })
            .collect();

        let child_calls: Vec<TokenStream2> = child_items
            .into_iter()
            .filter(|ts| !ts.is_empty())
            .map(|child| quote! { (#child).into_component() })
            .collect();

        return match submit_opt {
            None => quote! {{
                use crate::view_components::IntoComponent as _;
                crate::view_components::BrickForm::new(
                    None,
                    vec![ #(#child_calls),* ],
                )
            }},
            Some(submit_ts) => quote! {{
                use crate::view_components::IntoComponent as _;
                crate::view_components::BrickForm::new(
                    Some((#submit_ts).name),
                    vec![ #(#child_calls),* ],
                )
            }},
        };
    }

    // Non-form element: detect leading `class(EXPR)` item
    let (class_opt, child_items): (Option<TokenStream2>, Vec<TokenStream2>) = {
        let first_tts: Vec<TT> = items
            .first()
            .map(|ts| ts.clone().into_iter().collect())
            .unwrap_or_default();
        let is_class = first_tts.len() >= 2
            && matches!(&first_tts[0], TT::Ident(id) if id.to_string() == "class")
            && matches!(&first_tts[1], TT::Group(g) if g.delimiter() == Delimiter::Parenthesis);
        if is_class {
            let class_expr = match &first_tts[1] {
                TT::Group(g) => g.stream(),
                _ => unreachable!(),
            };
            (Some(class_expr), items[1..].to_vec())
        } else {
            (None, items)
        }
    };

    let child_calls: Vec<TokenStream2> = child_items
        .into_iter()
        .filter(|ts| !ts.is_empty())
        .map(|child| quote! { (#child).into_component() })
        .collect();

    match class_opt {
        None => quote! {{
            use crate::view_components::IntoComponent as _;
            Box::new(crate::view_components::BrickContainer {
                class: "",
                children: vec![ #(#child_calls),* ],
            })
        }},
        Some(class_ts) => {
            // String literal → static BrickContainer; anything else → reactive ReactiveDiv
            let class_tts: Vec<TT> = class_ts.clone().into_iter().collect();
            let is_string_lit = class_tts.len() == 1
                && matches!(&class_tts[0], TT::Literal(lit) if lit.to_string().starts_with('"'));
            if is_string_lit {
                quote! {{
                    use crate::view_components::IntoComponent as _;
                    Box::new(crate::view_components::BrickContainer {
                        class: #class_ts,
                        children: vec![ #(#child_calls),* ],
                    })
                }}
            } else {
                quote! {{
                    use crate::view_components::IntoComponent as _;
                    crate::view_components::ReactiveDiv::new(
                        &(#class_ts),
                        vec![ #(#child_calls),* ],
                    )
                }}
            }
        }
    }
}

pub fn expand_fragment_block(inner: TokenStream2) -> TokenStream2 {
    let items = split_by_top_comma(inner);
    let child_calls: Vec<TokenStream2> = items
        .into_iter()
        .filter(|ts| !ts.is_empty())
        .map(|child| quote! { (#child).into_component() })
        .collect();
    quote! {{
        use crate::view_components::IntoComponent as _;
        crate::view_components::BrickFragment::boxed(vec![ #(#child_calls),* ])
    }}
}

pub fn expand(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let struct_name: Ident = match syn::parse2(attr) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let preprocessed: TokenStream2 = preprocess_view_body(item);
    let mut func: ItemFn = match syn::parse2(preprocessed) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let portable_body = extract_portable_block(&mut func.block.stmts);

    let func_block = func.block;
    let mut statements = func_block.stmts;
    let tail = statements.remove(statements.len() - 1);

    let portable_impl = portable_body.map(|body| {
        quote! {
            impl crate::portable::PortableView for #struct_name {
                fn blueprint_node(&self) -> crate::portable::BlueprintNode {
                    use crate::portable::PortableView as _;
                    (#body).blueprint_node()
                }
            }
        }
    });

    quote! {
        #[cfg(not(brick_android))]
        impl crate::view_components::IntoComponent for #struct_name {
            fn into_component(mut self) -> Box<dyn crate::view_components::Brick> {
                use crate::state_mgmt::BrickModel as _;
                let _brick_instance_class = format!("{}_{}",
                    stringify!(#struct_name),
                    crate::INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                );
                crate::view_components::set_scope(&_brick_instance_class);
                let (event_listeners, lifecycle, intervals, model_rc, raf_callbacks) = self.controller_methods();
                use crate::state_mgmt::BrickWrapDefault as _;
                let my = model_rc.borrow().clone().wrap_for_view();
                #( #statements )*
                make_composite!(_brick_instance_class, event_listeners, lifecycle, intervals, (model_rc as std::rc::Rc<std::cell::RefCell<dyn std::any::Any>>), raf_callbacks, #tail)
            }
        }

        #portable_impl
    }
}

fn extract_portable_block(stmts: &mut Vec<Stmt>) -> Option<TokenStream2> {
    // syn 1.x parses `portable! { ... }` (brace-delimited, no semicolon) as
    // `Stmt::Item(Item::Macro)` in non-tail position. Cover all three forms.
    let pos = stmts.iter().position(|stmt| match stmt {
        Stmt::Expr(syn::Expr::Macro(em), _) => em.mac.path.is_ident("portable"),
        Stmt::Item(syn::Item::Macro(im)) => im.mac.path.is_ident("portable"),
        _ => false,
    })?;
    match stmts.remove(pos) {
        Stmt::Expr(syn::Expr::Macro(em), _) => Some(em.mac.tokens),
        Stmt::Item(syn::Item::Macro(im)) => Some(im.mac.tokens),
        _ => None,
    }
}
