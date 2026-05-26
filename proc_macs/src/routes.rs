use proc_macro2::{Ident as Ident2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Ident, LitStr, Type, parse::Parse, parse::ParseStream};

pub struct RoutesAttr {
    pub parent_enum: Option<Ident>,
    pub parent_variant: Option<Ident>,
}

impl Parse for RoutesAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(RoutesAttr {
                parent_enum: None,
                parent_variant: None,
            });
        }
        // Parse `ParentEnum::Variant` — two path segments joined by `::`
        let path: syn::Path = input.parse()?;
        let mut segs = path.segments.into_iter();
        let parent_enum = segs.next().map(|s| s.ident);
        let parent_variant = segs.next().map(|s| s.ident);
        Ok(RoutesAttr {
            parent_enum,
            parent_variant,
        })
    }
}

pub enum RouteVariantKind {
    Unit,
    Nested(Type),
    Params(Vec<(Ident, Type)>),
}

pub struct RouteVariant {
    pub name: Ident,
    pub path: String,
    pub kind: RouteVariantKind,
}

pub fn builder_struct_name(enum_name: &Ident, variant_name: &Ident) -> Ident2 {
    format_ident!("__{}{}Builder", enum_name, variant_name)
}

pub fn show_builder_struct_name(enum_name: &Ident, variant_name: &Ident) -> Ident2 {
    format_ident!("__{}{}ShowBuilder", enum_name, variant_name)
}

/// ZST name for a nested unit route, e.g. `__AppRouteUsersListRoute`.
pub fn unit_nested_struct_name(pen: &Ident, pvar: &Ident, vname: &Ident) -> Ident2 {
    format_ident!("__{}{}{}Route", pen, pvar, vname)
}

pub fn snake(ident: &Ident) -> Ident2 {
    let s = ident.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    format_ident!("{}", out)
}

/// Parse a `#[path("...")]` attribute's string value.
pub fn extract_path_attr(attrs: &mut Vec<syn::Attribute>) -> Option<String> {
    let pos = attrs.iter().position(|a| a.path().is_ident("path"))?;
    let attr = attrs.remove(pos);
    let lit: LitStr = attr.parse_args().ok()?;
    Some(lit.value())
}

pub enum PathSeg {
    Literal(String),
    /// A parameter, optionally preceded by a literal prefix (e.g. `@` in `/@:username`).
    Param {
        prefix: String,
        name: String,
    },
}

/// Split a path pattern into typed segments. Handles plain params (`:id`),
/// prefixed params (`@:username`), and literals (`edit`).
pub fn parse_path_segments(pattern: &str) -> Vec<PathSeg> {
    pattern
        .trim_start_matches('/')
        .split('/')
        .map(|seg| {
            if let Some(colon) = seg.find(':') {
                PathSeg::Param {
                    prefix: seg[..colon].to_string(),
                    name: seg[colon + 1..].to_string(),
                }
            } else {
                PathSeg::Literal(seg.to_string())
            }
        })
        .collect()
}

