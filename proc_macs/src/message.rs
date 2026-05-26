use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

// ─── FNV-32 hash for stable field tags ───────────────────────────────────────

fn fnv32(s: &str) -> u32 {
    let mut hash: u32 = 2_166_136_261; // FNV-1 32-bit offset basis
    for byte in s.bytes() {
        hash = hash.wrapping_mul(16_777_619); // FNV prime
        hash ^= byte as u32;
    }
    hash
}

/// Map a field name to a stable protobuf tag number.
///
/// Tag space: `fnv32(name) & 0x1FFF_FFFF`, clamped away from 0 and the
/// protobuf-reserved range 19000–19999.
fn stable_tag(field_name: &str) -> u32 {
    let tag = fnv32(field_name) & 0x1FFF_FFFF;
    let tag = if tag == 0 { 1 } else { tag };
    // Protobuf spec reserves 19000–19999 for internal use.
    if (19000..=19999).contains(&tag) { tag + 10_000 } else { tag }
}

// ─── Type → prost annotation ─────────────────────────────────────────────────

/// Return the inner type name of `Vec<T>` as a string, or `None` for
/// non-simple paths (qualified, generic Vec arg that is itself generic, etc.).
fn vec_inner_ident(args: &syn::PathArguments) -> Option<String> {
    if let syn::PathArguments::AngleBracketed(ab) = args {
        if let Some(syn::GenericArgument::Type(Type::Path(tp))) = ab.args.first() {
            let segs: Vec<_> = tp.path.segments.iter().collect();
            if segs.len() == 1 {
                return Some(segs[0].ident.to_string());
            }
        }
    }
    None
}

