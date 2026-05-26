use super::color::IntoSvgColor;
use super::core::{HasSvgLeaf, IntoSvgF64, SvgF64Value, SvgLeaf, fmt_svg_f64};
use super::transform::IntoSvgTransform;
use crate::view_components::IntoComponent;

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Linecap {
    Butt,
    Round,
    Square,
}

impl Linecap {
    pub fn as_str(self) -> &'static str {
        match self {
            Linecap::Butt => "butt",
            Linecap::Round => "round",
            Linecap::Square => "square",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Linejoin {
    Miter,
    Round,
    Bevel,
    Arcs,
    MiterClip,
}

impl Linejoin {
    pub fn as_str(self) -> &'static str {
        match self {
            Linejoin::Miter => "miter",
            Linejoin::Round => "round",
            Linejoin::Bevel => "bevel",
            Linejoin::Arcs => "arcs",
            Linejoin::MiterClip => "miter-clip",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

impl FillRule {
    pub fn as_str(self) -> &'static str {
        match self {
            FillRule::NonZero => "nonzero",
            FillRule::EvenOdd => "evenodd",
        }
    }
}

// ── StrokeBuilder ─────────────────────────────────────────────────────────────

/// Sub-builder for stroke presentation attributes. Returned by `.stroke(color)`.
/// Chain `.width()`, `.opacity()`, `.linecap()`, `.linejoin()`, `.dasharray()`,
/// `.dashoffset()`, `.miterlimit()`.
///
/// Exit back to element-level methods by calling `.fill()`, `.transform()`,
/// `.attr()`, `.class()`, or terminate into a component via `IntoComponent`.
pub struct StrokeBuilder<T: HasSvgLeaf> {
    pub(crate) inner: T,
}

impl<T: HasSvgLeaf> StrokeBuilder<T> {
    pub(crate) fn new(inner: T) -> Self {
        StrokeBuilder { inner }
    }

    pub fn width(mut self, w: impl IntoSvgF64) -> Self {
        let v = match w.into_svg_f64() {
            SvgF64Value::Static(v) => fmt_svg_f64(v),
            SvgF64Value::Reactive { initial, .. } => fmt_svg_f64(initial),
        };
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke-width".to_string(), v));
        self
    }

    pub fn opacity(mut self, o: f64) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke-opacity".to_string(), fmt_svg_f64(o)));
        self
    }

    pub fn linecap(mut self, cap: Linecap) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke-linecap".to_string(), cap.as_str().to_string()));
        self
    }

    pub fn linejoin(mut self, join: Linejoin) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke-linejoin".to_string(), join.as_str().to_string()));
        self
    }

    pub fn dasharray(mut self, pattern: &str) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke-dasharray".to_string(), pattern.to_string()));
        self
    }

    pub fn dashoffset(mut self, offset: f64) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke-dashoffset".to_string(), fmt_svg_f64(offset)));
        self
    }

    pub fn miterlimit(mut self, v: f64) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke-miterlimit".to_string(), fmt_svg_f64(v)));
        self
    }

    // ── Exit stroke context ───────────────────────────────────────────────────

    pub fn fill(mut self, color: impl IntoSvgColor) -> FillBuilder<T> {
        self.inner
            .leaf_mut()
            .attrs
            .push(("fill".to_string(), color.into_svg_color()));
        FillBuilder::new(self.inner)
    }

    pub fn fill_opacity(mut self, o: f64) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("fill-opacity".to_string(), fmt_svg_f64(o)));
        self.inner
    }

    pub fn transform(mut self, t: impl IntoSvgTransform) -> T {
        self.inner.leaf_mut().transform = Some(t.into_svg_transform());
        self.inner
    }

    pub fn class(mut self, c: &str) -> T {
        self.inner.leaf_mut().add_class(c);
        self.inner
    }

    pub fn attr(mut self, k: &str, v: &str) -> T {
        self.inner
            .leaf_mut()
            .push_owned_attr(k.to_string(), v.to_string());
        self.inner
    }

    pub fn id(mut self, id: &str) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("id".to_string(), id.to_string()));
        self.inner
    }

    pub fn clip_path(mut self, id: &str) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("clip-path".to_string(), format!("url(#{})", id)));
        self.inner
    }

    pub fn filter(mut self, id: &str) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("filter".to_string(), format!("url(#{})", id)));
        self.inner
    }

    pub fn mask_ref(mut self, id: &str) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("mask".to_string(), format!("url(#{})", id)));
        self.inner
    }

    pub fn done(self) -> T {
        self.inner
    }
}

