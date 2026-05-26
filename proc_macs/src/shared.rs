use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Expr, LitInt, Token, Type, parse::Parse, parse::ParseStream};

pub fn is_list(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        tp.path
            .segments
            .last()
            .map(|seg| seg.ident == "List")
            .unwrap_or(false)
    } else {
        false
    }
}

pub fn is_resource(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        tp.path
            .segments
            .last()
            .map(|seg| seg.ident == "Resource")
            .unwrap_or(false)
    } else {
        false
    }
}

pub fn is_navigator(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        tp.path
            .segments
            .last()
            .map(|seg| seg.ident == "Navigator")
            .unwrap_or(false)
    } else {
        false
    }
}

/// True for types that have a meaningful `.len()` — String, str, Vec<T>.
/// Used to decide whether `min`/`max` should compare length instead of value.
pub fn type_has_len(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        let last = tp.path.segments.last().map(|s| s.ident.to_string());
        matches!(last.as_deref(), Some("String") | Some("str") | Some("Vec"))
    } else {
        false
    }
}

// ─── #[from] and #[default] attribute parsers ────────────────────────────────

pub struct FromAttr {
    pub source: Ident,
    pub func: Expr,
}

impl Parse for FromAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source: Ident = input.parse()?;
        let _: Token![,] = input.parse()?;
        let func: Expr = input.parse()?;
        Ok(FromAttr { source, func })
    }
}

pub enum FieldKind {
    Primary,
    Derived(FromAttr),
    Default(Expr),
    Validated { validators: Vec<ValidatorSpec> },
    Prop,
    Slot,
    List,
    Resource,                  // Resource<T> — no default; uses lazy placeholder
    ResourceWithDefault(Expr), // Resource<T> with #[default(expr)] — init expr used as-is
    Store,  // #[store] — plain field (Rc-handle struct), excluded from cascade_defaults
    Global, // #[global] — auto-filled via TypeName::get() in cascade_defaults
    Persist(String), // #[persist("localStorage-key")]
    Nav(Expr), // Navigator<S> with #[default(InitialScreen)] — plain field, not a Signal
}

// ─── #[validate] validator specs ─────────────────────────────────────────────

pub enum ValidatorSpec {
    Required(Option<String>),
    MinLength {
        n: usize,
        msg: Option<String>,
    },
    MaxLength {
        n: usize,
        msg: Option<String>,
    },
    Email(Option<String>),
    Url(Option<String>),
    Alpha(Option<String>),
    Alphanumeric(Option<String>),
    Min {
        tokens: TokenStream2,
        msg: Option<String>,
    },
    Max {
        tokens: TokenStream2,
        msg: Option<String>,
    },
    Positive(Option<String>),
    NonNegative(Option<String>),
    Custom(TokenStream2),
}

pub const BUILTIN_VALIDATORS: &[&str] = &[
    "required",
    "min_length",
    "max_length",
    "email",
    "url",
    "alpha",
    "alphanumeric",
    "min",
    "max",
    "positive",
    "non_negative",
];

