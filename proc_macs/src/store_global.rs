use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Expr, ItemStruct, Type};

use crate::shared::{FieldKind, is_list};

pub fn expand_store(_attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
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

        let default_pos = field.attrs.iter().position(|a| a.path().is_ident("default"));

        let kind = if let Some(pos) = default_pos {
            let attr = field.attrs.remove(pos);
            match attr.parse_args::<Expr>() {
                Ok(val) => FieldKind::Default(val),
                Err(e) => return e.to_compile_error(),
            }
        } else if is_list(&ty) {
            FieldKind::List
        } else {
            FieldKind::Primary
        };

        fields.push(FieldInfo { name, ty, kind });
    }

    let struct_field_decls: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            match &f.kind {
                FieldKind::List => quote! { pub #name: std::rc::Rc<std::cell::RefCell<#ty>>, },
                _ => quote! { pub #name: crate::state_mgmt::Signal<#ty>, },
            }
        })
        .collect();

    let clone_exprs: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            match &f.kind {
                FieldKind::List => quote! { #name: std::rc::Rc::clone(&self.#name), },
                _ => quote! { #name: self.#name.clone(), },
            }
        })
        .collect();

    let new_inits: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            match &f.kind {
                FieldKind::List => quote! {
                    #name: std::rc::Rc::new(std::cell::RefCell::new(<#ty>::new())),
                },
                FieldKind::Default(val) => quote! {
                    #name: crate::state_mgmt::Signal::new(#val),
                },
                _ => quote! {
                    #name: crate::state_mgmt::Signal::placeholder(),
                },
            }
        })
        .collect();

    quote! {
        pub struct #struct_name {
            #( #struct_field_decls )*
        }

        impl Clone for #struct_name {
            fn clone(&self) -> Self {
                #struct_name { #( #clone_exprs )* }
            }
        }

        impl #struct_name {
            pub fn new() -> Self {
                #struct_name { #( #new_inits )* }
            }
        }
    }
}

pub fn expand_global(_attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let mut item: ItemStruct = match syn::parse2(item) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let struct_name = item.ident.clone();
    let tls_name = format_ident!("__BRICK_GLOBAL_{}", struct_name.to_string().to_uppercase());

    struct FieldInfo {
        name: Ident,
        ty: Type,
        kind: FieldKind,
    }

    let mut fields: Vec<FieldInfo> = Vec::new();

    for field in item.fields.iter_mut() {
        let name = field.ident.clone().unwrap();
        let ty = field.ty.clone();

        let default_pos = field.attrs.iter().position(|a| a.path().is_ident("default"));

        let kind = if let Some(pos) = default_pos {
            let attr = field.attrs.remove(pos);
            match attr.parse_args::<Expr>() {
                Ok(val) => FieldKind::Default(val),
                Err(e) => return e.to_compile_error(),
            }
        } else if is_list(&ty) {
            FieldKind::List
        } else {
            FieldKind::Primary
        };

        fields.push(FieldInfo { name, ty, kind });
    }

    let tls_inits: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            match &f.kind {
                FieldKind::List => quote! {
                    #name: std::rc::Rc::new(std::cell::RefCell::new(<#ty>::new()))
                },
                FieldKind::Default(val) => quote! {
                    #name: crate::state_mgmt::Signal::new(#val)
                },
                _ => quote! {
                    #name: crate::state_mgmt::Signal::placeholder()
                },
            }
        })
        .collect();

    let struct_field_decls: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            match &f.kind {
                FieldKind::List => quote! { pub #name: std::rc::Rc<std::cell::RefCell<#ty>>, },
                _ => quote! { pub #name: crate::state_mgmt::Signal<#ty>, },
            }
        })
        .collect();

    let clone_exprs: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            match &f.kind {
                FieldKind::List => quote! { #name: std::rc::Rc::clone(&self.#name), },
                _ => quote! { #name: self.#name.clone(), },
            }
        })
        .collect();

    let get_exprs: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let name = &f.name;
            match &f.kind {
                FieldKind::List => quote! {
                    #name: #tls_name.with(|s| std::rc::Rc::clone(&s.#name)),
                },
                _ => quote! {
                    #name: #tls_name.with(|s| s.#name.clone()),
                },
            }
        })
        .collect();

    // Static setters for each non-List Signal field.
    let setters: Vec<TokenStream2> = fields
        .iter()
        .filter(|f| !matches!(f.kind, FieldKind::List))
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            let setter_name = format_ident!("set_{}", name);
            quote! {
                pub fn #setter_name(value: #ty) {
                    Self::get().#name.set(value);
                }
            }
        })
        .collect();

    quote! {
        pub struct #struct_name {
            #( #struct_field_decls )*
        }

        impl Clone for #struct_name {
            fn clone(&self) -> Self {
                #struct_name { #( #clone_exprs )* }
            }
        }

        thread_local! {
            static #tls_name: #struct_name = #struct_name {
                #( #tls_inits ),*
            };
        }

        impl #struct_name {
            pub fn get() -> #struct_name {
                #struct_name { #( #get_exprs )* }
            }

            #( #setters )*
        }
    }
}