pub fn expand(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let routes_attr: RoutesAttr = if attr.is_empty() {
        RoutesAttr {
            parent_enum: None,
            parent_variant: None,
        }
    } else {
        match syn::parse2(attr) {
            Ok(r) => r,
            Err(e) => return e.to_compile_error(),
        }
    };

    let mut item_enum: syn::ItemEnum = match syn::parse2(item) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };
    let enum_name = item_enum.ident.clone();
    let vis = item_enum.vis.clone();

    // Parse variants and strip #[path] attributes.
    let mut variants: Vec<RouteVariant> = Vec::new();
    for v in item_enum.variants.iter_mut() {
        let path =
            extract_path_attr(&mut v.attrs).unwrap_or_else(|| format!("/{}", snake(&v.ident)));
        let kind = match &v.fields {
            syn::Fields::Unit => RouteVariantKind::Unit,
            syn::Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                RouteVariantKind::Nested(f.unnamed.first().unwrap().ty.clone())
            }
            syn::Fields::Named(f) => {
                let fields = f
                    .named
                    .iter()
                    .map(|field| (field.ident.clone().unwrap(), field.ty.clone()))
                    .collect();
                RouteVariantKind::Params(fields)
            }
            _ => RouteVariantKind::Unit,
        };
        variants.push(RouteVariant {
            name: v.ident.clone(),
            path,
            kind,
        });
    }

    // Add NotFound variant to the enum.
    item_enum.variants.push(syn::parse_quote! { NotFound });
    // Add derives.
    item_enum
        .attrs
        .push(syn::parse_quote! { #[derive(Clone, PartialEq, Debug)] });

    // ── from_path arms ────────────────────────────────────────────────────────
    let from_path_arms: Vec<TokenStream2> = variants.iter().map(|v| {
        let vname = &v.name;
        let path = &v.path;
        match &v.kind {
            RouteVariantKind::Unit => {
                if path == "/" {
                    quote! { if path == "/" { return Self::#vname; } }
                } else {
                    let p = path.as_str();
                    quote! { if path == #p { return Self::#vname; } }
                }
            }
            RouteVariantKind::Nested(ty) => {
                let p = path.as_str();
                quote! {
                    if path == #p || path.starts_with(concat!(#p, "/")) {
                        let rest = &path[#p.len()..];
                        return Self::#vname(<#ty as crate::routing::Route>::from_path(rest));
                    }
                }
            }
            RouteVariantKind::Params(fields) => {
                let segs = parse_path_segments(path);
                let n = segs.len();
                let lit_checks: Vec<TokenStream2> = segs.iter().enumerate()
                    .filter_map(|(i, seg)| match seg {
                        PathSeg::Literal(lit) if !lit.is_empty() => {
                            Some(quote! { parts[#i] == #lit })
                        }
                        PathSeg::Param { prefix, .. } if !prefix.is_empty() => {
                            Some(quote! { parts[#i].starts_with(#prefix) })
                        }
                        _ => None,
                    })
                    .collect();
                let param_bindings: Vec<TokenStream2> = segs.iter().enumerate()
                    .filter_map(|(i, seg)| match seg {
                        PathSeg::Param { prefix, name } => {
                            let ident = format_ident!("{}", name);
                            if prefix.is_empty() {
                                Some(quote! { let #ident = parts[#i].to_string(); })
                            } else {
                                Some(quote! { let #ident = parts[#i].strip_prefix(#prefix).unwrap_or(parts[#i]).to_string(); })
                            }
                        }
                        _ => None,
                    })
                    .collect();
                let field_names: Vec<Ident2> = fields.iter().map(|(f, _)| f.clone()).collect();
                let checks = if lit_checks.is_empty() {
                    quote! { true }
                } else {
                    quote! { #( #lit_checks )&&* }
                };
                quote! {
                    {
                        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
                        if parts.len() == #n && #checks {
                            #( #param_bindings )*
                            return Self::#vname { #( #field_names ),* };
                        }
                    }
                }
            }
        }
    }).collect();

    // ── to_path arms ──────────────────────────────────────────────────────────
    let to_path_arms: Vec<TokenStream2> = variants.iter().map(|v| {
        let vname = &v.name;
        let path = &v.path;
        match &v.kind {
            RouteVariantKind::Unit => quote! {
                Self::#vname => #path.to_string(),
            },
            RouteVariantKind::Nested(_) => {
                let p = path.as_str();
                quote! {
                    Self::#vname(child) => {
                        let combined = format!("{}{}", #p, child.to_path());
                        if combined.len() > 1 {
                            combined.trim_end_matches('/').to_string()
                        } else {
                            combined
                        }
                    },
                }
            }
            RouteVariantKind::Params(fields) => {
                let segs = parse_path_segments(path);
                let fmt: String = segs.iter().map(|s| match s {
                    PathSeg::Literal(lit) => lit.clone(),
                    PathSeg::Param { prefix, .. } => format!("{}{{}}", prefix),
                }).collect::<Vec<_>>().join("/");
                let fmt = format!("/{}", fmt);
                let param_idents: Vec<Ident2> = fields.iter().map(|(f, _)| f.clone()).collect();
                quote! {
                    Self::#vname { #( #param_idents ),* } => format!(#fmt, #( #param_idents ),*),
                }
            }
        }
    }).collect();

    // ── Builder constants / methods ───────────────────────────────────────────
    let mut builder_consts: Vec<TokenStream2> = Vec::new();
    let mut extra_items: Vec<TokenStream2> = Vec::new();

    for v in &variants {
        let vname = &v.name;
        let method_name = snake(vname);
        match &v.kind {
            RouteVariantKind::Unit => {
                builder_consts.push(quote! {
                    #[allow(non_upper_case_globals)]
                    pub const #method_name: Self = Self::#vname;
                });
            }
            RouteVariantKind::Nested(_) => {
                let builder = builder_struct_name(&enum_name, vname);
                let path_str = v.path.as_str();
                builder_consts.push(quote! {
                    #[allow(non_upper_case_globals)]
                    pub const #method_name: #builder = #builder::__init();
                });
                extra_items.push(quote! {
                    impl crate::routing::Linkable for #builder {
                        fn to_nav_path(&self) -> String { #path_str.to_string() }
                    }
                    impl crate::view_components::IntoComponent for Box<#builder> {
                        fn into_component(self) -> Box<dyn crate::view_components::Brick> {
                            Box::new(crate::routing::NavLink {
                                path: #path_str.to_string(),
                                children: vec![],
                            })
                        }
                    }
                });
            }
            RouteVariantKind::Params(fields) => {
                let param_args2: Vec<TokenStream2> = fields
                    .iter()
                    .map(|(f, t)| {
                        let ts = quote!(#t).to_string();
                        if ts.contains("String") {
                            quote! { #f: impl ::std::string::ToString }
                        } else {
                            quote! { #f: #t }
                        }
                    })
                    .collect();
                let param_inits: Vec<TokenStream2> = fields
                    .iter()
                    .map(|(f, t)| {
                        let ts = quote!(#t).to_string();
                        if ts.contains("String") {
                            quote! { #f: #f.to_string() }
                        } else {
                            quote! { #f }
                        }
                    })
                    .collect();
                builder_consts.push(quote! {
                    pub fn #method_name( #( #param_args2 ),* ) -> Self {
                        Self::#vname { #( #param_inits ),* }
                    }
                });
            }
        }
    }

    // ── Builder struct + impl (generated by child #[routes(parent_enum=..., parent_variant=...)]) ─
    let parent_impl_block: Option<TokenStream2> = routes_attr
        .parent_enum
        .as_ref()
        .zip(routes_attr.parent_variant.as_ref())
        .map(|(pen, pvar)| {
            let parent_builder = builder_struct_name(pen, pvar);
            let mut methods: Vec<TokenStream2> = Vec::new();
            let mut unit_field_decls: Vec<TokenStream2> = Vec::new();
            let mut unit_field_inits: Vec<TokenStream2> = Vec::new();
            let mut unit_zst_items: Vec<TokenStream2> = Vec::new();
            let mut show_builder_items: Vec<TokenStream2> = Vec::new();

            for v in &variants {
                let vname = &v.name;
                let method_name = snake(vname);
                match &v.kind {
                    RouteVariantKind::Unit => {
                        let zst = unit_nested_struct_name(pen, pvar, vname);
                        let full_route = quote! { #pen::#pvar(#enum_name::#vname) };
                        unit_zst_items.push(quote! {
                            #[derive(Clone, Copy, Debug, PartialEq)]
                            #vis struct #zst;
                            impl crate::routing::Linkable for #zst {
                                fn to_nav_path(&self) -> String {
                                    use crate::routing::Route as _;
                                    #full_route.to_path()
                                }
                            }
                            impl crate::view_components::IntoComponent for Box<#zst> {
                                fn into_component(self) -> Box<dyn crate::view_components::Brick> {
                                    Box::new(crate::routing::NavLink {
                                        path: crate::routing::Linkable::to_nav_path(self.as_ref()),
                                        children: vec![],
                                    })
                                }
                            }
                        });
                        unit_field_decls.push(quote! { pub #method_name: #zst, });
                        unit_field_inits.push(quote! { #method_name: #zst, });
                    }
                    RouteVariantKind::Params(fields) => {
                        let param_args: Vec<TokenStream2> = fields
                            .iter()
                            .map(|(f, t)| {
                                let ts = quote!(#t).to_string();
                                if ts.contains("String") {
                                    quote! { #f: impl ::std::string::ToString }
                                } else {
                                    quote! { #f: #t }
                                }
                            })
                            .collect();
                        let param_inits: Vec<TokenStream2> = fields
                            .iter()
                            .map(|(f, t)| {
                                let ts = quote!(#t).to_string();
                                if ts.contains("String") {
                                    quote! { #f: #f.to_string() }
                                } else {
                                    quote! { #f }
                                }
                            })
                            .collect();

                        let show_builder = show_builder_struct_name(pen, vname);
                        show_builder_items.push(quote! {
                            #[derive(Clone)]
                            #vis struct #show_builder {
                                pub _route: #pen,
                            }
                            impl crate::routing::Linkable for #show_builder {
                                fn to_nav_path(&self) -> String {
                                    use crate::routing::Route as _;
                                    self._route.to_path()
                                }
                            }
                        });

                        methods.push(quote! {
                            pub fn #method_name(self, #( #param_args ),*) -> #show_builder {
                                #show_builder {
                                    _route: #pen::#pvar(#enum_name::#vname { #( #param_inits ),* }),
                                }
                            }
                        });
                    }
                    RouteVariantKind::Nested(_) => {}
                }
            }

            quote! {
                #( #unit_zst_items )*

                #[derive(Clone, Copy)]
                #vis struct #parent_builder {
                    #( #unit_field_decls )*
                }
                impl #parent_builder {
                    #[doc(hidden)]
                    pub const fn __init() -> Self {
                        Self { #( #unit_field_inits )* }
                    }
                    #( #methods )*
                }

                #( #show_builder_items )*
            }
        });

    // ── Router signal + navigate() (top-level enums only) ────────────────────
    let signal_name = format_ident!("__{}Signal", enum_name);
    let accessor_name = format_ident!("{}", snake(&enum_name));
    let guards_name = format_ident!("__{}_guards", snake(&enum_name));
    let navigate_fn = quote! {
        #[allow(non_snake_case)]
        fn #signal_name() -> crate::state_mgmt::Signal<#enum_name> {
            use std::cell::OnceCell;
            thread_local! {
                static CELL: OnceCell<crate::state_mgmt::Signal<#enum_name>> = OnceCell::new();
            }
            CELL.with(|cell| cell.get_or_init(|| {
                let initial = {
                    #[cfg(not(test))]
                    {
                        let path = web_sys::window()
                            .unwrap()
                            .location()
                            .pathname()
                            .unwrap_or_default();
                        #enum_name::from_path(&path)
                    }
                    #[cfg(test)]
                    { #enum_name::NotFound }
                };
                let sig = crate::state_mgmt::Signal::new(initial);
                #[cfg(not(test))]
                {
                    use wasm_bindgen::{JsCast, closure::Closure};
                    let sig2 = sig.clone();
                    let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
                        let path = web_sys::window()
                            .unwrap()
                            .location()
                            .pathname()
                            .unwrap_or_default();
                        sig2.set(#enum_name::from_path(&path));
                    }) as Box<dyn FnMut(web_sys::Event)>);
                    web_sys::window()
                        .unwrap()
                        .add_event_listener_with_callback("popstate", cb.as_ref().unchecked_ref())
                        .unwrap();
                    cb.forget();
                }
                sig
            }).clone())
        }

        thread_local! {
            #[allow(non_upper_case_globals)]
            static #guards_name: std::cell::RefCell<Vec<Box<dyn Fn(#enum_name) -> Option<#enum_name>>>>
                = std::cell::RefCell::new(Vec::new());
        }

        impl #enum_name {
            /// Register a navigation guard. Return `None` to allow, `Some(redirect)` to redirect.
            pub fn guard(f: impl Fn(#enum_name) -> Option<#enum_name> + 'static) {
                #guards_name.with(|g| g.borrow_mut().push(Box::new(f)));
            }
        }

        /// Returns the reactive signal for the current route.
        #vis fn #accessor_name() -> crate::state_mgmt::Signal<#enum_name> {
            #signal_name()
        }

        /// Navigate to a route, running guards first, then updating the URL and the router signal.
        #vis fn navigate(route: #enum_name) {
            use crate::routing::Route as _;
            let final_route = #guards_name.with(|guards| {
                let mut r = route;
                for guard in guards.borrow().iter() {
                    if let Some(redirect) = guard(r.clone()) {
                        r = redirect;
                    }
                }
                r
            });
            let path = final_route.to_path();
            crate::routing::navigate_to_path(&path);
            #signal_name().set(final_route);
        }
    };

    let maybe_navigate = if routes_attr.parent_enum.is_none() {
        navigate_fn
    } else {
        quote! {}
    };

    quote! {
        #item_enum

        impl crate::routing::Route for #enum_name {
            fn from_path(path: &str) -> Self {
                let path = if path.is_empty() { "/" } else { path };
                let path = if path.len() > 1 { path.trim_end_matches('/') } else { path };
                #( #from_path_arms )*
                Self::NotFound
            }
            fn to_path(&self) -> String {
                match self {
                    #( #to_path_arms )*
                    Self::NotFound => "/404".to_string(),
                }
            }
        }

        impl crate::routing::Linkable for #enum_name {
            fn to_nav_path(&self) -> String {
                use crate::routing::Route as _;
                self.to_path()
            }
        }

        impl #enum_name {
            #( #builder_consts )*
        }

        #( #extra_items )*

        #maybe_navigate

        #parent_impl_block
    }
}