/// Parse a `#[validate(...)]` attribute's inner token stream into a Vec<ValidatorSpec>.
pub fn parse_validators(ts: TokenStream2) -> Result<Vec<ValidatorSpec>, syn::Error> {
    use proc_macro2::TokenTree as TT;

    let items = split_by_top_comma(ts);
    let mut specs = Vec::new();

    for item in items {
        let tts: Vec<TT> = item.clone().into_iter().collect();
        if tts.is_empty() {
            continue;
        }

        // The first token is the validator name ident (or a custom expression path)
        let name = match &tts[0] {
            TT::Ident(id) => id.to_string(),
            _ => {
                // Treat whole item as custom validator expression
                specs.push(ValidatorSpec::Custom(item));
                continue;
            }
        };

        // Check if it's a known built-in
        if !BUILTIN_VALIDATORS.contains(&name.as_str()) {
            // Treat as custom validator expression
            specs.push(ValidatorSpec::Custom(item));
            continue;
        }

        // Parse optional (args) group
        let args_ts: Option<TokenStream2> = if tts.len() >= 2 {
            match &tts[1] {
                TT::Group(g) if g.delimiter() == proc_macro2::Delimiter::Parenthesis => {
                    Some(g.stream())
                }
                _ => None,
            }
        } else {
            None
        };

        // Helper: extract optional message string from last positional arg
        // e.g. min_length(3, "Too short") or min_length(3)
        fn extract_args_and_msg(args: TokenStream2) -> (Vec<TokenStream2>, Option<String>) {
            let parts = split_by_top_comma(args);
            let mut msg = None;
            let mut value_parts = parts.clone();
            if let Some(last) = parts.last() {
                let last_tts: Vec<proc_macro2::TokenTree> = last.clone().into_iter().collect();
                if last_tts.len() == 1 {
                    if let proc_macro2::TokenTree::Literal(lit) = &last_tts[0] {
                        let s = lit.to_string();
                        if s.starts_with('"') {
                            msg = Some(s.trim_matches('"').to_string());
                            value_parts.pop();
                        }
                    }
                }
            }
            (value_parts, msg)
        }

        let spec = match name.as_str() {
            "required" => {
                let msg = args_ts.and_then(|ts| {
                    let tts: Vec<_> = ts.into_iter().collect();
                    if tts.len() == 1 {
                        if let proc_macro2::TokenTree::Literal(lit) = &tts[0] {
                            let s = lit.to_string();
                            if s.starts_with('"') {
                                return Some(s.trim_matches('"').to_string());
                            }
                        }
                    }
                    None
                });
                ValidatorSpec::Required(msg)
            }
            "min_length" => {
                let (vals, msg) = extract_args_and_msg(args_ts.unwrap_or_default());
                let n: usize = vals
                    .first()
                    .and_then(|ts| {
                        let s = ts.to_string();
                        s.parse().ok()
                    })
                    .unwrap_or(0);
                ValidatorSpec::MinLength { n, msg }
            }
            "max_length" => {
                let (vals, msg) = extract_args_and_msg(args_ts.unwrap_or_default());
                let n: usize = vals
                    .first()
                    .and_then(|ts| {
                        let s = ts.to_string();
                        s.parse().ok()
                    })
                    .unwrap_or(usize::MAX);
                ValidatorSpec::MaxLength { n, msg }
            }
            "email" => {
                let msg = args_ts.and_then(|ts| {
                    let s = ts.to_string();
                    let s = s.trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                ValidatorSpec::Email(msg)
            }
            "url" => {
                let msg = args_ts.and_then(|ts| {
                    let s = ts.to_string();
                    let s = s.trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                ValidatorSpec::Url(msg)
            }
            "alpha" => {
                let msg = args_ts.and_then(|ts| {
                    let s = ts.to_string();
                    let s = s.trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                ValidatorSpec::Alpha(msg)
            }
            "alphanumeric" => {
                let msg = args_ts.and_then(|ts| {
                    let s = ts.to_string();
                    let s = s.trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                ValidatorSpec::Alphanumeric(msg)
            }
            "min" => {
                let (vals, msg) = extract_args_and_msg(args_ts.unwrap_or_default());
                let tokens = vals.into_iter().next().unwrap_or_default();
                ValidatorSpec::Min { tokens, msg }
            }
            "max" => {
                let (vals, msg) = extract_args_and_msg(args_ts.unwrap_or_default());
                let tokens = vals.into_iter().next().unwrap_or_default();
                ValidatorSpec::Max { tokens, msg }
            }
            "positive" => {
                let msg = args_ts.and_then(|ts| {
                    let s = ts.to_string();
                    let s = s.trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                ValidatorSpec::Positive(msg)
            }
            "non_negative" => {
                let msg = args_ts.and_then(|ts| {
                    let s = ts.to_string();
                    let s = s.trim_matches('"');
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                ValidatorSpec::NonNegative(msg)
            }
            _ => unreachable!(),
        };

        specs.push(spec);
    }

    Ok(specs)
}

/// Generate the body of a single `validate_{field}` method.
pub fn generate_validator_body(
    field_name: &Ident,
    field_type: &Type,
    validators: &[ValidatorSpec],
) -> TokenStream2 {
    let error_field = format_ident!("{}_error", field_name);
    let mut checks: Vec<TokenStream2> = Vec::new();

    for spec in validators {
        let check = match spec {
            ValidatorSpec::Required(msg) => {
                let m = msg.as_deref().unwrap_or("This field is required");
                quote! {
                    if !crate::validation::IsPresent::is_present(&val) {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::MinLength { n, msg } => {
                let m = msg.as_deref().unwrap_or("Too short");
                quote! {
                    if val.len() < #n {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::MaxLength { n, msg } => {
                let m = msg.as_deref().unwrap_or("Too long");
                quote! {
                    if val.len() > #n {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::Email(msg) => {
                let m = msg.as_deref().unwrap_or("Invalid email address");
                quote! {
                    if !(val.contains('@') && !val.contains(' ') && val.len() >= 3) {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::Url(msg) => {
                let m = msg.as_deref().unwrap_or("Invalid URL");
                quote! {
                    if !(val.starts_with("http://") || val.starts_with("https://")) {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::Alpha(msg) => {
                let m = msg
                    .as_deref()
                    .unwrap_or("Only alphabetic characters allowed");
                quote! {
                    if !crate::validation::IsAlpha::is_alpha(&val) {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::Alphanumeric(msg) => {
                let m = msg
                    .as_deref()
                    .unwrap_or("Only alphanumeric characters allowed");
                quote! {
                    if !crate::validation::IsAlpha::is_alphanumeric(&val) {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::Min { tokens, msg } => {
                if type_has_len(field_type) {
                    let m = msg.as_deref();
                    if let Some(m) = m {
                        quote! {
                            if val.len() < (#tokens) {
                                set!(self.#error_field => #m.to_string());
                                return false;
                            }
                        }
                    } else {
                        quote! {
                            if val.len() < (#tokens) {
                                set!(self.#error_field => format!("Must be at least {} characters", #tokens));
                                return false;
                            }
                        }
                    }
                } else {
                    let m = msg.as_deref().unwrap_or("Value too small");
                    quote! {
                        if val < (#tokens) {
                            set!(self.#error_field => #m.to_string());
                            return false;
                        }
                    }
                }
            }
            ValidatorSpec::Max { tokens, msg } => {
                if type_has_len(field_type) {
                    let m = msg.as_deref();
                    if let Some(m) = m {
                        quote! {
                            if val.len() > (#tokens) {
                                set!(self.#error_field => #m.to_string());
                                return false;
                            }
                        }
                    } else {
                        quote! {
                            if val.len() > (#tokens) {
                                set!(self.#error_field => format!("Must be no more than {} characters", #tokens));
                                return false;
                            }
                        }
                    }
                } else {
                    let m = msg.as_deref().unwrap_or("Value too large");
                    quote! {
                        if val > (#tokens) {
                            set!(self.#error_field => #m.to_string());
                            return false;
                        }
                    }
                }
            }
            ValidatorSpec::Positive(msg) => {
                let m = msg.as_deref().unwrap_or("Must be positive");
                quote! {
                    if !crate::validation::IsPositive::is_positive(&val) {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::NonNegative(msg) => {
                let m = msg.as_deref().unwrap_or("Must be non-negative");
                quote! {
                    if !crate::validation::IsPositive::is_non_negative(&val) {
                        set!(self.#error_field => #m.to_string());
                        return false;
                    }
                }
            }
            ValidatorSpec::Custom(expr_ts) => {
                quote! {
                    if let Err(__e) = crate::validation::Validate::validate(&(#expr_ts), &val) {
                        set!(self.#error_field => __e);
                        return false;
                    }
                }
            }
        };
        checks.push(check);
    }

    quote! {
        let val = self.#field_name.read();
        #( #checks )*
        set!(self.#error_field => String::new());
        true
    }
}

// ─── #[on] attribute parser ─────────────────────────────────────────────────

pub enum OnExtractor {
    Ident(Ident),
    Int(LitInt),
}

pub struct OnAttr {
    pub event_type: Ident,
    pub extractor: Option<OnExtractor>,
}

impl Parse for OnAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let event_type: Ident = input.parse()?;
        let extractor = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.peek(LitInt) {
                Some(OnExtractor::Int(input.parse::<LitInt>()?))
            } else {
                Some(OnExtractor::Ident(input.parse::<Ident>()?))
            }
        } else {
            None
        };
        Ok(OnAttr {
            event_type,
            extractor,
        })
    }
}

pub fn split_by_top_comma(ts: TokenStream2) -> Vec<TokenStream2> {
    use proc_macro2::TokenTree as TT;
    let mut items: Vec<TokenStream2> = vec![TokenStream2::new()];
    for tt in ts {
        if let TT::Punct(ref p) = tt {
            if p.as_char() == ',' {
                items.push(TokenStream2::new());
                continue;
            }
        }
        items.last_mut().unwrap().extend(std::iter::once(tt));
    }
    if items.last().map(|ts| ts.is_empty()).unwrap_or(false) {
        items.pop();
    }
    items
}
