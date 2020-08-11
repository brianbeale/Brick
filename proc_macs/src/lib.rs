use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ImplItem, ItemFn, ItemImpl, ItemStruct, Type};

#[proc_macro_attribute]
pub fn model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let struct_name = item.ident;
    let field_names: Vec<Ident> = item
        .fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .collect();
    let field_types: Vec<Type> = item.fields.iter().map(|x| x.ty.clone()).collect();

    let expanded = quote! {
        use crate::state_mgmt::Subject;
        use std::{rc::Rc, cell::RefCell};
        pub struct #struct_name {
            #(
                pub #field_names: Rc<RefCell<Box<dyn Subject<#field_types>>>>,
            )*
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_input = parse_macro_input!(item as ItemImpl);
    let model_name = *impl_input.self_ty.clone();

    let methods = impl_input.items.iter();
    let method_idents: Vec<Ident> = methods
        .filter_map(|x| {
            if let ImplItem::Method(meth) = x {
                Some(meth)
            } else {
                None
            }
        })
        .map(|x| x.sig.ident.clone())
        .collect();

    let expanded = quote! {
        #impl_input

        impl #model_name {
            pub fn controller_methods(self) -> (std::collections::HashMap<&'static str, (&'static str, wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>)>,
        std::rc::Rc<std::cell::RefCell<#model_name>>) {
                let mut event_listeners = std::collections::HashMap::new();
                let sharing_model = std::rc::Rc::new(std::cell::RefCell::new(self));
                #(
                    event_listeners.insert(stringify!(#method_idents), ("click", method!(#method_idents, std::rc::Rc::clone(&sharing_model))));
                )*
                (event_listeners, sharing_model)
            }

        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let prop_struct_name = item.ident;
    let model_name = Ident::new(
        &format!("{}Model", prop_struct_name),
        prop_struct_name.span(),
    );
    let prop_field_names: Vec<Ident> = item
        .fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .collect();

    let prop_field_types: Vec<Type> = item.fields.iter().map(|x| x.ty.clone()).collect();

    let additional = quote! {
        use crate::state_mgmt::{State, Subject};
        pub struct #model_name {
            #(
                pub #prop_field_names: State<#prop_field_types>,
            )*
        }
        impl #model_name {
            pub fn new(prop_struct: &#prop_struct_name) -> Self {
                #model_name {
                    #(
                        #prop_field_names: State::new(prop_struct.#prop_field_names),
                    )*
                }
            }
        }
        impl #prop_struct_name {
            pub fn make_model(&self) -> #model_name {
                #model_name::new(&self)
            }
        }
    };

    TokenStream::from(additional)
}

#[proc_macro_attribute]
pub fn view(attr: TokenStream, item: TokenStream) -> TokenStream {
    let struct_name = parse_macro_input!(attr as Ident);
    let func = parse_macro_input!(item as ItemFn);
    let func_block = func.block;
    let mut statements = func_block.stmts;
    let tail = statements.remove(statements.len() - 1);

    let output = quote! {
        impl Render for #struct_name {
            fn render(mut self) -> Box<ViewComposite> {
                // let props = std::rc::Rc::new(std::cell::RefCell::new(self));
                // let mut model = props.make_model();
                let (event_listeners, sharing_model) = self.controller_methods();
                #( #statements )*
                composite!(#struct_name, event_listeners, (sharing_model as Rc<RefCell<dyn std::any::Any>>), #tail)
            }
        }
    };
    TokenStream::from(output)
}

// Utililty
#[proc_macro_attribute]
pub fn print_item(_attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("item: \"{}\"", item.to_string());
    item
}
