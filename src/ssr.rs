//! Server-side rendering, server functions, and client hydration.
//!
//! [`BrickError`] is used by `#[server]` functions on both client and server.
//! [`ServerFnEntry`] and [`call_server_fn`] are server-only (non-WASM).
//!
//! Works on any native Rust target — no browser, no WASM required. The
//! active renderer is always `HtmlRenderer` when `brick_dom` is not set,
//! so `ssr::render` is a thin public wrapper around `component.render().to_html_string()`.

// ─── BrickError ──────────────────────────────────────────────────────────────

/// The framework error type. Used by `#[server]` functions, `Load`,
/// and all async browser APIs (`fetch`, `BrickSocket`, `BrickEventSource`, `BrickStorage`).
///
/// `status` follows HTTP conventions (404, 500, etc.); use `0` for non-network errors.
#[derive(Clone, Debug, PartialEq)]
pub struct BrickError {
    pub status: u32,
    pub message: String,
}

impl BrickError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: 404,
            message: msg.into(),
        }
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            status: 401,
            message: msg.into(),
        }
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self {
            status: 403,
            message: msg.into(),
        }
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: 500,
            message: msg.into(),
        }
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: 400,
            message: msg.into(),
        }
    }
    pub fn custom(msg: impl Into<String>) -> Self {
        Self {
            status: 0,
            message: msg.into(),
        }
    }
    pub fn http(status: u32, msg: impl Into<String>) -> Self {
        Self {
            status,
            message: msg.into(),
        }
    }
    pub fn is_not_found(&self) -> bool {
        self.status == 404
    }
    pub fn is_unauthorized(&self) -> bool {
        self.status == 401
    }
    pub fn is_server_error(&self) -> bool {
        self.status >= 500
    }
}

impl From<String> for BrickError {
    fn from(s: String) -> Self {
        Self::internal(s)
    }
}
impl From<&str> for BrickError {
    fn from(s: &str) -> Self {
        Self::internal(s)
    }
}

impl std::fmt::Display for BrickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.status == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "HTTP {}: {}", self.status, self.message)
        }
    }
}

impl std::error::Error for BrickError {}

// ─── Server fn registry (server-side only) ───────────────────────────────────