impl<T: HasSvgLeaf + crate::view_components::Brick + 'static> crate::view_components::Brick
    for StrokeBuilder<T>
{
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        self.inner.render_into(parent)
    }
    fn attach_listeners(&self) {
        self.inner.attach_listeners();
    }
}

impl<T: HasSvgLeaf + crate::view_components::Brick + 'static> IntoComponent for StrokeBuilder<T> {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self.inner)
    }
}

// ── FillBuilder ───────────────────────────────────────────────────────────────

/// Sub-builder for fill presentation attributes. Returned by `.fill(color)`.
/// Chain `.opacity()`, `.rule()`. Exit with `.stroke()`, `.transform()`, etc.
pub struct FillBuilder<T: HasSvgLeaf> {
    pub(crate) inner: T,
}

impl<T: HasSvgLeaf> FillBuilder<T> {
    pub(crate) fn new(inner: T) -> Self {
        FillBuilder { inner }
    }

    pub fn opacity(mut self, o: f64) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("fill-opacity".to_string(), fmt_svg_f64(o)));
        self
    }

    pub fn rule(mut self, r: FillRule) -> Self {
        self.inner
            .leaf_mut()
            .attrs
            .push(("fill-rule".to_string(), r.as_str().to_string()));
        self
    }

    // ── Exit fill context ─────────────────────────────────────────────────────

    pub fn stroke(mut self, color: impl IntoSvgColor) -> StrokeBuilder<T> {
        self.inner
            .leaf_mut()
            .attrs
            .push(("stroke".to_string(), color.into_svg_color()));
        StrokeBuilder::new(self.inner)
    }

    pub fn transform(mut self, t: impl IntoSvgTransform) -> T {
        self.inner.leaf_mut().transform = Some(t.into_svg_transform());
        self.inner
    }

    pub fn class(mut self, c: &str) -> T {
        self.inner.leaf_mut().add_class(c);
        self.inner
    }

    pub fn attr(mut self, k: &str, v: &str) -> T {
        self.inner
            .leaf_mut()
            .push_owned_attr(k.to_string(), v.to_string());
        self.inner
    }

    pub fn id(mut self, id: &str) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("id".to_string(), id.to_string()));
        self.inner
    }

    pub fn clip_path(mut self, id: &str) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("clip-path".to_string(), format!("url(#{})", id)));
        self.inner
    }

    pub fn filter(mut self, id: &str) -> T {
        self.inner
            .leaf_mut()
            .attrs
            .push(("filter".to_string(), format!("url(#{})", id)));
        self.inner
    }

    pub fn done(self) -> T {
        self.inner
    }
}

impl<T: HasSvgLeaf + crate::view_components::Brick + 'static> crate::view_components::Brick
    for FillBuilder<T>
{
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        self.inner.render_into(parent)
    }
    fn attach_listeners(&self) {
        self.inner.attach_listeners();
    }
}

impl<T: HasSvgLeaf + crate::view_components::Brick + 'static> IntoComponent for FillBuilder<T> {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self.inner)
    }
}

// ── SvgElemMethods ────────────────────────────────────────────────────────────

