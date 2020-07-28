use proc_macro::{Literal, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
// use syn::proc_macro2::Literal;
use syn::{parse_macro_input, Result};

// struct MyMacroInput {
//     /* ... */
//     value: String,
// }

// impl Parse for MyMacroInput {
//     fn parse(input: ParseStream) -> Result<Self> {
//         /* ... */
//         Ok(MyMacroInput {
//             value: syn::parse(input)?,
//         })
//     }
// }

#[proc_macro]
pub fn my_macro(tokens: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input: Literal = syn::parse(tokens).unwrap();

    // let func_name = format_ident!("{}", input);
    let func_name = syn::Ident::new(TokenTree::from(&input), tokens.span());

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        pub fn #func_name<T: NodeWrap>(inner: T) -> Node {
            Node::new(#input, inner)
        }
        // ...
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
