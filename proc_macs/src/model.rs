use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Expr, ItemStruct, Type};

use crate::shared::{
    FieldKind, FromAttr, ValidatorSpec, generate_validator_body, is_list, is_navigator,
    is_resource, parse_validators,
};

pub fn expand(_attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let mut item: ItemStruct = match syn::parse2(item) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let struct_name = item.ident.clone();

    struct FieldInfo {
        name: Ident,
        ty: Type,
        kind: FieldKind,
    }

    let mut fields: Vec<FieldInfo> = Vec::new();

    for field in item.fields.iter_mut() {
        let name = field.ident.clone().unwrap();
        let ty = field.ty.clone();

        let prop_pos = field.attrs.iter().position(|a| a.path().is_ident("prop"));
        let slot_pos = field.attrs.iter().position(|a| a.path().is_ident("slot"));
        let from_pos = field.attrs.iter().position(|a| a.path().is_ident("from"));
        let default_pos = field.attrs.iter().position(|a| a.path().is_ident("default"));
        let persist_pos = field.attrs.iter().position(|a| a.path().is_ident("persist"));
        let store_pos = field.attrs.iter().position(|a| a.path().is_ident("store"));
        let global_pos = field.attrs.iter().position(|a| a.path().is_ident("global"));
        let validate_pos = field.attrs.iter().position(|a| a.path().is_ident("validate"));

        let kind = if let Some(pos) = prop_pos {
            field.attrs.remove(pos);
            FieldKind::Prop
        } else if let Some(pos) = slot_pos {
            field.attrs.remove(pos);
            FieldKind::Slot
        } else if let Some(pos) = from_pos {
            let attr = field.attrs.remove(pos);
            match attr.parse_args::<FromAttr>() {
                Ok(parsed) => FieldKind::Derived(parsed),
                Err(e) => return e.to_compile_error(),
            }
        } else if is_navigator(&ty) {
            // Navigator<S> fields are plain state (not Signal). They require #[default(initial_screen)].
            if let Some(pos) = default_pos {
                let attr = field.attrs.remove(pos);
                match attr.parse_args::<Expr>() {
                    Ok(val) => FieldKind::Nav(val),
                    Err(e) => return e.to_compile_error(),
                }
            } else {
                return syn::Error::new_spanned(
                    &field.ty,
                    "Navigator<S> fields require #[default(InitialScreen)] to specify the initial screen",
                )
                .to_compile_error();
            }
        } else if is_resource(&ty) {
            // Resource<T> fields are detected by type before #[default] so we can
            // consume the optional #[default] attr and emit the expr directly (no Signal wrapping).
            if let Some(pos) = default_pos {
                let attr = field.attrs.remove(pos);
                match attr.parse_args::<Expr>() {
                    Ok(val) => FieldKind::ResourceWithDefault(val),
                    Err(e) => return e.to_compile_error(),
                }
            } else {
                FieldKind::Resource
            }
        } else if let Some(pos) = default_pos {
            let attr = field.attrs.remove(pos);
            match attr.parse_args::<Expr>() {
                Ok(val) => FieldKind::Default(val),
                Err(e) => return e.to_compile_error(),
            }
        } else if let Some(pos) = validate_pos {
            let attr = field.attrs.remove(pos);
            let inner_ts: TokenStream2 =
                match attr.parse_args_with(|ps: syn::parse::ParseStream| {
                    ps.parse::<proc_macro2::TokenStream>()
                }) {
                    Ok(ts) => ts,
                    Err(e) => return e.to_compile_error(),
                };
            match parse_validators(inner_ts) {
                Ok(validators) => FieldKind::Validated { validators },
                Err(e) => return e.to_compile_error(),
            }
        } else if let Some(pos) = persist_pos {
            let attr = field.attrs.remove(pos);
            match attr.parse_args::<syn::LitStr>() {
                Ok(key) => FieldKind::Persist(key.value()),
                Err(e) => return e.to_compile_error(),
            }
        } else if let Some(pos) = store_pos {
            field.attrs.remove(pos);
            FieldKind::Store
        } else if let Some(pos) = global_pos {
            field.attrs.remove(pos);
            FieldKind::Global
        } else if is_resource(&ty) {
            FieldKind::Resource
        } else if is_list(&ty) {
            FieldKind::List
        } else {
            FieldKind::Primary
        };

        fields.push(FieldInfo { name, ty, kind });
    }

    // wire_cascade replaces #[from] fields with compute() post-construction
    // and wires #[persist] fields to save on change.
    let wire_stmts: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            if let FieldKind::Derived(from_attr) = &f.kind {
                let name = &f.name;
                let source = &from_attr.source;
                let func = &from_attr.func;
                Some(quote! {
                    self.#name = crate::state_mgmt::Signal::from_rc(
                        crate::state_mgmt::compute(self.#source.rc(), #func)
                    );
                })
            } else if let FieldKind::Persist(key) = &f.kind {
                let name = &f.name;
                let ty = &f.ty;
                let obs_name = format!("__persist_{}", name);
                Some(quote! {
                    {
                        self.#name.add_observer(
                            #obs_name,
                            Box::new(crate::state_mgmt::Effect::new(move |val: &#ty| {
                                crate::browser::BrickStorage::set(#key, val);
                            }))
                        );
                    }
                })
            } else {
                None
            }
        })
        .collect();

    // Collect validated field info for companion injection
    let validated_fields: Vec<(&Ident, &Type, &Vec<ValidatorSpec>)> = fields
        .iter()
        .filter_map(|f| {
            if let FieldKind::Validated { validators } = &f.kind {
                Some((&f.name, &f.ty, validators))
            } else {
                None
            }
        })
        .collect();

    // Companion field declarations injected into the struct for each #[validate] field
    let companion_field_decls: Vec<TokenStream2> = validated_fields
        .iter()
        .flat_map(|(name, _, _)| {
            let error_field = format_ident!("{}_error", name);
            let touched_field = format_ident!("{}_touched", name);
            let dirty_field = format_ident!("{}_dirty", name);
            vec![
                quote! { pub #error_field: crate::state_mgmt::Signal<String>, },
                quote! { pub #touched_field: crate::state_mgmt::Signal<bool>, },
                quote! { pub #dirty_field: crate::state_mgmt::Signal<bool>, },
            ]
        })
        .collect();

    let companion_clone_exprs: Vec<TokenStream2> = validated_fields
        .iter()
        .flat_map(|(name, _, _)| {
            let error_field = format_ident!("{}_error", name);
            let touched_field = format_ident!("{}_touched", name);
            let dirty_field = format_ident!("{}_dirty", name);
            vec![
                quote! { #error_field: self.#error_field.clone(), },
                quote! { #touched_field: self.#touched_field.clone(), },
                quote! { #dirty_field: self.#dirty_field.clone(), },
            ]
        })
        .collect();

    let companion_cascade_inits: Vec<TokenStream2> = validated_fields
        .iter()
        .flat_map(|(name, _, _)| {
            let error_field = format_ident!("{}_error", name);
            let touched_field = format_ident!("{}_touched", name);
            let dirty_field = format_ident!("{}_dirty", name);
            vec![
                quote! { #error_field: crate::state_mgmt::Signal::new(String::new()), },
                quote! { #touched_field: crate::state_mgmt::Signal::new(false), },
                quote! { #dirty_field: crate::state_mgmt::Signal::new(false), },
            ]
        })
        .collect();

    // Aggregate form-state fields: valid / dirty / touched — only when the model has validated fields.
    // `valid` starts false; becomes true only after the form is touched AND all errors are empty.
    // `dirty` and `touched` are one-way latches: true as soon as any field changes / is interacted with.
    let has_form = !validated_fields.is_empty();

    let agg_field_decls: Vec<TokenStream2> = if has_form {
        vec![
            quote! { pub valid: crate::state_mgmt::Signal<bool>, },
            quote! { pub dirty: crate::state_mgmt::Signal<bool>, },
            quote! { pub touched: crate::state_mgmt::Signal<bool>, },
        ]
    } else {
        vec![]
    };

    let agg_clone_exprs: Vec<TokenStream2> = if has_form {
        vec![
            quote! { valid: self.valid.clone(), },
            quote! { dirty: self.dirty.clone(), },
            quote! { touched: self.touched.clone(), },
        ]
    } else {
        vec![]
    };

    let agg_cascade_inits: Vec<TokenStream2> = if has_form {
        vec![
            quote! { valid: crate::state_mgmt::Signal::new(false), },
            quote! { dirty: crate::state_mgmt::Signal::new(false), },
            quote! { touched: crate::state_mgmt::Signal::new(false), },
        ]
    } else {
        vec![]
    };

    // Struct field declarations — plain types for #[prop]/#[slot], Signal<T> for reactive.
    let mut struct_field_decls: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            match &f.kind {
                FieldKind::Prop | FieldKind::Slot => quote! { pub #name: #ty, },
                FieldKind::List => quote! { pub #name: std::rc::Rc<std::cell::RefCell<#ty>>, },
                FieldKind::Store
                | FieldKind::Global
                | FieldKind::Resource
                | FieldKind::Nav(_)
                | FieldKind::ResourceWithDefault(_) => quote! { pub #name: #ty, },
                _ => quote! { pub #name: crate::state_mgmt::Signal<#ty>, },
            }
        })
        .collect();
    struct_field_decls.extend(companion_field_decls);
    struct_field_decls.extend(agg_field_decls);

    // Clone — Signal and List fields clone cheaply via Rc, prop/slot use .clone().
    let mut clone_exprs: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            match &f.kind {
                FieldKind::List => quote! { #name: std::rc::Rc::clone(&self.#name), },
                // Resource<T> is already Clone via its internal Rc — use regular clone
                FieldKind::Resource | FieldKind::ResourceWithDefault(_) => {
                    quote! { #name: self.#name.clone(), }
                }
                _ => quote! { #name: self.#name.clone(), },
            }
        })
        .collect();
    clone_exprs.extend(companion_clone_exprs);
    clone_exprs.extend(agg_clone_exprs);

    let cascade_impl = {
        let mut cascade_inits: Vec<TokenStream2> = fields
            .iter()
            .map(|f| {
                let name = &f.name;
                let ty = &f.ty;
                match &f.kind {
                    FieldKind::Prop => quote! { #name: Default::default(), },
                    FieldKind::Slot => quote! { #name: crate::view_components::Slot::default(), },
                    FieldKind::List => quote! {
                        #name: std::rc::Rc::new(std::cell::RefCell::new(<#ty>::new())),
                    },
                    FieldKind::Default(val) => quote! {
                        #name: crate::state_mgmt::Signal::new(#val),
                    },
                    FieldKind::Store => quote! { #name: <#ty>::new(), },
                    FieldKind::Global => quote! { #name: <#ty>::get(), },
                    FieldKind::Resource => quote! {
                        #name: crate::state_mgmt::Resource::lazy(|| async { Err(String::new()) }),
                    },
                    FieldKind::ResourceWithDefault(val) => quote! { #name: #val, },
                    // Validated fields default to T::default() so validate_*() can read them
                    // without panicking when a test builds via `..cascade()`.
                    FieldKind::Validated { .. } => quote! {
                        #name: crate::state_mgmt::Signal::new(<#ty as Default>::default()),
                    },
                    FieldKind::Persist(key) => quote! {
                        #name: crate::state_mgmt::Signal::new(
                            crate::browser::BrickStorage::get::<#ty>(#key).unwrap_or_default()
                        ),
                    },
                    FieldKind::Nav(val) => quote! {
                        #name: crate::portable::Navigator::new(#val),
                    },
                    _ => quote! {
                        #name: crate::state_mgmt::Signal::placeholder(),
                    },
                }
            })
            .collect();
        cascade_inits.extend(companion_cascade_inits);
        cascade_inits.extend(agg_cascade_inits);

        quote! {
            impl crate::state_mgmt::Cascade for #struct_name {
                fn cascade_defaults() -> Self {
                    #struct_name { #( #cascade_inits )* }
                }
            }
        }
    };

    // Generate validate_{field} methods for each #[validate] field
    let validate_field_methods: Vec<TokenStream2> = validated_fields
        .iter()
        .map(|(name, ty, validators)| {
            let validate_method = format_ident!("validate_{}", name);
            let body = generate_validator_body(name, ty, validators);
            quote! {
                pub fn #validate_method(&mut self) -> bool {
                    #body
                }
            }
        })
        .collect();

    // validate_all calls each validate_{field} and ANDs results
    let validate_all_calls: Vec<TokenStream2> = validated_fields
        .iter()
        .map(|(name, _, _)| {
            let validate_method = format_ident!("validate_{}", name);
            quote! { __valid &= self.#validate_method(); }
        })
        .collect();

    // __wire_form_validation:
    // 1. field signal → {field}_dirty (existing: set dirty latch on field change)
    // 2. {field}_dirty → self.dirty aggregate (latch: any field dirty → form dirty)
    // 3. {field}_touched → self.touched aggregate (latch: any field touched → form touched)
    // 4. self.touched + each {field}_error → self.valid (valid = touched && all errors empty)

    let wire_field_dirty_stmts: Vec<TokenStream2> = validated_fields
        .iter()
        .map(|(name, _, _)| {
            let dirty_field = format_ident!("{}_dirty", name);
            let observer_name = format!("__dirty_{}", name);
            quote! {
                {
                    let __dirty = self.#dirty_field.clone();
                    self.#name.add_observer(
                        #observer_name,
                        Box::new(crate::state_mgmt::Effect::new(move |_| __dirty.set(true)))
                    );
                }
            }
        })
        .collect();

    // Aggregate dirty: set form.dirty = true when any {field}_dirty fires true
    let wire_agg_dirty_stmts: Vec<TokenStream2> = if has_form {
        validated_fields
            .iter()
            .map(|(name, _, _)| {
                let dirty_field = format_ident!("{}_dirty", name);
                quote! {
                    {
                        let __agg = self.dirty.clone();
                        self.#dirty_field.add_observer(
                            "__agg_dirty",
                            Box::new(crate::state_mgmt::Effect::new(move |v: &bool| {
                                if *v { __agg.set(true); }
                            }))
                        );
                    }
                }
            })
            .collect()
    } else {
        vec![]
    };

    // Aggregate touched: set form.touched = true when any {field}_touched fires true
    let wire_agg_touched_stmts: Vec<TokenStream2> = if has_form {
        validated_fields
            .iter()
            .map(|(name, _, _)| {
                let touched_field = format_ident!("{}_touched", name);
                quote! {
                    {
                        let __agg = self.touched.clone();
                        self.#touched_field.add_observer(
                            "__agg_touched",
                            Box::new(crate::state_mgmt::Effect::new(move |v: &bool| {
                                if *v { __agg.set(true); }
                            }))
                        );
                    }
                }
            })
            .collect()
    } else {
        vec![]
    };

    // Aggregate valid: valid = touched && all error signals empty.
    // Re-evaluated whenever self.touched or any {field}_error changes.
    // Each observer block gets its own set of signal clones.
    let wire_agg_valid_stmts: Vec<TokenStream2> = if has_form {
        let error_field_idents: Vec<Ident> = validated_fields
            .iter()
            .map(|(name, _, _)| format_ident!("{}_error", name))
            .collect();

        // Helper: build a block that adds an observer on `source` which recomputes valid.
        // clone_name_prefix is used to avoid shadowing across blocks (each block has its own clones).
        let make_valid_observer =
            |source_signal: TokenStream2, cb_arg_type: TokenStream2| -> TokenStream2 {
                let clone_lets: Vec<TokenStream2> = error_field_idents
                    .iter()
                    .enumerate()
                    .map(|(i, ef)| {
                        let cname = format_ident!("__ve{}", i);
                        quote! { let #cname = self.#ef.clone(); }
                    })
                    .collect();
                let all_empty: TokenStream2 = (0..error_field_idents.len())
                    .map(|i| {
                        let cname = format_ident!("__ve{}", i);
                        quote! { #cname.read().is_empty() }
                    })
                    .reduce(|a, b| quote! { #a && #b })
                    .unwrap_or(quote! { true });
                quote! {
                    {
                        let __fv = self.valid.clone();
                        let __ft = self.touched.clone();
                        #( #clone_lets )*
                        #source_signal.add_observer("__agg_valid", Box::new(
                            crate::state_mgmt::Effect::new(move |_: &#cb_arg_type| {
                                if __ft.read() { __fv.set(#all_empty); }
                            })
                        ));
                    }
                }
            };

        let mut stmts = vec![make_valid_observer(
            quote! { self.touched },
            quote! { bool },
        )];
        for ef in &error_field_idents {
            stmts.push(make_valid_observer(quote! { self.#ef }, quote! { String }));
        }
        stmts
    } else {
        vec![]
    };

    let touch_on_validate = if has_form {
        quote! { self.touched.set(true); }
    } else {
        quote! {}
    };

    let validation_impl = quote! {
        impl #struct_name {
            // Mark form as fully touched before running validators so that self.valid
            // reflects the true state after validate_all() (not blocked by untouched gate).
            pub fn validate_all(&mut self) -> bool {
                #touch_on_validate
                let mut __valid = true;
                #( #validate_all_calls )*
                __valid
            }

            #( #validate_field_methods )*

            pub fn __wire_form_validation(&self) {
                #( #wire_field_dirty_stmts )*
                #( #wire_agg_dirty_stmts )*
                #( #wire_agg_touched_stmts )*
                #( #wire_agg_valid_stmts )*
            }
        }
    };

    quote! {
        pub struct #struct_name {
            #( #struct_field_decls )*
        }

        impl Clone for #struct_name {
            fn clone(&self) -> Self {
                #struct_name { #( #clone_exprs )* }
            }
        }

        impl crate::state_mgmt::BrickModel for #struct_name {
            fn wire_cascade(&mut self) {
                #( #wire_stmts )*
            }
        }

        #cascade_impl

        #validation_impl
    }
}