/// Blanket trait giving every SVG element the common presentation and layout
/// builder methods. Implemented automatically for any `T: HasSvgLeaf + Sized`.
///
/// ```rust,ignore
/// svg_circle(50.0, 50.0, 30.0)
///     .fill("steelblue")
///         .opacity(0.9)
///         .stroke("black").width(2.0).linecap(Linecap::Round)
///     .transform(Transform::rotate(15.0))
///     .class("highlighted")
/// ```
pub trait SvgElemMethods: HasSvgLeaf + Sized {
    fn fill(mut self, color: impl IntoSvgColor) -> FillBuilder<Self> {
        self.leaf_mut()
            .attrs
            .push(("fill".to_string(), color.into_svg_color()));
        FillBuilder::new(self)
    }

    fn fill_opacity(mut self, o: f64) -> Self {
        self.leaf_mut()
            .attrs
            .push(("fill-opacity".to_string(), fmt_svg_f64(o)));
        self
    }

    fn fill_rule(mut self, r: FillRule) -> Self {
        self.leaf_mut()
            .attrs
            .push(("fill-rule".to_string(), r.as_str().to_string()));
        self
    }

    fn stroke(mut self, color: impl IntoSvgColor) -> StrokeBuilder<Self> {
        self.leaf_mut()
            .attrs
            .push(("stroke".to_string(), color.into_svg_color()));
        StrokeBuilder::new(self)
    }

    fn opacity(mut self, o: f64) -> Self {
        self.leaf_mut()
            .attrs
            .push(("opacity".to_string(), fmt_svg_f64(o)));
        self
    }

    fn transform(mut self, t: impl IntoSvgTransform) -> Self {
        self.leaf_mut().transform = Some(t.into_svg_transform());
        self
    }

    fn class(mut self, c: &str) -> Self {
        self.leaf_mut().add_class(c);
        self
    }

    fn id(mut self, id: &str) -> Self {
        self.leaf_mut()
            .attrs
            .push(("id".to_string(), id.to_string()));
        self
    }

    fn attr(mut self, k: &str, v: &str) -> Self {
        self.leaf_mut()
            .push_owned_attr(k.to_string(), v.to_string());
        self
    }

    fn clip_path(mut self, id: &str) -> Self {
        self.leaf_mut()
            .attrs
            .push(("clip-path".to_string(), format!("url(#{})", id)));
        self
    }

    fn filter(mut self, id: &str) -> Self {
        self.leaf_mut()
            .attrs
            .push(("filter".to_string(), format!("url(#{})", id)));
        self
    }

    fn mask_ref(mut self, id: &str) -> Self {
        self.leaf_mut()
            .attrs
            .push(("mask".to_string(), format!("url(#{})", id)));
        self
    }

    fn pointer_events(mut self, v: &str) -> Self {
        self.leaf_mut()
            .attrs
            .push(("pointer-events".to_string(), v.to_string()));
        self
    }

    fn cursor(mut self, v: &str) -> Self {
        self.leaf_mut()
            .attrs
            .push(("cursor".to_string(), v.to_string()));
        self
    }

    fn visibility(mut self, v: &str) -> Self {
        self.leaf_mut()
            .attrs
            .push(("visibility".to_string(), v.to_string()));
        self
    }

    fn animate(mut self, anim: super::animate::Animate) -> Self {
        self.leaf_mut().children.push(Box::new(anim));
        self
    }

    fn animate_transform(mut self, anim: super::animate::AnimateTransform) -> Self {
        self.leaf_mut().children.push(Box::new(anim));
        self
    }

    fn animate_motion(mut self, anim: super::animate::AnimateMotion) -> Self {
        self.leaf_mut().children.push(Box::new(anim));
        self
    }

    fn trigger(mut self, action: &crate::state_mgmt::BrickAction) -> Self {
        self.leaf_mut()
            .push_str_attr("data-brick-action", action.name);
        self
    }
}

impl<T: HasSvgLeaf + Sized> SvgElemMethods for T {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::Brick;

    struct TestElem {
        leaf: SvgLeaf,
    }

