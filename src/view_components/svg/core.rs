use crate::state_mgmt::Signal;
use crate::view_components::Brick;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) static SVG_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn fmt_svg_f64(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

// ── IntoSvgF64 ───────────────────────────────────────────────────────────────

pub enum SvgF64Value {
    Static(f64),
    Reactive { initial: f64, signal: Signal<f64> },
}

/// Accepted by all SVG geometry attributes: `f64`, `f32`, `i32`, `i64`, `u32`,
/// `usize`, and `Signal<f64>` for reactive geometry.
pub trait IntoSvgF64 {
    fn into_svg_f64(self) -> SvgF64Value;
}

impl IntoSvgF64 for f64 {
    fn into_svg_f64(self) -> SvgF64Value {
        SvgF64Value::Static(self)
    }
}
impl IntoSvgF64 for f32 {
    fn into_svg_f64(self) -> SvgF64Value {
        SvgF64Value::Static(self as f64)
    }
}
impl IntoSvgF64 for i32 {
    fn into_svg_f64(self) -> SvgF64Value {
        SvgF64Value::Static(self as f64)
    }
}
impl IntoSvgF64 for i64 {
    fn into_svg_f64(self) -> SvgF64Value {
        SvgF64Value::Static(self as f64)
    }
}
impl IntoSvgF64 for u32 {
    fn into_svg_f64(self) -> SvgF64Value {
        SvgF64Value::Static(self as f64)
    }
}
impl IntoSvgF64 for usize {
    fn into_svg_f64(self) -> SvgF64Value {
        SvgF64Value::Static(self as f64)
    }
}

impl IntoSvgF64 for Signal<f64> {
    fn into_svg_f64(self) -> SvgF64Value {
        let initial = self.read();
        SvgF64Value::Reactive {
            initial,
            signal: self,
        }
    }
}

// ── ReactiveF64Attr ───────────────────────────────────────────────────────────

pub(crate) struct ReactiveF64Attr {
    pub attr_name: String,
    pub identity_class: String,
    pub signal: Signal<f64>,
}

// ── SvgLeaf ──────────────────────────────────────────────────────────────────

/// Backing struct for all SVG leaf elements. Holds the resolved attribute set,
/// optional inner text, optional children (for `<animate>`, `<tspan>`, etc.),
/// and any reactive geometry signals that need DOM observers wired after mount.
pub struct SvgLeaf {
    pub(crate) tag: &'static str,
    pub(crate) class_name: String,
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) reactive_attrs: Vec<ReactiveF64Attr>,
    pub(crate) transform: Option<String>,
    pub(crate) inner_text: Option<String>,
    pub(crate) children: Vec<Box<dyn Brick>>,
}

impl SvgLeaf {
    pub(crate) fn new(tag: &'static str) -> Self {
        SvgLeaf {
            tag,
            class_name: String::new(),
            attrs: Vec::new(),
            reactive_attrs: Vec::new(),
            transform: None,
            inner_text: None,
            children: Vec::new(),
        }
    }

    /// Push a numeric attribute, registering a DOM observer if the value is reactive.
    pub(crate) fn push_f64(&mut self, attr: &'static str, val: impl IntoSvgF64) {
        match val.into_svg_f64() {
            SvgF64Value::Static(v) => {
                self.attrs.push((attr.to_string(), fmt_svg_f64(v)));
            }
            SvgF64Value::Reactive { initial, signal } => {
                let id = SVG_COUNTER.fetch_add(1, Ordering::SeqCst);
                let identity = format!("brick-svg-{}", id);
                self.attrs.push((attr.to_string(), fmt_svg_f64(initial)));
                self.reactive_attrs.push(ReactiveF64Attr {
                    attr_name: attr.to_string(),
                    identity_class: identity.clone(),
                    signal,
                });
                self.add_class(&identity);
            }
        }
    }

    pub(crate) fn push_str_attr(&mut self, attr: &'static str, val: &str) {
        self.attrs.push((attr.to_string(), val.to_string()));
    }

