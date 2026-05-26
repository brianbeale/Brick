use crate::renderer::BrickEvent;

/// Extracts a typed value from a `BrickEvent`. Implemented for `String`, `f64`, and `bool` —
/// the three types that can be bound to an input with `.bind(signal)`.
pub trait FromBrickEvent: Clone + std::fmt::Display + 'static {
    fn from_brick_event(event: BrickEvent) -> Option<Self>;
}

impl FromBrickEvent for String {
    fn from_brick_event(e: BrickEvent) -> Option<String> {
        if let BrickEvent::InputString(v) = e { Some(v) } else { None }
    }
}

impl FromBrickEvent for f64 {
    fn from_brick_event(e: BrickEvent) -> Option<f64> {
        if let BrickEvent::InputNumber(v) = e { Some(v) } else { None }
    }
}

impl FromBrickEvent for bool {
    fn from_brick_event(e: BrickEvent) -> Option<bool> {
        if let BrickEvent::InputBool(v) = e { Some(v) } else { None }
    }
}

// Kept for downstream compatibility during transition — remove at v1.0.
#[deprecated(note = "use FromBrickEvent instead")]
pub trait FromInputEvent: Clone + std::fmt::Display + 'static {
    fn from_event(event: &web_sys::Event) -> Option<Self>;
}

#[allow(deprecated)]
impl FromInputEvent for f64 {
    fn from_event(e: &web_sys::Event) -> Option<f64> { extract_f64(e) }
}

#[allow(deprecated)]
impl FromInputEvent for String {
    fn from_event(e: &web_sys::Event) -> Option<String> { extract_string(e) }
}

#[allow(deprecated)]
impl FromInputEvent for bool {
    fn from_event(e: &web_sys::Event) -> Option<bool> { extract_bool(e) }
}

// ─── DOM → domain Adapters ───────────────────────────────────────────────────

use wasm_bindgen::JsCast;

pub fn extract_f64(event: &web_sys::Event) -> Option<f64> {
    event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.value_as_number())
        .filter(|v: &f64| !v.is_nan())
}

#[allow(dead_code)]
pub fn extract_string(event: &web_sys::Event) -> Option<String> {
    event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.value())
}

#[allow(dead_code)]
pub fn extract_bool(event: &web_sys::Event) -> Option<bool> {
    event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.checked())
}