    impl HasSvgLeaf for TestElem {
        fn leaf(&self) -> &SvgLeaf {
            &self.leaf
        }
        fn leaf_mut(&mut self) -> &mut SvgLeaf {
            &mut self.leaf
        }
    }

    impl crate::view_components::Brick for TestElem {
        fn attach_listeners(&self) {
            self.leaf.attach_reactive();
        }
        fn render_into(&self, parent: &crate::renderer::BrickNode) {
            use crate::renderer::{BrickRenderer as _, HtmlNode, Renderer};
            Renderer::append(
                parent,
                &HtmlNode::raw(crate::view_components::render_to_html(
                    &self.leaf as &dyn crate::view_components::Brick,
                )),
            );
        }
    }

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    fn elem(tag: &'static str) -> TestElem {
        TestElem {
            leaf: SvgLeaf::new(tag),
        }
    }

    #[test]
    fn fill_sets_attr() {
        let html = rh(&elem("circle").fill("steelblue").done());
        assert!(html.contains(r#"fill="steelblue""#));
    }

    #[test]
    fn fill_opacity_chained() {
        let html = rh(&elem("circle").fill("red").opacity(0.5).done());
        assert!(html.contains(r#"fill="red""#));
        assert!(html.contains(r#"fill-opacity="0.5""#));
    }

    #[test]
    fn stroke_width_chain() {
        let html = rh(&elem("rect").stroke("black").width(2.0).done());
        assert!(html.contains(r#"stroke="black""#));
        assert!(html.contains(r#"stroke-width="2""#));
    }

    #[test]
    fn stroke_linecap_linejoin() {
        let html = rh(&elem("path")
            .stroke("navy")
            .width(1.5)
            .linecap(Linecap::Round)
            .linejoin(Linejoin::Bevel)
            .done());
        assert!(html.contains(r#"stroke-linecap="round""#));
        assert!(html.contains(r#"stroke-linejoin="bevel""#));
    }

    #[test]
    fn stroke_dasharray() {
        let html = rh(&elem("line").stroke("gray").dasharray("4 2").done());
        assert!(html.contains(r#"stroke-dasharray="4 2""#));
    }

    #[test]
    fn fill_then_stroke_chain() {
        let html = rh(&elem("circle")
            .fill("blue")
            .stroke("white")
            .width(1.0)
            .done());
        assert!(html.contains(r#"fill="blue""#));
        assert!(html.contains(r#"stroke="white""#));
        assert!(html.contains(r#"stroke-width="1""#));
    }

    #[test]
    fn opacity_sets_elem_level_attr() {
        let html = rh(&elem("g").opacity(0.75));
        assert!(html.contains(r#"opacity="0.75""#));
    }

    #[test]
    fn transform_sets_attr() {
        let html = rh(&elem("rect").transform("translate(10,20)"));
        assert!(html.contains(r#"transform="translate(10,20)""#));
    }

    #[test]
    fn class_and_id_set() {
        let html = rh(&elem("circle").class("dot").id("center"));
        assert!(html.contains(r#"class="dot""#));
        assert!(html.contains(r#"id="center""#));
    }

    #[test]
    fn fill_rule_evenodd() {
        let html = rh(&elem("path").fill_rule(FillRule::EvenOdd));
        assert!(html.contains(r#"fill-rule="evenodd""#));
    }

    #[test]
    fn clip_path_produces_url() {
        let html = rh(&elem("circle").clip_path("my-clip"));
        assert!(html.contains(r#"clip-path="url(#my-clip)""#));
    }

    #[test]
    fn stroke_builder_into_component() {
        use crate::view_components::IntoComponent;
        let c = elem("circle").stroke("red");
        let boxed = c.into_component();
        assert!(rh(&*boxed).contains(r#"stroke="red""#));
    }

    #[test]
    fn fill_builder_into_component() {
        use crate::view_components::IntoComponent;
        let c = elem("circle").fill("green");
        let boxed = c.into_component();
        assert!(rh(&*boxed).contains(r#"fill="green""#));
    }
}
