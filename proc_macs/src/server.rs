use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, LitStr, Pat, ReturnType, Token, Type, parse::Parse, parse::ParseStream,
};

// ─── Attribute parsing ────────────────────────────────────────────────────────

enum Encoding {
    Prost,
    Json,
}

/// Parsed contents of `#[server]`, `#[server(path = "/foo")]`, or
/// `#[server(path = "/foo", encoding = "json")]`.
struct ServerAttr {
    path: Option<LitStr>,
    encoding: Encoding,
}

impl Parse for ServerAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path: Option<LitStr> = None;
        let mut encoding = Encoding::Prost;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val: LitStr = input.parse()?;
            match key.to_string().as_str() {
                "path" => path = Some(val),
                "encoding" => {
                    encoding = match val.value().as_str() {
                        "prost" => Encoding::Prost,
                        "json" => Encoding::Json,
                        other => {
                            return Err(syn::Error::new(
                                val.span(),
                                format!(
                                    "unknown encoding '{}'; expected \"prost\" or \"json\"",
                                    other
                                ),
                            ));
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown #[server] attribute key: {}", other),
                    ));
                }
            }
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(ServerAttr { path, encoding })
    }
}

// ─── Macro entry point ────────────────────────────────────────────────────────

/// Expand the `#[server]` attribute macro.
///
/// Emits a `#[cfg(not(brick_dom))]` server branch (real body + inventory
/// registration) and a `#[cfg(brick_dom)]` client branch (fetch stub).
pub fn expand(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let func: ItemFn = match syn::parse2(item.clone()) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let server_attr: ServerAttr = match syn::parse2(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error(),
    };

    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();

    let path_str: String = match &server_attr.path {
        Some(lit) => lit.value(),
        None => format!("/brick/fn/{}", fn_name_str),
    };

    let params: Vec<(Ident, Type)> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some((pat_ident.ident.clone(), (*pat_type.ty).clone()));
                }
            }
            None
        })
        .collect();

    let ok_type: TokenStream2 = match &func.sig.output {
        ReturnType::Type(_, ty) => {
            if let Type::Path(tp) = &**ty {
                if let Some(seg) = tp.path.segments.last() {
                    if seg.ident == "Result" {
                        if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                            if let Some(syn::GenericArgument::Type(ok_ty)) = ab.args.first() {
                                quote! { #ok_ty }
                            } else {
                                quote! { () }
                            }
                        } else {
                            quote! { () }
                        }
                    } else {
                        quote! { #ty }
                    }
                } else {
                    quote! { #ty }
                }
            } else {
                quote! { #ty }
            }
        }
        ReturnType::Default => quote! { () },
    };

    match server_attr.encoding {
        Encoding::Prost => expand_prost(func, params, ok_type, fn_name_str, path_str),
        Encoding::Json => expand_json(func, params, ok_type, fn_name_str, path_str),
    }
}

// ─── Prost encoding ───────────────────────────────────────────────────────────
//
// Wire format: prost-encoded protobuf. Requires:
//   - 0 or 1 arguments (2+ → compile error; wrap them in a #[derive(prost::Message)] struct)
//   - Return type T must impl prost::Message + Default
//   - Argument type (if present) must impl prost::Message + Default

fn expand_prost(
    func: ItemFn,
    params: Vec<(Ident, Type)>,
    ok_type: TokenStream2,
    fn_name_str: String,
    path_str: String,
) -> TokenStream2 {
    if params.len() > 1 {
        return syn::Error::new(
            func.sig.ident.span(),
            "prost encoding requires 0 or 1 arguments; wrap multiple args in a \
             #[derive(prost::Message)] request struct, or use #[server(encoding = \"json\")]",
        )
        .to_compile_error();
    }

    let fn_name = &func.sig.ident;
    let vis = &func.vis;
    let sig = &func.sig;

    // ── Server handler body ───────────────────────────────────────────────────

    let handler_body: TokenStream2 = if let Some((arg_name, arg_ty)) = params.first() {
        quote! {
            let #arg_name = <#arg_ty as brick::prost::Message>::decode(
                __body.as_slice()
            ).map_err(|e| brick::ssr::BrickError::bad_request(e.to_string()))?;
            let __result = #fn_name(#arg_name).await?;
            let __out = { use brick::prost::Message as _; __result.encode_to_vec() };
            Ok(__out)
        }
    } else {
        quote! {
            let _ = __body;
            let __result = #fn_name().await?;
            let __out = { use brick::prost::Message as _; __result.encode_to_vec() };
            Ok(__out)
        }
    };

    let server_branch = quote! {
        #func

        brick::inventory::submit! {
            brick::ssr::ServerFnEntry {
                name: #fn_name_str,
                path: #path_str,
                handler: |__body: Vec<u8>| Box::pin(async move {
                    #handler_body
                }),
            }
        }
    };

    // ── Client stub ───────────────────────────────────────────────────────────

    let client_body: TokenStream2 = if let Some((arg_name, _)) = params.first() {
        quote! {
            let __bytes = { use brick::prost::Message as _; #arg_name.encode_to_vec() };
            let __resp_bytes = brick::browser::fetch::fetch_bytes(#path_str, __bytes).await?;
            <#ok_type as brick::prost::Message>::decode(__resp_bytes.as_slice())
                .map_err(|e| brick::ssr::BrickError::internal(e.to_string()))
        }
    } else {
        quote! {
            let __resp_bytes = brick::browser::fetch::fetch_bytes(#path_str, vec![]).await?;
            <#ok_type as brick::prost::Message>::decode(__resp_bytes.as_slice())
                .map_err(|e| brick::ssr::BrickError::internal(e.to_string()))
        }
    };

    let client_branch = quote! {
        #vis #sig {
            #client_body
        }
    };

    quote! {
        #[cfg(not(brick_dom))]
        #server_branch

        #[cfg(brick_dom)]
        #client_branch
    }
}

// ─── JSON encoding ────────────────────────────────────────────────────────────
//
// Wire format: serde_json. All arguments and the return type must impl
// serde::Serialize + serde::de::DeserializeOwned. Multiple arguments are
// supported; they are bundled as a JSON object keyed by parameter name.

fn expand_json(
    func: ItemFn,
    params: Vec<(Ident, Type)>,
    ok_type: TokenStream2,
    fn_name_str: String,
    path_str: String,
) -> TokenStream2 {
    let fn_name = &func.sig.ident;
    let vis = &func.vis;
    let sig = &func.sig;

    let param_names: Vec<&Ident> = params.iter().map(|(n, _)| n).collect();
    let param_types: Vec<&Type> = params.iter().map(|(_, t)| t).collect();
    let param_names_str: Vec<String> = param_names.iter().map(|n| n.to_string()).collect();

    let deserialize_args: Vec<TokenStream2> = param_names
        .iter()
        .zip(param_types.iter())
        .zip(param_names_str.iter())
        .map(|((name, ty), name_str)| {
            quote! {
                let #name: #ty = serde_json::from_value(__args[#name_str].clone())
                    .map_err(|e| brick::ssr::BrickError::bad_request(e.to_string()))?;
            }
        })
        .collect();

    let call_args = quote! { #( #param_names ),* };

    let server_branch = quote! {
        #func

        brick::inventory::submit! {
            brick::ssr::ServerFnEntry {
                name: #fn_name_str,
                path: #path_str,
                handler: |__body: Vec<u8>| Box::pin(async move {
                    let __args: serde_json::Value = serde_json::from_slice(&__body)
                        .map_err(|e| brick::ssr::BrickError::bad_request(e.to_string()))?;
                    #( #deserialize_args )*
                    let __result = #fn_name(#call_args).await?;
                    serde_json::to_vec(&__result)
                        .map_err(|e| brick::ssr::BrickError::internal(e.to_string()))
                }),
            }
        }
    };

    let build_body_fields: Vec<TokenStream2> = param_names
        .iter()
        .zip(param_names_str.iter())
        .map(|(name, name_str)| quote! { #name_str: #name })
        .collect();

    let client_branch = quote! {
        #vis #sig {
            let __body = serde_json::json!({ #( #build_body_fields ),* });
            let __bytes = serde_json::to_vec(&__body)
                .map_err(|e| brick::ssr::BrickError::internal(e.to_string()))?;
            let __resp_bytes = brick::browser::fetch::fetch_bytes(#path_str, __bytes)
                .await
                .map_err(brick::ssr::BrickError::internal)?;
            serde_json::from_slice::<#ok_type>(&__resp_bytes)
                .map_err(|e| brick::ssr::BrickError::internal(e.to_string()))
        }
    };

    quote! {
        #[cfg(not(brick_dom))]
        #server_branch

        #[cfg(brick_dom)]
        #client_branch
    }
}