    pub(crate) fn push_owned_attr(&mut self, attr: String, val: String) {
        self.attrs.push((attr, val));
    }

    pub(crate) fn add_class(&mut self, c: &str) {
        if self.class_name.is_empty() {
            self.class_name = c.to_string();
        } else {
            self.class_name.push(' ');
            self.class_name.push_str(c);
        }
    }

    pub(crate) fn render_open_tag(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.class_name.is_empty() {
            parts.push(format!(r#"class="{}""#, self.class_name));
        }
        if let Some(ref t) = self.transform {
            parts.push(format!(r#"transform="{}""#, t));
        }
        for (k, v) in &self.attrs {
            parts.push(format!(r#"{}="{}""#, k, v));
        }
        if parts.is_empty() {
            format!("<{}", self.tag)
        } else {
            format!("<{} {}", self.tag, parts.join(" "))
        }
    }

    pub(crate) fn html(&self) -> String {
        if self.children.is_empty() && self.inner_text.is_none() {
            format!("{}/>", self.render_open_tag())
        } else {
            let text = self.inner_text.as_deref().unwrap_or("");
            let child_html: String = self
                .children
                .iter()
                .map(|c| crate::view_components::render_to_html(c.as_ref()))
                .collect();
            format!(
                "{}>{}{}</{}>",
                self.render_open_tag(),
                text,
                child_html,
                self.tag
            )
        }
    }

    pub(crate) fn attach_reactive(&self) {
        for ra in &self.reactive_attrs {
            #[cfg(brick_dom)]
            {
                use crate::state_mgmt::observers::Effect;
                let attr = ra.attr_name.clone();
                let class = ra.identity_class.clone();
                ra.signal.add_observer(
                    &class.clone(),
                    Box::new(Effect::new(move |v: &f64| {
                        if let Some(el) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.get_elements_by_class_name(&class).item(0))
                        {
                            let _ = el.set_attribute(&attr, &fmt_svg_f64(*v));
                        }
                    })),
                );
            }
            #[cfg(not(brick_dom))]
            let _ = ra;
        }
    }
}

// ── SvgContainer ─────────────────────────────────────────────────────────────

/// Backing struct for SVG container elements (`<svg>`, `<g>`, `<defs>`, etc.).
/// Holds a `SvgLeaf` for attrs and a children vec for nested elements.
pub struct SvgContainer {
    pub(crate) leaf: SvgLeaf,
    pub(crate) children: Vec<Box<dyn Brick>>,
}

impl SvgContainer {
    pub(crate) fn new(tag: &'static str) -> Self {
        SvgContainer {
            leaf: SvgLeaf::new(tag),
            children: Vec::new(),
        }
    }

    pub(crate) fn html(&self) -> String {
        let open = format!("{}>", self.leaf.render_open_tag());
        let inner: String = self
            .children
            .iter()
            .map(|c| crate::view_components::render_to_html(c.as_ref()))
            .collect();
        format!("{}{}</{}>", open, inner, self.leaf.tag)
    }

    pub(crate) fn attach(&self) {
        self.leaf.attach_reactive();
    }
}

// ── Phase 10: Brick impls ────────────────────────────────────────────

impl crate::view_components::Brick for SvgLeaf {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, HtmlNode, Renderer};
        Renderer::append(parent, &HtmlNode::raw(self.html()));
    }
}

impl crate::view_components::Brick for SvgContainer {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element(self.leaf.tag);
        if !self.leaf.class_name.is_empty() {
            Renderer::set_attr(&node, "class", &self.leaf.class_name);
        }
        if let Some(ref t) = self.leaf.transform {
            Renderer::set_attr(&node, "transform", t);
        }
        for (k, v) in &self.leaf.attrs {
            Renderer::set_attr(&node, k, v);
        }
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }
}

// ── HasSvgLeaf ───────────────────────────────────────────────────────────────

/// Implemented by all SVG element wrappers. Unlocks `SvgElemMethods` via
/// blanket impl — giving every element `.fill()`, `.stroke()`, `.transform()`,
/// `.opacity()`, `.class()`, `.attr()`, `.animate()`, and more for free.
pub trait HasSvgLeaf {
    fn leaf(&self) -> &SvgLeaf;
    fn leaf_mut(&mut self) -> &mut SvgLeaf;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_svg_f64_whole_numbers_omit_decimal() {
        assert_eq!(fmt_svg_f64(10.0), "10");
        assert_eq!(fmt_svg_f64(0.0), "0");
        assert_eq!(fmt_svg_f64(-5.0), "-5");
    }

    #[test]
    fn fmt_svg_f64_fractions_preserved() {
        assert_eq!(fmt_svg_f64(10.5), "10.5");
        #[allow(clippy::approx_constant)]
        {
            assert_eq!(fmt_svg_f64(3.14), "3.14");
        }
    }

    #[test]
    fn svgleaf_static_attr_renders() {
        let mut leaf = SvgLeaf::new("circle");
        leaf.push_f64("cx", 50.0f64);
        leaf.push_f64("r", 30.0f64);
        let html = leaf.html(); // SvgLeaf::html() is still a pub(crate) helper
        assert!(html.starts_with("<circle"), "tag present");
        assert!(html.contains(r#"cx="50""#));
        assert!(html.contains(r#"r="30""#));
        assert!(html.ends_with("/>"), "self-closing");
    }

    #[test]
    fn svgleaf_with_transform() {
        let mut leaf = SvgLeaf::new("rect");
        leaf.push_f64("width", 100.0f64);
        leaf.transform = Some("translate(10,20)".to_string());
        let html = leaf.html();
        assert!(html.contains(r#"transform="translate(10,20)""#));
    }

    #[test]
    fn svgleaf_with_class() {
        let mut leaf = SvgLeaf::new("circle");
        leaf.add_class("my-circle");
        let html = leaf.html();
        assert!(html.contains(r#"class="my-circle""#));
    }

    #[test]
    fn svgleaf_with_inner_text_renders_as_element_with_content() {
        let mut leaf = SvgLeaf::new("text");
        leaf.push_f64("x", 10.0f64);
        leaf.inner_text = Some("Hello".to_string());
        let html = leaf.html();
        assert_eq!(html, r#"<text x="10">Hello</text>"#);
    }

    #[test]
    fn svgcontainer_wraps_children() {
        let mut c = SvgContainer::new("g");
        let mut child = SvgLeaf::new("circle");
        child.push_f64("r", 5.0f64);
        struct LeafWrap(SvgLeaf);
        impl crate::view_components::Brick for LeafWrap {
            fn render_into(&self, parent: &crate::renderer::BrickNode) {
                use crate::renderer::{BrickRenderer as _, HtmlNode, Renderer};
                Renderer::append(parent, &HtmlNode::raw(self.0.html()));
            }
        }
        c.children.push(Box::new(LeafWrap(child)));
        let html = crate::view_components::render_to_html(&c as &dyn crate::view_components::Brick);
        assert!(html.starts_with("<g>"));
        assert!(html.ends_with("</g>"));
        assert!(html.contains(r#"r="5""#));
    }

    #[test]
    fn into_svg_f64_numeric_types() {
        assert!(matches!(10.0f64.into_svg_f64(), SvgF64Value::Static(v) if v == 10.0));
        assert!(matches!(10.0f32.into_svg_f64(), SvgF64Value::Static(v) if v == 10.0));
        assert!(matches!(10i32.into_svg_f64(), SvgF64Value::Static(v) if v == 10.0));
        assert!(matches!(10usize.into_svg_f64(), SvgF64Value::Static(v) if v == 10.0));
    }

    #[test]
    fn into_svg_f64_signal_reads_initial() {
        let sig = Signal::new(42.0f64);
        match sig.into_svg_f64() {
            SvgF64Value::Reactive { initial, .. } => assert_eq!(initial, 42.0),
            _ => panic!("expected Reactive"),
        }
    }
}