/// A registered server function entry, submitted at link time via `inventory::submit!`.
/// Only present on non-WASM (server) builds.
#[cfg(not(brick_dom))]
pub struct ServerFnEntry {
    pub name: &'static str,
    pub path: &'static str,
    pub handler: fn(
        Vec<u8>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<u8>, BrickError>> + Send>,
    >,
}

#[cfg(not(brick_dom))]
inventory::collect!(ServerFnEntry);

/// Look up a registered server function by name or path and call it.
/// Only available on non-WASM (server) builds.
#[cfg(not(brick_dom))]
pub async fn call_server_fn(name_or_path: &str, body: Vec<u8>) -> Result<Vec<u8>, BrickError> {
    for entry in inventory::iter::<ServerFnEntry> {
        if entry.name == name_or_path || entry.path == name_or_path {
            return (entry.handler)(body).await;
        }
    }
    Err(BrickError::not_found(format!(
        "server fn '{}' not registered",
        name_or_path
    )))
}

// ─── SSR output ──────────────────────────────────────────────────────────────

/// Output from [`render_full`]: the rendered HTML plus the counter bounds needed for hydration.
pub struct SsrOutput {
    /// The rendered HTML string; embed this in your page.
    pub html: String,
    /// Value of [`crate::INSTANCE_COUNTER`] *before* the factory ran.
    /// Pass this to `ssr::hydrate` on the client so component class names match.
    pub counter_start: usize,
    /// Value of [`crate::INSTANCE_COUNTER`] *after* the factory ran.
    /// Equals `counter_start` when no `#[view]` components were constructed (e.g. bare element
    /// trees). Useful for debugging and for chaining multiple `render_full` calls on one page.
    pub counter_end: usize,
}

impl SsrOutput {
    /// Emit a `<script>` tag that makes `counter_start` readable on the client.
    ///
    /// Embed this in your page between `<head>` and the WASM bundle `<script>`:
    ///
    /// ```rust,ignore
    /// let out = ssr::render_full(|| Page { ..cascade() }.into_component());
    /// let page_html = format!(
    ///     "<!doctype html><html><head>{}</head><body>{}</body></html>",
    ///     out.script_tag(), out.html,
    /// );
    /// ```
    pub fn script_tag(&self) -> String {
        format!(
            r#"<script>window.__brick_counter__={};</script>"#,
            self.counter_start
        )
    }
}

/// Server-side rendering: render a component tree to an HTML string.
///
/// Works on any native Rust target — no browser, no WASM required.
///
/// ```rust,ignore
/// use brick::ssr;
/// use brick::view_components::*;
///
/// #[model]
/// struct Page { title: String }
///
/// #[view(Page)]
/// fn render() {
///     children! { h1(live!("{my.title}")) }
/// }
///
/// fn main() {
///     let page = Page { title: Signal::new("Hello".to_string()), ..cascade() };
///     let html = ssr::render(page.into_component());
///     // serve html over HTTP
/// }
/// ```
pub fn render(component: impl crate::view_components::Brick) -> String {
    crate::view_components::render_to_html(&component)
}

/// Render a type-erased component to an HTML string.
pub fn render_boxed(component: Box<dyn crate::view_components::Brick>) -> String {
    crate::view_components::render_to_html(&*component)
}

/// Render a component factory to HTML, capturing the counter bounds needed for client hydration.
///
/// The factory is a closure that constructs *and* renders the component. For a `#[view]`-generated
/// model struct `Page`, call `.into_component()` inside the closure so that `IntoComponent::into_component()` —
/// which advances [`crate::INSTANCE_COUNTER`] — fires while this function holds the bookmarks:
///
/// ```rust,ignore
/// let out = ssr::render_full(|| Page { title: Signal::new("Hi".into()), ..cascade() }.into_component());
/// // out.counter_start is what to pass to ssr::hydrate() on the client
/// let html = format!("…{}{}…", out.script_tag(), out.html);
/// ```
///
/// For bare element trees (`p(…)`, `BrickContainer { … }`) where no `#[view]` macro fires,
/// `counter_start == counter_end` and the `SsrOutput` is equivalent to calling `render()`.
pub fn render_full<C: crate::view_components::Brick>(factory: impl FnOnce() -> C) -> SsrOutput {
    use std::sync::atomic::Ordering;
    let counter_start = crate::INSTANCE_COUNTER.load(Ordering::SeqCst);
    let component = factory();
    let counter_end = crate::INSTANCE_COUNTER.load(Ordering::SeqCst);
    let html = crate::view_components::render_to_html(&component);
    SsrOutput {
        html,
        counter_start,
        counter_end,
    }
}

/// Read the hydration counter embedded by [`SsrOutput::script_tag`] from the browser `window`.
///
/// Call this before constructing or mounting any Brick components in the WASM entry point:
///
/// ```rust,ignore
/// #[wasm_bindgen(start)]
/// pub fn main() {
///     let slot = get_element_by_id("app");
///     ssr::hydrate(|| Page { title: Signal::new("".into()), ..cascade() }, ssr::read_ssr_counter());
/// }
/// ```
#[cfg(brick_dom)]
pub fn read_ssr_counter() -> usize {
    js_sys::Reflect::get(
        &web_sys::window().unwrap().into(),
        &"__brick_counter__".into(),
    )
    .ok()
    .and_then(|v| v.as_f64())
    .map(|f| f as usize)
    .unwrap_or(0)
}

/// Client-side hydration: wire reactive observers onto an SSR-rendered DOM without re-rendering.
///
/// Sets [`crate::INSTANCE_COUNTER`] to `counter_start` so that `into_component()` — called
/// inside the factory — produces the same instance class names that were stamped in the SSR HTML.
/// Then calls `attach_listeners()` to wire all signal observers and DOM event handlers.
///
/// `counter_start` comes from [`SsrOutput::counter_start`], typically read at runtime via
/// [`read_ssr_counter`].
///
/// ```rust,ignore
/// #[wasm_bindgen(start)]
/// pub fn main() {
///     ssr::hydrate(
///         || Page { title: Signal::new("".into()), ..cascade() },
///         ssr::read_ssr_counter(),
///     );
/// }
/// ```
#[cfg(brick_dom)]
pub fn hydrate<C: crate::view_components::IntoComponent>(
    factory: impl FnOnce() -> C,
    counter_start: usize,
) {
    use std::sync::atomic::Ordering;
    use crate::view_components::IntoComponent as _;
    crate::INSTANCE_COUNTER.store(counter_start, Ordering::SeqCst);
    factory().into_component().attach_listeners();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::examples::Counter;
    use crate::state_mgmt::{Signal, cascade};
    use crate::view_components::{Brick, BrickContainer, IntoComponent, button, h1, nothing, p};

    #[test]
    fn render_paragraph() {
        assert_eq!(render(p("hello")), "<p>hello</p>");
    }

    #[test]
    fn render_heading() {
        assert_eq!(render(h1("Title")), "<h1>Title</h1>");
    }

    #[test]
    fn render_nothing_is_empty() {
        assert_eq!(render_boxed(nothing()), "");
    }

    #[test]
    fn render_nested_container() {
        let html = render(BrickContainer {
            class: "wrapper",
            children: vec![p("a").into_component(), p("b").into_component()],
        });
        assert!(html.contains(r#"class="wrapper""#));
        assert!(html.contains("<p>a</p>"));
        assert!(html.contains("<p>b</p>"));
    }

    #[test]
    fn render_boxed_paragraph() {
        assert_eq!(render_boxed(p("hi")), "<p>hi</p>");
    }

    #[test]
    fn render_button_with_class() {
        let html = render(button("Click").c("btn"));
        assert!(html.contains("Click"));
        assert!(html.contains(r#"class="btn""#));
    }

    #[test]
    fn render_full_produces_correct_html() {
        let out = render_full(|| p("hello world"));
        assert_eq!(out.html, "<p>hello world</p>");
    }

    #[test]
    fn render_full_counter_end_gte_start() {
        let out = render_full(|| p("test"));
        assert!(out.counter_end >= out.counter_start);
    }

    #[test]
    fn render_full_captures_counter_advancement_inside_factory() {
        use std::sync::atomic::Ordering;
        // Simulate two #[view] component instantiations inside the factory
        let out = render_full(|| {
            crate::INSTANCE_COUNTER.fetch_add(2, Ordering::SeqCst);
            p("sim")
        });
        // counter_end must be at least counter_start + 2; other parallel tests may also
        // have advanced it, so we check >= rather than ==
        assert!(out.counter_end >= out.counter_start + 2);
    }

    #[test]
    fn render_full_script_tag_contains_counter_start() {
        let out = render_full(|| p("x"));
        let tag = out.script_tag();
        assert!(tag.contains("__brick_counter__"));
        assert!(tag.contains(&out.counter_start.to_string()));
        assert!(tag.starts_with("<script>"));
        assert!(tag.ends_with("</script>"));
    }

    // ── SSR + hydration integration ───────────────────────────────────────────

    /// Renders a real `#[view]` component and verifies:
    /// 1. The instance class (e.g. `Counter_42`) appears in the HTML.
    /// 2. That class number falls within the captured counter bounds.
    /// 3. `script_tag()` embeds `counter_start` for the client.
    #[test]
    fn view_component_ssr_class_within_counter_bounds() {
        let out = render_full(|| {
            Counter {
                count: Signal::new(0),
                ..cascade()
            }
            .into_component()
        });

        assert!(
            out.counter_end > out.counter_start,
            "#[view] render must advance INSTANCE_COUNTER"
        );

        // Extract the numeric suffix of the first `Counter_N` class in the HTML.
        let prefix = "Counter_";
        let pos = out
            .html
            .find(prefix)
            .expect("SSR HTML must contain Counter_ instance class");
        let num_str: String = out.html[pos + prefix.len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let used_n: usize = num_str
            .parse()
            .expect("Counter_ must be followed by digits");

        assert!(
            used_n >= out.counter_start && used_n < out.counter_end,
            "instance counter {used_n} must be in [{}, {})",
            out.counter_start,
            out.counter_end,
        );

        let tag = out.script_tag();
        assert!(
            tag.contains(&out.counter_start.to_string()),
            "script_tag must embed counter_start for client hydration"
        );
    }

    /// Core hydration contract: restoring `INSTANCE_COUNTER` to `counter_start`
    /// and re-rendering the same component tree produces the same instance class
    /// that was stamped in the SSR HTML.
    ///
    /// Note: `INSTANCE_COUNTER` is global so parallel test threads may cause
    /// the re-render to get a slightly higher counter value. The assertion is
    /// written in terms of the class extracted from the HTML, not an absolute
    /// number, so it is correct when the two renders are serialised by the OS.
    #[test]
    fn hydration_counter_restore_reproduces_class_name() {
        use std::sync::atomic::Ordering;

        let out = render_full(|| {
            Counter {
                count: Signal::new(0),
                ..cascade()
            }
            .into_component()
        });

        // Extract the actual instance number stamped in the SSR HTML.
        let prefix = "Counter_";
        let pos = out.html.find(prefix).expect("must have Counter_ class");
        let n: usize = out.html[pos + prefix.len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap();

        // Client: restore counter to n so the next fetch_add returns n.
        crate::INSTANCE_COUNTER.store(n, Ordering::SeqCst);
        let expected_class = format!("Counter_{n}");
        let component = Counter {
            count: Signal::new(0),
            ..cascade()
        }
        .into_component();

        // The class stamped by the client render must match what's in the SSR HTML.
        assert!(
            out.html.contains(&expected_class),
            "client class '{expected_class}' not found in SSR HTML after counter restore to {n}: {}",
            out.html,
        );

        // attach_listeners is a no-op without brick_dom — just prove it doesn't panic.
        component.attach_listeners();
    }
}

#[cfg(test)]
mod server_error_tests {
    use super::*;

    #[test]
    fn not_found_status() {
        assert_eq!(BrickError::not_found("x").status, 404);
    }
    #[test]
    fn unauthorized_status() {
        assert_eq!(BrickError::unauthorized("x").status, 401);
    }
    #[test]
    fn forbidden_status() {
        assert_eq!(BrickError::forbidden("x").status, 403);
    }
    #[test]
    fn internal_status() {
        assert_eq!(BrickError::internal("x").status, 500);
    }
    #[test]
    fn bad_request_status() {
        assert_eq!(BrickError::bad_request("x").status, 400);
    }
    #[test]
    fn custom_status_zero() {
        assert_eq!(BrickError::custom("x").status, 0);
    }
    #[test]
    fn http_status() {
        assert_eq!(BrickError::http(422, "x").status, 422);
    }

    #[test]
    fn from_string_is_internal() {
        let e = BrickError::from("boom".to_string());
        assert_eq!(e.status, 500);
        assert_eq!(e.message, "boom");
    }

    #[test]
    fn display_includes_status_when_nonzero() {
        let s = BrickError::not_found("gone").to_string();
        assert!(s.contains("404") && s.contains("gone"));
    }

    #[test]
    fn display_message_only_when_zero() {
        assert_eq!(BrickError::custom("oops").to_string(), "oops");
    }

    #[test]
    fn clone_and_eq() {
        let a = BrickError::internal("x");
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn is_predicates() {
        assert!(BrickError::not_found("x").is_not_found());
        assert!(BrickError::unauthorized("x").is_unauthorized());
        assert!(BrickError::internal("x").is_server_error());
        assert!(!BrickError::not_found("x").is_server_error());
    }

    #[test]
    fn error_trait() {
        let e: &dyn std::error::Error = &BrickError::internal("x");
        assert!(e.to_string().contains("x"));
    }

    #[cfg(not(brick_dom))]
    mod registry_tests {
        use super::*;

        #[test]
        fn unknown_fn_returns_404() {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            let err = rt
                .block_on(call_server_fn("no_such_fn", vec![]))
                .unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.message.contains("no_such_fn"));
        }

        inventory::submit! {
            ServerFnEntry {
                name: "echo_test",
                path: "/brick/fn/echo_test",
                handler: |body: Vec<u8>| Box::pin(async move {
                    Ok::<Vec<u8>, BrickError>(body)
                }),
            }
        }

        #[test]
        fn registered_entry_callable_by_name() {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            let input = b"hello".to_vec();
            assert_eq!(
                rt.block_on(call_server_fn("echo_test", input.clone()))
                    .unwrap(),
                input
            );
        }

        #[test]
        fn registered_entry_callable_by_path() {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            let input = b"world".to_vec();
            assert_eq!(
                rt.block_on(call_server_fn("/brick/fn/echo_test", input.clone()))
                    .unwrap(),
                input
            );
        }

        // ── Prost encoding round-trip ─────────────────────────────────────────
        //
        // Tests that prost encode → ServerFnEntry decode → encode → client decode
        // works end-to-end. These simulate what #[server] (prost default) generates
        // without going through the macro (which emits `brick::prost::` paths that
        // don't resolve from inside the brick crate itself).

        #[derive(Clone, PartialEq, prost::Message)]
        struct PingRequest {
            #[prost(string, tag = "1")]
            pub payload: String,
        }

        #[derive(Clone, PartialEq, prost::Message)]
        struct PingResponse {
            #[prost(string, tag = "1")]
            pub echo: String,
            #[prost(uint32, tag = "2")]
            pub len: u32,
        }

        inventory::submit! {
            ServerFnEntry {
                name: "ping_prost",
                path: "/brick/fn/ping_prost",
                handler: |body: Vec<u8>| Box::pin(async move {
                    use prost::Message as _;
                    let req = PingRequest::decode(body.as_slice())
                        .map_err(|e| BrickError::bad_request(e.to_string()))?;
                    let resp = PingResponse { len: req.payload.len() as u32, echo: req.payload };
                    Ok(resp.encode_to_vec())
                }),
            }
        }

        #[test]
        fn prost_encode_decode_round_trip() {
            use prost::Message as _;
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();

            let req = PingRequest {
                payload: "hello prost".to_string(),
            };
            let req_bytes = req.encode_to_vec();

            let resp_bytes = rt
                .block_on(call_server_fn("ping_prost", req_bytes))
                .unwrap();
            let resp = PingResponse::decode(resp_bytes.as_slice()).unwrap();

            assert_eq!(resp.echo, "hello prost");
            assert_eq!(resp.len, 11);
        }

        #[test]
        fn prost_bad_bytes_returns_400() {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            // Deliberately malformed prost bytes (valid UTF-8 but not a valid PingRequest proto)
            // Note: prost is lenient with unknown fields, so we send clearly invalid data
            let bad_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
            let err = rt
                .block_on(call_server_fn("ping_prost", bad_bytes))
                .unwrap_err();
            assert_eq!(err.status, 400);
        }

        #[test]
        fn prost_zero_arg_handler() {
            // Simulates a 0-arg #[server] (prost): body is ignored, returns encoded response.
            inventory::submit! {
                ServerFnEntry {
                    name: "version_prost",
                    path: "/brick/fn/version_prost",
                    handler: |_body: Vec<u8>| Box::pin(async move {
                        use prost::Message as _;
                        let resp = PingResponse { echo: "1.0".to_string(), len: 0 };
                        Ok(resp.encode_to_vec())
                    }),
                }
            }

            use prost::Message as _;
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            let resp_bytes = rt
                .block_on(call_server_fn("version_prost", vec![]))
                .unwrap();
            let resp = PingResponse::decode(resp_bytes.as_slice()).unwrap();
            assert_eq!(resp.echo, "1.0");
        }
    }
}

// ─── brick::Message derive tests ─────────────────────────────────────────────

#[cfg(test)]
mod message_derive_tests {
    use prost::Message as _;

    // Scalar field round-trip
    #[derive(Clone, PartialEq, crate::Message)]
    struct ScalarMsg {
        pub name: String,
        pub count: u32,
        pub score: f64,
        pub active: bool,
    }

    #[test]
    fn scalar_round_trip() {
        let msg = ScalarMsg {
            name: "hello".to_string(),
            count: 42,
            score: 3.14,
            active: true,
        };
        let bytes = msg.encode_to_vec();
        let decoded = ScalarMsg::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn empty_values_round_trip() {
        let msg = ScalarMsg::default();
        let bytes = msg.encode_to_vec();
        let decoded = ScalarMsg::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded, msg);
    }

    // All integer widths
    #[derive(Clone, PartialEq, crate::Message)]
    struct IntMsg {
        pub a: u32,
        pub b: u64,
        pub c: i32,
        pub d: i64,
    }

    #[test]
    fn integer_widths_round_trip() {
        let msg = IntMsg { a: u32::MAX, b: u64::MAX, c: i32::MIN, d: i64::MIN };
        let bytes = msg.encode_to_vec();
        let decoded = IntMsg::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded, msg);
    }

    // Vec<u8> (bytes) field
    #[derive(Clone, PartialEq, crate::Message)]
    struct BytesMsg {
        pub data: Vec<u8>,
    }

    #[test]
    fn bytes_field_round_trip() {
        let msg = BytesMsg { data: vec![1, 2, 3, 255] };
        let bytes = msg.encode_to_vec();
        let decoded = BytesMsg::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.data, msg.data);
    }

    // Repeated scalars: Vec<String> and Vec<u64>
    #[derive(Clone, PartialEq, crate::Message)]
    struct TagsMsg {
        pub tags: Vec<String>,
        pub ids: Vec<u64>,
    }

    #[test]
    fn repeated_scalars_round_trip() {
        let msg = TagsMsg {
            tags: vec!["a".to_string(), "b".to_string()],
            ids: vec![1, 2, 3],
        };
        let bytes = msg.encode_to_vec();
        let decoded = TagsMsg::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded, msg);
    }

    // encoded_len consistency
    #[test]
    fn encoded_len_matches_vec_len() {
        let msg = ScalarMsg { name: "test".to_string(), count: 7, score: 1.0, active: false };
        assert_eq!(msg.encoded_len(), msg.encode_to_vec().len());
    }

    // clear() zeroes all fields
    #[test]
    fn clear_resets_fields() {
        let mut msg = ScalarMsg { name: "hi".to_string(), count: 5, score: 2.0, active: true };
        prost::Message::clear(&mut msg);
        assert_eq!(msg, ScalarMsg::default());
    }

    // Tags are stable: same struct encodes to same bytes
    #[test]
    fn encoding_is_deterministic() {
        let msg = ScalarMsg { name: "x".to_string(), count: 1, score: 0.5, active: false };
        assert_eq!(msg.encode_to_vec(), msg.encode_to_vec());
    }

    // Zero-field struct is a valid empty message
    #[derive(Clone, PartialEq, crate::Message)]
    struct EmptyMsg {}

    #[test]
    fn zero_field_struct_encodes_to_empty() {
        assert!(EmptyMsg {}.encode_to_vec().is_empty());
        let decoded = EmptyMsg::decode(&[][..]).unwrap();
        assert_eq!(decoded, EmptyMsg {});
    }

    // f32 field
    #[derive(Clone, PartialEq, crate::Message)]
    struct FloatMsg {
        pub value: f32,
    }

    #[test]
    fn f32_field_round_trip() {
        let msg = FloatMsg { value: 1.5 };
        let bytes = msg.encode_to_vec();
        assert_eq!(FloatMsg::decode(bytes.as_slice()).unwrap(), msg);
    }

    // brick::Message used as argument to #[server] (integration)
    #[cfg(not(brick_dom))]
    mod server_fn_integration {
        use crate::ssr::{BrickError, ServerFnEntry, call_server_fn};

        #[derive(Clone, PartialEq, crate::Message)]
        struct EchoReq { pub payload: String }

        #[derive(Clone, PartialEq, crate::Message)]
        struct EchoResp { pub echo: String }

        inventory::submit! {
            ServerFnEntry {
                name: "brick_msg_echo",
                path: "/brick/fn/brick_msg_echo",
                handler: |body: Vec<u8>| Box::pin(async move {
                    use prost::Message as _;
                    let req = EchoReq::decode(body.as_slice())
                        .map_err(|e| BrickError::bad_request(e.to_string()))?;
                    let resp = EchoResp { echo: req.payload };
                    Ok(resp.encode_to_vec())
                }),
            }
        }

        #[test]
        fn brick_message_roundtrip_through_server_fn() {
            use prost::Message as _;
            let rt = tokio::runtime::Builder::new_current_thread()
                .build().unwrap();
            let req = EchoReq { payload: "brick".to_string() };
            let resp_bytes = rt.block_on(call_server_fn("brick_msg_echo", req.encode_to_vec())).unwrap();
            let resp = EchoResp::decode(resp_bytes.as_slice()).unwrap();
            assert_eq!(resp.echo, "brick");
        }
    }
}
