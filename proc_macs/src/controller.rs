use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, ImplItem, ItemImpl};

use crate::shared::{OnAttr, OnExtractor};

pub fn expand(_attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let mut impl_input: ItemImpl = match syn::parse2(item) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let model_name = *impl_input.self_ty.clone();

    let model_ident = if let syn::Type::Path(tp) = &*impl_input.self_ty {
        tp.path.segments.last().unwrap().ident.clone()
    } else {
        return syn::Error::new_spanned(
            &*impl_input.self_ty,
            "#[controller] requires a named struct",
        )
        .to_compile_error();
    };
    let view_name = format_ident!("{}View", model_ident);

    enum MethodKind {
        DomEvent {
            event_type: String,
            extractor: Option<Ident>,
        },
        Created,
        Mount,
        BeforeUnmount,
        Unmount,
        Watch {
            field: Ident,
        },
        Interval(i32),
        AnimationFrame,
        Submit,
    }

    struct MethodInfo {
        ident: Ident,
        kind: MethodKind,
    }

    let mut methods: Vec<MethodInfo> = Vec::new();

    for item in impl_input.items.iter_mut() {
        if let ImplItem::Fn(meth) = item {
            let on_attr = meth.attrs.iter().find(|a| a.path().is_ident("on"));

            let kind = match on_attr {
                Some(attr) => match attr.parse_args::<OnAttr>() {
                    Ok(parsed) => {
                        let ev = parsed.event_type.to_string();
                        match ev.as_str() {
                            "created" => MethodKind::Created,
                            "mount" => MethodKind::Mount,
                            "before_unmount" => MethodKind::BeforeUnmount,
                            "unmount" => MethodKind::Unmount,
                            "submit" => MethodKind::Submit,
                            "watch" => {
                                let field = match parsed.extractor {
                                    Some(OnExtractor::Ident(id)) => id,
                                    _ => return syn::Error::new(
                                        proc_macro2::Span::call_site(),
                                        "#[on(watch)] requires a field name, e.g. #[on(watch, my_field)]",
                                    )
                                    .to_compile_error(),
                                };
                                MethodKind::Watch { field }
                            }
                            "interval" => {
                                let ms = match parsed.extractor {
                                    Some(OnExtractor::Int(lit)) => {
                                        lit.base10_parse::<i32>().unwrap_or(1000)
                                    }
                                    _ => 1000,
                                };
                                MethodKind::Interval(ms)
                            }
                            "animation_frame" => MethodKind::AnimationFrame,
                            _ => {
                                let extractor = match parsed.extractor {
                                    Some(OnExtractor::Ident(id)) => Some(id),
                                    _ => None,
                                };
                                MethodKind::DomEvent {
                                    event_type: ev,
                                    extractor,
                                }
                            }
                        }
                    }
                    Err(_) => MethodKind::DomEvent {
                        event_type: "click".to_string(),
                        extractor: None,
                    },
                },
                None => MethodKind::DomEvent {
                    event_type: "click".to_string(),
                    extractor: None,
                },
            };

            meth.attrs.retain(|a| !a.path().is_ident("on"));
            methods.push(MethodInfo {
                ident: meth.sig.ident.clone(),
                kind,
            });
        }
    }

    let listener_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| match &info.kind {
            MethodKind::DomEvent {
                event_type,
                extractor,
            } => {
                let method_ident = &info.ident;
                let closure = build_closure(method_ident, extractor);
                Some(quote! {
                    event_listeners.insert(stringify!(#method_ident), (#event_type, #closure));
                })
            }
            MethodKind::Submit => {
                let method_ident = &info.ident;
                Some(quote! {
                    event_listeners.insert(stringify!(#method_ident), ("click", {
                        let s = std::rc::Rc::clone(&sharing_model);
                        std::rc::Rc::new(move |_e: crate::renderer::BrickEvent| {
                            if s.borrow_mut().validate_all() {
                                s.borrow_mut().#method_ident();
                            }
                        }) as std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>
                    }));
                })
            }
            _ => None,
        })
        .collect();

    let mount_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| {
            if let MethodKind::Mount = &info.kind {
                let method_ident = &info.ident;
                Some(quote! {
                    lifecycle.on_mount.push({
                        let s = std::rc::Rc::clone(&sharing_model);
                        std::rc::Rc::new(move || s.borrow_mut().#method_ident())
                            as std::rc::Rc<dyn Fn()>
                    });
                })
            } else {
                None
            }
        })
        .collect();

    let created_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| {
            if let MethodKind::Created = &info.kind {
                let method_ident = &info.ident;
                Some(quote! {
                    sharing_model.borrow_mut().#method_ident();
                })
            } else {
                None
            }
        })
        .collect();

    let before_unmount_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| {
            if let MethodKind::BeforeUnmount = &info.kind {
                let method_ident = &info.ident;
                Some(quote! {
                    lifecycle.on_before_unmount.push({
                        let s = std::rc::Rc::clone(&sharing_model);
                        std::rc::Rc::new(move || s.borrow_mut().#method_ident())
                            as std::rc::Rc<dyn Fn()>
                    });
                })
            } else {
                None
            }
        })
        .collect();

    let unmount_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| {
            if let MethodKind::Unmount = &info.kind {
                let method_ident = &info.ident;
                Some(quote! {
                    lifecycle.on_unmount.push({
                        let s = std::rc::Rc::clone(&sharing_model);
                        std::rc::Rc::new(move || s.borrow_mut().#method_ident())
                            as std::rc::Rc<dyn Fn()>
                    });
                })
            } else {
                None
            }
        })
        .collect();

    let watch_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| {
            if let MethodKind::Watch { field } = &info.kind {
                let method_ident = &info.ident;
                Some(quote! {
                    {
                        let s = std::rc::Rc::clone(&sharing_model);
                        let _sig = sharing_model.borrow().#field.clone();
                        _sig.add_observer(
                            stringify!(#method_ident),
                            Box::new(crate::state_mgmt::Effect::new(move |_| {
                                s.borrow_mut().#method_ident();
                            }))
                        );
                    }
                })
            } else {
                None
            }
        })
        .collect();

    let interval_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| {
            if let MethodKind::Interval(ms) = &info.kind {
                let method_ident = &info.ident;
                Some(quote! {
                    intervals.push((#ms, {
                        let s = std::rc::Rc::clone(&sharing_model);
                        std::rc::Rc::new(move || s.borrow_mut().#method_ident())
                            as std::rc::Rc<dyn Fn()>
                    }));
                })
            } else {
                None
            }
        })
        .collect();

    let raf_inserts: Vec<TokenStream2> = methods
        .iter()
        .filter_map(|info| {
            if let MethodKind::AnimationFrame = &info.kind {
                let method_ident = &info.ident;
                Some(quote! {
                    raf_callbacks.push({
                        let s = std::rc::Rc::clone(&sharing_model);
                        std::rc::Rc::new(move || s.borrow_mut().#method_ident())
                            as std::rc::Rc<dyn Fn()>
                    });
                })
            } else {
                None
            }
        })
        .collect();

    let dom_methods: Vec<_> = methods
        .iter()
        .filter(|m| matches!(m.kind, MethodKind::DomEvent { .. } | MethodKind::Submit))
        .collect();
    let has_dom_actions = !dom_methods.is_empty();

    let action_field_decls: Vec<TokenStream2> = dom_methods
        .iter()
        .map(|m| {
            let field_name = &m.ident;
            quote! { pub #field_name: crate::state_mgmt::BrickAction, }
        })
        .collect();

    let action_field_inits: Vec<TokenStream2> = dom_methods.iter().map(|m| {
        let field_name = &m.ident;
        let method_str = m.ident.to_string();
        let event_str: &str = match &m.kind {
            MethodKind::DomEvent { event_type, .. } => event_type.as_str(),
            MethodKind::Submit => "click",
            _ => unreachable!(),
        };
        quote! { #field_name: crate::state_mgmt::BrickAction { name: #method_str, event: #event_str }, }
    }).collect();

    // Associated constants: `ModelName::METHOD_NAME` (uppercase) for each DOM action.
    // Used in `portable! {}` view blocks to bind actions without the view wrapper.
    let action_const_decls: Vec<TokenStream2> = dom_methods.iter().map(|m| {
        let const_name = format_ident!("{}", m.ident.to_string().to_uppercase());
        let method_str = m.ident.to_string();
        let event_str: &str = match &m.kind {
            MethodKind::DomEvent { event_type, .. } => event_type.as_str(),
            MethodKind::Submit => "click",
            _ => unreachable!(),
        };
        quote! {
            pub const #const_name: crate::state_mgmt::BrickAction =
                crate::state_mgmt::BrickAction { name: #method_str, event: #event_str };
        }
    }).collect();

    // Android action registration: no-extractor → register_action; String → register_string_action.
    let android_action_inserts: Vec<TokenStream2> = dom_methods.iter()
        .filter_map(|m| {
            let method_ident = &m.ident;
            let method_str = m.ident.to_string();
            match &m.kind {
                MethodKind::DomEvent { extractor: None, .. } | MethodKind::Submit => Some(quote! {
                    {
                        let m = std::rc::Rc::clone(&model);
                        crate::native::register_action(#method_str, move || m.borrow_mut().#method_ident());
                    }
                }),
                MethodKind::DomEvent { extractor: Some(ext), .. }
                    if ext.to_string() == "String" => Some(quote! {
                    {
                        let m = std::rc::Rc::clone(&model);
                        crate::native::register_string_action(#method_str, move |s: String| m.borrow_mut().#method_ident(s));
                    }
                }),
                MethodKind::DomEvent { extractor: Some(ext), .. }
                    if ext.to_string() == "i32" => Some(quote! {
                    {
                        let m = std::rc::Rc::clone(&model);
                        crate::native::register_int_action(#method_str, move |i: i32| m.borrow_mut().#method_ident(i));
                    }
                }),
                MethodKind::DomEvent { extractor: Some(ext), .. }
                    if ext.to_string() == "bool" => Some(quote! {
                    {
                        let m = std::rc::Rc::clone(&model);
                        crate::native::register_bool_action(#method_str, move |b: bool| m.borrow_mut().#method_ident(b));
                    }
                }),
                _ => None,
            }
        })
        .collect();

    let register_android_actions_method = if !android_action_inserts.is_empty() {
        quote! {
            #[cfg(brick_android)]
            pub fn register_android_actions(model: std::rc::Rc<std::cell::RefCell<Self>>) {
                #( #android_action_inserts )*
            }
        }
    } else {
        quote! {}
    };

    // Android interval registration: one `register_interval` call per #[on(interval)] method.
    let interval_registrations: Vec<TokenStream2> = methods.iter()
        .filter_map(|info| {
            if let MethodKind::Interval(ms) = &info.kind {
                let method_ident = &info.ident;
                let ms_u64 = *ms as u64;
                Some(quote! {
                    {
                        let m = std::rc::Rc::clone(&model);
                        crate::native::register_interval(#ms_u64, move || m.borrow_mut().#method_ident());
                    }
                })
            } else {
                None
            }
        })
        .collect();

    let register_android_intervals_method = if !interval_registrations.is_empty() {
        quote! {
            #[cfg(brick_android)]
            pub fn register_android_intervals(model: std::rc::Rc<std::cell::RefCell<Self>>) {
                #( #interval_registrations )*
            }
        }
    } else {
        quote! {}
    };

    let view_struct_tokens = if has_dom_actions {
        quote! {
            #[cfg(not(brick_android))]
            pub struct #view_name {
                pub __inner: #model_name,
                #( #action_field_decls )*
            }

            #[cfg(not(brick_android))]
            impl std::ops::Deref for #view_name {
                type Target = #model_name;
                fn deref(&self) -> &Self::Target {
                    &self.__inner
                }
            }
        }
    } else {
        quote! {}
    };

    let wrap_for_view_method = if has_dom_actions {
        quote! {
            #[cfg(not(brick_android))]
            pub fn wrap_for_view(self) -> #view_name {
                #view_name {
                    #( #action_field_inits )*
                    __inner: self,
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #impl_input

        #view_struct_tokens

        impl #model_name {
            #( #action_const_decls )*

            #wrap_for_view_method

            #register_android_actions_method

            #register_android_intervals_method

            #[cfg(not(brick_android))]
            pub fn controller_methods(mut self) -> (
                std::collections::HashMap<&'static str, (&'static str, std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>)>,
                crate::state_mgmt::BrickLifecycle,
                Vec<(i32, std::rc::Rc<dyn Fn()>)>,
                std::rc::Rc<std::cell::RefCell<#model_name>>,
                Vec<std::rc::Rc<dyn Fn()>>,
            ) {
                <Self as crate::state_mgmt::BrickModel>::wire_cascade(&mut self);
                let mut event_listeners = std::collections::HashMap::new();
                let mut lifecycle = crate::state_mgmt::BrickLifecycle::default();
                let mut intervals: Vec<(i32, std::rc::Rc<dyn Fn()>)> = Vec::new();
                let mut raf_callbacks: Vec<std::rc::Rc<dyn Fn()>> = Vec::new();
                let sharing_model = std::rc::Rc::new(std::cell::RefCell::new(self));
                sharing_model.borrow().__wire_form_validation();
                #( #created_inserts )*
                #( #listener_inserts )*
                #( #mount_inserts )*
                #( #before_unmount_inserts )*
                #( #unmount_inserts )*
                #( #watch_inserts )*
                #( #interval_inserts )*
                #( #raf_inserts )*
                (event_listeners, lifecycle, intervals, sharing_model, raf_callbacks)
            }
        }
    }
}

/// Generate the event-listener closure for one controller method.
///
/// `extractor` maps to the Adapter strategy:
///   - `None`      → no DOM extraction; method takes only `&mut self`
///   - `f64`       → number input value (NaN filtered)
///   - `String`    → text input value
///   - `bool`      → checkbox checked state
///   - `raw`       → raw `web_sys::Event` forwarded (opt-in escape hatch)
pub fn build_closure(method: &Ident, extractor: &Option<Ident>) -> TokenStream2 {
    match extractor {
        None => quote! {{
            let s = std::rc::Rc::clone(&sharing_model);
            std::rc::Rc::new(move |_e: crate::renderer::BrickEvent| {
                s.borrow_mut().#method();
            }) as std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>
        }},
        Some(ext) => match ext.to_string().as_str() {
            "f64" => quote! {{
                let s = std::rc::Rc::clone(&sharing_model);
                std::rc::Rc::new(move |e: crate::renderer::BrickEvent| {
                    if let crate::renderer::BrickEvent::InputNumber(v) = e {
                        s.borrow_mut().#method(v);
                    }
                }) as std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>
            }},
            "String" => quote! {{
                let s = std::rc::Rc::clone(&sharing_model);
                std::rc::Rc::new(move |e: crate::renderer::BrickEvent| {
                    if let crate::renderer::BrickEvent::InputString(v) = e {
                        s.borrow_mut().#method(v);
                    }
                }) as std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>
            }},
            "bool" => quote! {{
                let s = std::rc::Rc::clone(&sharing_model);
                std::rc::Rc::new(move |e: crate::renderer::BrickEvent| {
                    if let crate::renderer::BrickEvent::InputBool(v) = e {
                        s.borrow_mut().#method(v);
                    }
                }) as std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>
            }},
            "coords" => quote! {{
                let s = std::rc::Rc::clone(&sharing_model);
                std::rc::Rc::new(move |e: crate::renderer::BrickEvent| {
                    if let crate::renderer::BrickEvent::ClickAt(x, y) = e {
                        s.borrow_mut().#method(x, y);
                    }
                }) as std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>
            }},
            _ => syn::Error::new(
                ext.span(),
                format!("unknown extractor '{}'; use f64, String, bool, or coords", ext),
            )
            .to_compile_error(),
        },
    }
}
