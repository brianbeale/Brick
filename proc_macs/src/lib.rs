use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod controller;
mod css;
mod keyframes;
mod live;
mod message;
mod model;
mod routes;
mod server;
mod shared;
mod store_global;
mod view;

// ─── #[model] ────────────────────────────────────────────────────────────────

/// Derive reactive infrastructure for a model struct.
///
/// Every plain field becomes `Signal<T>`. Opt-in field annotations:
/// - `#[default(expr)]` — fills the field in `cascade()` with `expr`
/// - `#[from(source, fn)]` — derived field; recomputed whenever `source` changes
/// - `#[prop]` / `#[slot]` — passed through as-is (not wrapped in Signal)
/// - `#[store]` / `#[global]` — shared/singleton reactive state handles
/// - `#[validate(required, min_length(3), email, my_fn)]` — validation rules;
///   injects companion signals `{field}_error`, `{field}_touched`, `{field}_dirty`
///   and generates `validate_{field}()` and `validate_all()` methods
///
/// Generates `Clone`, `impl BrickModel` (with `wire_cascade`), and `impl Cascade`.
#[proc_macro_attribute]
pub fn model(attr: TokenStream, item: TokenStream) -> TokenStream {
    model::expand(attr.into(), item.into()).into()
}

// ─── #[controller] ───────────────────────────────────────────────────────────

/// Wire controller methods to DOM events and lifecycle hooks.
///
/// Applied to an `impl ModelName { ... }` block. Each method becomes a named
/// DOM event handler via its attribute:
/// - `#[on(click)]` / `#[on(input)]` / `#[on(change)]` etc. — DOM event listeners
/// - `#[on(submit)]` — click listener gated on `validate_all()` returning `true`
/// - `#[on(created)]` — runs before the first render (synchronous setup)
/// - `#[on(mount)]` — runs after the component's HTML is in the DOM
/// - `#[on(before_unmount)]` / `#[on(unmount)]` — teardown hooks
/// - `#[on(interval, ms)]` — `setInterval` callback at the given millisecond rate
/// - `#[watch(field)]` — fires whenever `field` signal changes
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller::expand(attr.into(), item.into()).into()
}

// ─── #[view] ─────────────────────────────────────────────────────────────────

/// Generate `impl Render for ModelName` from a plain function body.
///
/// The attribute argument is the model struct name. Inside the function body,
/// `my` is a cheap clone of the model. The last expression must produce
/// `Box<dyn ViewComponent>` (typically via `children!` or a single element).
///
/// Generates `impl Render for ModelName` and `impl IntoComponent for ModelName`.
#[proc_macro_attribute]
pub fn view(attr: TokenStream, item: TokenStream) -> TokenStream {
    view::expand(attr.into(), item.into()).into()
}

// ─── live! ───────────────────────────────────────────────────────────────────

/// Reactive template macro. Parses a format-like string where every {variable}
/// or {variable:spec} is a reactive binding.
#[proc_macro]
pub fn live(input: TokenStream) -> TokenStream {
    live::expand_live(TokenStream2::new(), input.into()).into()
}

// ─── compute! ────────────────────────────────────────────────────────────────

/// Derive a `Signal<T>` from an expression containing `my.field` references.
#[proc_macro]
pub fn compute(input: TokenStream) -> TokenStream {
    live::expand_compute(TokenStream2::new(), input.into()).into()
}

// ─── css! ────────────────────────────────────────────────────────────────────

/// Build a reactive CSS declarations string for use with `.style()`.
#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    css::expand_css(input)
}

// ─── style! ──────────────────────────────────────────────────────────────────

/// Inject scoped CSS rules into the page, supporting reactive `{expr:spec}` values.
#[proc_macro]
pub fn style(input: TokenStream) -> TokenStream {
    css::expand_style(input)
}

// ─── __brick_css_str ─────────────────────────────────────────────────────────

/// Internal helper: convert CSS-like tokens into a `&'static str` string literal.
#[proc_macro]
#[doc(hidden)]
pub fn __brick_css_str(input: TokenStream) -> TokenStream {
    css::expand_brick_css_str(input)
}

