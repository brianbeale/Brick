use super::core::{HasSvgLeaf, IntoSvgF64, SvgLeaf};
use crate::view_components::{Brick, IntoComponent};

// ── SvgText ───────────────────────────────────────────────────────────────────

/// `<text>` — renders text at `(x, y)`. Chain `.content()` for a plain string
/// or `.tspan()` to add positioned `<tspan>` children.
///
/// ```rust,ignore
/// svg_text(50.0, 50.0)
///     .content("Hello world")
///     .font_size("16px")
///     .text_anchor("middle")
///     .fill("white")
/// ```
pub struct SvgText {
    leaf: SvgLeaf,
}

pub fn svg_text(x: impl IntoSvgF64, y: impl IntoSvgF64) -> SvgText {
    let mut leaf = SvgLeaf::new("text");
    leaf.push_f64("x", x);
    leaf.push_f64("y", y);
    SvgText { leaf }
}

/// Create a `<text>` element at `(x, y)` with immediate text content.
pub fn svg_text_str(x: impl IntoSvgF64, y: impl IntoSvgF64, content: &str) -> SvgText {
    svg_text(x, y).content(content)
}

impl HasSvgLeaf for SvgText {
    fn leaf(&self) -> &SvgLeaf {
        &self.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.leaf
    }
}

impl IntoComponent for SvgText {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl Brick for SvgText {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
    fn attach_listeners(&self) {
        self.leaf.attach_reactive();
    }
}

impl SvgText {
    pub fn content(mut self, text: &str) -> Self {
        self.leaf.inner_text = Some(super::escape_svg_text(text));
        self
    }

    pub fn tspan(mut self, t: SvgTspan) -> Self {
        self.leaf.children.push(Box::new(t));
        self
    }

    pub fn font_family(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("font-family", v);
        self
    }

    pub fn font_size(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("font-size", v);
        self
    }

    pub fn font_weight(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("font-weight", v);
        self
    }

    pub fn font_style(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("font-style", v);
        self
    }

    pub fn text_anchor(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("text-anchor", v);
        self
    }

    pub fn dominant_baseline(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("dominant-baseline", v);
        self
    }

    pub fn letter_spacing(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("letter-spacing", v);
        self
    }

    pub fn text_decoration(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("text-decoration", v);
        self
    }

    pub fn writing_mode(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("writing-mode", v);
        self
    }
}

// ── SvgTspan ──────────────────────────────────────────────────────────────────

/// `<tspan>` — a positioned run of text inside `<text>`. Use `.tspan()` on
/// `SvgText` to add one.
///
/// ```rust,ignore
/// svg_text(10.0, 20.0)
///     .tspan(svg_tspan("Line 1"))
///     .tspan(svg_tspan("Line 2").x(10.0).dy(1.4))
/// ```
pub struct SvgTspan {
    leaf: SvgLeaf,
}

pub fn svg_tspan(text: &str) -> SvgTspan {
    let mut leaf = SvgLeaf::new("tspan");
    leaf.inner_text = Some(super::escape_svg_text(text));
    SvgTspan { leaf }
}

impl HasSvgLeaf for SvgTspan {
    fn leaf(&self) -> &SvgLeaf {
        &self.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.leaf
    }
}

impl IntoComponent for SvgTspan {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl Brick for SvgTspan {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
    fn attach_listeners(&self) {
        self.leaf.attach_reactive();
    }
}

impl SvgTspan {
    pub fn x(mut self, v: impl IntoSvgF64) -> Self {
        self.leaf.push_f64("x", v);
        self
    }
    pub fn y(mut self, v: impl IntoSvgF64) -> Self {
        self.leaf.push_f64("y", v);
        self
    }
    pub fn dx(mut self, v: impl IntoSvgF64) -> Self {
        self.leaf.push_f64("dx", v);
        self
    }
    pub fn dy(mut self, v: impl IntoSvgF64) -> Self {
        self.leaf.push_f64("dy", v);
        self
    }
    pub fn font_size(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("font-size", v);
        self
    }
    pub fn font_weight(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("font-weight", v);
        self
    }
    pub fn font_style(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("font-style", v);
        self
    }
    pub fn text_anchor(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("text-anchor", v);
        self
    }
    pub fn dominant_baseline(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("dominant-baseline", v);
        self
    }
    pub fn rotate(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("rotate", v);
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.leaf.push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::presentation::SvgElemMethods;
    use super::*;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn svg_text_simple_content() {
        let html = rh(&svg_text(10.0, 20.0).content("Hello"));
        assert_eq!(html, r#"<text x="10" y="20">Hello</text>"#);
    }

    #[test]
    fn svg_text_escapes_html() {
        let html = rh(&svg_text(0.0, 0.0).content("<br> & \"test\""));
        assert!(html.contains("&lt;br&gt;"));
        assert!(html.contains("&amp;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn svg_text_str_shorthand() {
        let html = rh(&svg_text_str(5.0, 15.0, "Hi"));
        assert!(html.contains("Hi"));
        assert!(html.contains(r#"x="5""#));
    }

    #[test]
    fn svg_text_with_font_attrs() {
        let html = rh(&svg_text(0.0, 0.0)
            .content("Label")
            .font_size("14px")
            .font_family("sans-serif")
            .text_anchor("middle")
            .dominant_baseline("central"));
        assert!(html.contains(r#"font-size="14px""#));
        assert!(html.contains(r#"text-anchor="middle""#));
        assert!(html.contains(r#"dominant-baseline="central""#));
    }

    #[test]
    fn svg_text_with_fill() {
        let html = rh(&svg_text(0.0, 0.0).content("X").fill("white").done());
        assert!(html.contains(r#"fill="white""#));
        assert!(html.contains("X"));
    }

    #[test]
    fn svg_tspan_renders() {
        let html = rh(&svg_tspan("World").x(10.0).dy(1.2));
        assert_eq!(html, r#"<tspan x="10" dy="1.2">World</tspan>"#);
    }

    #[test]
    fn svg_text_with_tspan_children() {
        let html = rh(&svg_text(10.0, 20.0)
            .tspan(svg_tspan("Line 1"))
            .tspan(svg_tspan("Line 2").dy(1.5)));
        assert!(html.starts_with(r#"<text x="10" y="20">"#));
        assert!(html.contains("<tspan>Line 1</tspan>"));
        assert!(html.contains(r#"dy="1.5""#));
        assert!(html.ends_with("</text>"));
    }

    #[test]
    fn svg_text_into_component() {
        let c = svg_text(0.0, 0.0).content("hi").into_component();
        assert!(rh(&*c).contains("hi"));
    }
}