/// Produce the `#[prost(type, tag = "N")]` attribute for a given Rust type and
/// tag number.  Returns `Err` for unsupported types.
fn prost_attr(ty: &Type, tag: u32) -> Result<TokenStream2, syn::Error> {
    let tag_str = tag.to_string();
    match ty {
        Type::Path(tp) => {
            let last = tp.path.segments.last().unwrap();
            match last.ident.to_string().as_str() {
                "String" => Ok(quote! { #[prost(string,  tag = #tag_str)] }),
                "u32"    => Ok(quote! { #[prost(uint32,  tag = #tag_str)] }),
                "u64"    => Ok(quote! { #[prost(uint64,  tag = #tag_str)] }),
                "i32"    => Ok(quote! { #[prost(int32,   tag = #tag_str)] }),
                "i64"    => Ok(quote! { #[prost(int64,   tag = #tag_str)] }),
                "f32"    => Ok(quote! { #[prost(float,   tag = #tag_str)] }),
                "f64"    => Ok(quote! { #[prost(double,  tag = #tag_str)] }),
                "bool"   => Ok(quote! { #[prost(bool,    tag = #tag_str)] }),

                "Vec" => {
                    let inner = vec_inner_ident(&last.arguments).unwrap_or_default();
                    let attr = match inner.as_str() {
                        "u8"     => quote! { #[prost(bytes = "vec", tag = #tag_str)] },
                        "String" => quote! { #[prost(string,  repeated, tag = #tag_str)] },
                        "u32"    => quote! { #[prost(uint32,  repeated, tag = #tag_str)] },
                        "u64"    => quote! { #[prost(uint64,  repeated, tag = #tag_str)] },
                        "i32"    => quote! { #[prost(int32,   repeated, tag = #tag_str)] },
                        "i64"    => quote! { #[prost(int64,   repeated, tag = #tag_str)] },
                        "f32"    => quote! { #[prost(float,   repeated, tag = #tag_str)] },
                        "f64"    => quote! { #[prost(double,  repeated, tag = #tag_str)] },
                        "bool"   => quote! { #[prost(bool,    repeated, tag = #tag_str)] },
                        _        => quote! { #[prost(message,  repeated, tag = #tag_str)] },
                    };
                    Ok(attr)
                }

                "Option" => Ok(quote! { #[prost(message, optional, tag = #tag_str)] }),

                other => Err(syn::Error::new(
                    last.ident.span(),
                    format!(
                        "brick::Message: unsupported field type `{other}`; \
                         use a scalar (String, u32, u64, i32, i64, f32, f64, bool), \
                         Vec<u8>, Vec<T>, or Option<T>"
                    ),
                )),
            }
        }
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "brick::Message: unsupported field type; \
             only named path types are supported",
        )),
    }
}

// ─── Derive expansion ─────────────────────────────────────────────────────────

pub fn expand(input: TokenStream2) -> TokenStream2 {
    let input: DeriveInput = match syn::parse2(input) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error(),
    };

    if !input.generics.params.is_empty() {
        return syn::Error::new(
            input.ident.span(),
            "brick::Message: generic structs are not supported",
        )
        .to_compile_error();
    }

    let struct_name = &input.ident;
    let proxy_name = syn::Ident::new(
        &format!("_BrickProxy_{}", struct_name),
        struct_name.span(),
    );

    // Collect named fields (unit structs are fine; tuple structs are not).
    let named_fields: Vec<_> = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.named.iter().collect(),
            Fields::Unit => vec![],
            Fields::Unnamed(_) => {
                return syn::Error::new(
                    struct_name.span(),
                    "brick::Message: tuple structs are not supported",
                )
                .to_compile_error();
            }
        },
        _ => {
            return syn::Error::new(
                struct_name.span(),
                "brick::Message: only structs are supported",
            )
            .to_compile_error();
        }
    };

    // Build per-field info; check for tag collisions.
    let mut seen_tags: std::collections::HashMap<u32, String> = Default::default();
    let mut proxy_field_defs: Vec<TokenStream2> = Vec::new();
    let mut field_idents: Vec<syn::Ident> = Vec::new();
    let mut field_ident_strs: Vec<String> = Vec::new();

    for field in &named_fields {
        let name = field.ident.as_ref().unwrap();
        let name_str = name.to_string();
        let tag = stable_tag(&name_str);

        // Detect hash collisions at compile time.
        if let Some(prior) = seen_tags.get(&tag) {
            return syn::Error::new(
                name.span(),
                format!(
                    "brick::Message: field `{name_str}` collides with `{prior}` \
                     on hash-based tag {tag}; rename one field to resolve"
                ),
            )
            .to_compile_error();
        }
        seen_tags.insert(tag, name_str.clone());

        let attr = match prost_attr(&field.ty, tag) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error(),
        };
        let ty = &field.ty;
        proxy_field_defs.push(quote! { #attr pub #name: #ty });
        field_ident_strs.push(name_str);
        field_idents.push(name.clone());
    }

    // Proxy construction (self → proxy): clone every field.
    let proxy_ctor = if field_idents.is_empty() {
        quote! { #proxy_name {} }
    } else {
        quote! { #proxy_name { #( #field_idents: self.#field_idents.clone(), )* } }
    };

    // After merge_field: copy proxy fields back to self.
    let copy_back = quote! { #( self.#field_idents = proxy.#field_idents; )* };

    // Default impl: zero every field.
    let default_fields = if field_idents.is_empty() {
        quote! {}
    } else {
        quote! { #( #field_idents: ::core::default::Default::default(), )* }
    };

    // Debug impl: use debug_struct builder.
    let debug_fields = field_idents.iter().zip(field_ident_strs.iter()).map(|(ident, name_str)| {
        quote! { .field(#name_str, &self.#ident) }
    });

    // prost 0.13 uses `impl Trait` syntax in the trait definition; our impl
    // must match exactly (generic parameters are incompatible — E0643).
    quote! {
        const _: () = {
            // prost-derive generates Default and Debug for the proxy automatically;
            // we must NOT also derive them here or we get E0119 conflicts.
            #[allow(non_camel_case_types, dead_code)]
            #[derive(Clone, PartialEq, ::prost::Message)]
            struct #proxy_name {
                #( #proxy_field_defs, )*
            }

            impl ::prost::Message for #struct_name {
                fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut)
                where Self: ::core::marker::Sized
                {
                    let proxy = #proxy_ctor;
                    ::prost::Message::encode_raw(&proxy, buf);
                }

                fn merge_field(
                    &mut self,
                    tag: u32,
                    wire_type: ::prost::encoding::WireType,
                    buf: &mut impl ::prost::bytes::Buf,
                    ctx: ::prost::encoding::DecodeContext,
                ) -> ::core::result::Result<(), ::prost::DecodeError>
                where Self: ::core::marker::Sized
                {
                    let mut proxy = #proxy_ctor;
                    ::prost::Message::merge_field(&mut proxy, tag, wire_type, buf, ctx)?;
                    #copy_back
                    Ok(())
                }

                fn encoded_len(&self) -> usize {
                    let proxy = #proxy_ctor;
                    ::prost::Message::encoded_len(&proxy)
                }

                fn clear(&mut self) {
                    #( self.#field_idents = ::core::default::Default::default(); )*
                }
            }

            // prost::Message: Debug + Send + Sync — generate both automatically.
            impl ::core::fmt::Debug for #struct_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct(stringify!(#struct_name))
                        #( #debug_fields )*
                        .finish()
                }
            }

            impl ::core::default::Default for #struct_name {
                fn default() -> Self {
                    Self { #default_fields }
                }
            }
        };
    }
}