// ─── #[store] ────────────────────────────────────────────────────────────────

/// Instantiable reactive state struct — like `#[model]` without view or controller.
/// Generates Signal<T> fields, `Clone`, and `::new()`.
#[proc_macro_attribute]
pub fn store(attr: TokenStream, item: TokenStream) -> TokenStream {
    store_global::expand_store(attr.into(), item.into()).into()
}

// ─── #[global] ───────────────────────────────────────────────────────────────

/// Singleton reactive state. Generates the same struct as `#[store]` plus a
/// thread-local so that `::get()` returns a cheap owned handle.
#[proc_macro_attribute]
pub fn global(attr: TokenStream, item: TokenStream) -> TokenStream {
    store_global::expand_global(attr.into(), item.into()).into()
}

// ─── #[routes] ───────────────────────────────────────────────────────────────

/// Annotate a route enum to generate: `Route` trait impl, builder constants and
/// methods, and (for top-level enums) a thread-local router signal + `navigate()`.
#[proc_macro_attribute]
pub fn routes(attr: TokenStream, item: TokenStream) -> TokenStream {
    routes::expand(attr.into(), item.into()).into()
}

// ─── keyframes! ──────────────────────────────────────────────────────────────

/// Inject a `@keyframes` animation rule and return a [`KeyframeName`] handle.
///
/// Syntax: `keyframes!({ 0% { props } 50% { props } 100% { props } })`
///
/// Also supports `from`/`to` keywords in place of `0%`/`100%`.
///
/// Dynamic values using `{expr:spec}unit` syntax (same as `style!`) are supported.
/// The returned [`KeyframeName`] is a collision-safe generated identifier — pass
/// `.as_str()` wherever CSS expects an animation name.
#[proc_macro]
pub fn keyframes(input: TokenStream) -> TokenStream {
    keyframes::expand_keyframes(input)
}

// ─── #[server] / #[server_fn] ────────────────────────────────────────────────

/// Mark an `async fn` as a server function.
///
/// On server builds (`not(brick_dom)`): the function body is preserved and an
/// `inventory` registration is emitted so it can be dispatched via
/// `brick::ssr::call_server_fn`.
///
/// On client builds (`brick_dom`): the body is replaced with a `fetch_bytes`
/// call that POST-encodes arguments as JSON and deserialises the response.
///
/// Optional attributes: `path = "/custom/path"`, `encoding = "json"`.
#[proc_macro_attribute]
pub fn server(attr: TokenStream, item: TokenStream) -> TokenStream {
    server::expand(attr.into(), item.into()).into()
}

/// Alias for [`server`].  Prefer `#[server_fn]` in new code.
#[proc_macro_attribute]
pub fn server_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    server::expand(attr.into(), item.into()).into()
}

// ─── #[derive(Message)] ──────────────────────────────────────────────────────

/// Derive `prost::Message` for a plain Rust struct without manually annotating
/// every field with `#[prost(type, tag = "N")]`.
///
/// Field types are inferred from their Rust types, and stable protobuf tag
/// numbers are derived from a hash of each field name (`FNV-32 & 0x1FFF_FFFF`),
/// so tags remain stable across field reordering.
///
/// **Supported types:** `String`, `u32`, `u64`, `i32`, `i64`, `f32`, `f64`,
/// `bool`, `Vec<u8>` (bytes), `Vec<T>` (repeated), `Option<T>` (optional
/// message).  Nested message fields must use `Option<T>` or `Vec<T>`.
///
/// You still need `#[derive(Clone, PartialEq, Default)]` separately:
///
/// ```rust,ignore
/// #[derive(Clone, PartialEq, Default, brick::Message)]
/// struct PingRequest {
///     pub payload: String,
///     pub count: u32,
/// }
/// ```
#[proc_macro_derive(Message)]
pub fn message_derive(item: TokenStream) -> TokenStream {
    message::expand(item.into()).into()
}

// ─── Utility ─────────────────────────────────────────────────────────────────

#[proc_macro_attribute]
pub fn print_item(_attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("item: \"{}\"", item.to_string());
    item
}
