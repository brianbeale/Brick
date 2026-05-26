use super::color::IntoSvgColor;
use super::core::{HasSvgLeaf, IntoSvgF64, SvgContainer, SvgLeaf, fmt_svg_f64};
use super::transform::IntoSvgTransform;
use crate::view_components::IntoComponent;

// ── GradientUnits / SpreadMethod ──────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum GradientUnits {
    UserSpaceOnUse,
    ObjectBoundingBox,
}

impl GradientUnits {
    fn as_str(self) -> &'static str {
        match self {
            GradientUnits::UserSpaceOnUse => "userSpaceOnUse",
            GradientUnits::ObjectBoundingBox => "objectBoundingBox",
        }
    }
}

#[derive(Clone, Copy)]
pub enum SpreadMethod {
    Pad,
    Reflect,
    Repeat,
}

impl SpreadMethod {
    fn as_str(self) -> &'static str {
        match self {
            SpreadMethod::Pad => "pad",
            SpreadMethod::Reflect => "reflect",
            SpreadMethod::Repeat => "repeat",
        }
    }
}

// ── Stop ──────────────────────────────────────────────────────────────────────

/// `<stop>` — a color stop inside a gradient.
pub struct Stop {
    leaf: SvgLeaf,
}

/// Create a gradient stop at `offset` (0.0–1.0) with the given color.
pub fn stop(offset: f64, color: impl IntoSvgColor) -> Stop {
    let mut leaf = SvgLeaf::new("stop");
    leaf.push_str_attr("offset", &fmt_svg_f64(offset));
    leaf.push_str_attr("stop-color", &color.into_svg_color());
    Stop { leaf }
}

impl Stop {
    pub fn opacity(mut self, o: f64) -> Self {
        self.leaf.push_str_attr("stop-opacity", &fmt_svg_f64(o));
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.leaf.push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

impl IntoComponent for Stop {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

// ── LinearGradient ────────────────────────────────────────────────────────────

/// `<linearGradient>` — define a linear gradient in `<defs>`, then reference
/// with `SvgColor::url("my-id")` or `GradientRef("my-id")`.
///
/// ```rust,ignore
/// svg_defs()
///     .child(
///         linear_gradient("sky")
///             .x1(0.0).y1(0.0).x2(0.0).y2(1.0)
///             .stop(stop(0.0, "#87ceeb"))
///             .stop(stop(1.0, "#1e3a5f"))
///     )
/// ```
pub struct LinearGradient {
    container: SvgContainer,
}

pub fn linear_gradient(id: &str) -> LinearGradient {
    let mut container = SvgContainer::new("linearGradient");
    container.leaf.push_str_attr("id", id);
    LinearGradient { container }
}

impl HasSvgLeaf for LinearGradient {
    fn leaf(&self) -> &SvgLeaf {
        &self.container.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.container.leaf
    }
}

impl IntoComponent for LinearGradient {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl LinearGradient {
    pub fn x1(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("x1", v);
        self
    }
    pub fn y1(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("y1", v);
        self
    }
    pub fn x2(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("x2", v);
        self
    }
    pub fn y2(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("y2", v);
        self
    }
    pub fn gradient_units(mut self, u: GradientUnits) -> Self {
        self.container
            .leaf
            .push_str_attr("gradientUnits", u.as_str());
        self
    }
    pub fn gradient_transform(mut self, t: impl IntoSvgTransform) -> Self {
        self.container
            .leaf
            .push_str_attr("gradientTransform", &t.into_svg_transform());
        self
    }
    pub fn spread_method(mut self, m: SpreadMethod) -> Self {
        self.container
            .leaf
            .push_str_attr("spreadMethod", m.as_str());
        self
    }
    /// Reference another gradient's stops via `xlink:href`.
    pub fn href(mut self, id: &str) -> Self {
        self.container.leaf.push_str_attr("href", id);
        self
    }
    pub fn stop(mut self, s: Stop) -> Self {
        self.container.children.push(Box::new(s));
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.container
            .leaf
            .push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── RadialGradient ────────────────────────────────────────────────────────────

/// `<radialGradient>` — a radial gradient. `cx`/`cy`/`r` define the outer circle;
/// `fx`/`fy` define the focal point.
pub struct RadialGradient {
    container: SvgContainer,
}

pub fn radial_gradient(id: &str) -> RadialGradient {
    let mut container = SvgContainer::new("radialGradient");
    container.leaf.push_str_attr("id", id);
    RadialGradient { container }
}

impl HasSvgLeaf for RadialGradient {
    fn leaf(&self) -> &SvgLeaf {
        &self.container.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.container.leaf
    }
}

impl IntoComponent for RadialGradient {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl RadialGradient {
    pub fn cx(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("cx", v);
        self
    }
    pub fn cy(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("cy", v);
        self
    }
    pub fn r(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("r", v);
        self
    }
    pub fn fx(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("fx", v);
        self
    }
    pub fn fy(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("fy", v);
        self
    }
    pub fn fr(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("fr", v);
        self
    }
    pub fn gradient_units(mut self, u: GradientUnits) -> Self {
        self.container
            .leaf
            .push_str_attr("gradientUnits", u.as_str());
        self
    }
    pub fn gradient_transform(mut self, t: impl IntoSvgTransform) -> Self {
        self.container
            .leaf
            .push_str_attr("gradientTransform", &t.into_svg_transform());
        self
    }
    pub fn spread_method(mut self, m: SpreadMethod) -> Self {
        self.container
            .leaf
            .push_str_attr("spreadMethod", m.as_str());
        self
    }
    pub fn href(mut self, id: &str) -> Self {
        self.container.leaf.push_str_attr("href", id);
        self
    }
    pub fn stop(mut self, s: Stop) -> Self {
        self.container.children.push(Box::new(s));
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.container
            .leaf
            .push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── ClipPath ──────────────────────────────────────────────────────────────────

/// `<clipPath>` — defines a clipping region. Reference on shapes with `.clip_path("id")`.
pub struct ClipPath {
    container: SvgContainer,
}

pub fn clip_path(id: &str) -> ClipPath {
    let mut container = SvgContainer::new("clipPath");
    container.leaf.push_str_attr("id", id);
    ClipPath { container }
}

impl HasSvgLeaf for ClipPath {
    fn leaf(&self) -> &SvgLeaf {
        &self.container.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.container.leaf
    }
}

impl IntoComponent for ClipPath {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl ClipPath {
    pub fn child(mut self, c: impl IntoComponent) -> Self {
        self.container.children.push(c.into_component());
        self
    }
    pub fn clip_path_units(mut self, v: GradientUnits) -> Self {
        self.container
            .leaf
            .push_str_attr("clipPathUnits", v.as_str());
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.container
            .leaf
            .push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── Mask ──────────────────────────────────────────────────────────────────────

/// `<mask>` — defines a luminance or alpha mask. Reference on shapes with `.mask_ref("id")`.
pub struct Mask {
    container: SvgContainer,
}

pub fn svg_mask(id: &str) -> Mask {
    let mut container = SvgContainer::new("mask");
    container.leaf.push_str_attr("id", id);
    Mask { container }
}

impl HasSvgLeaf for Mask {
    fn leaf(&self) -> &SvgLeaf {
        &self.container.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.container.leaf
    }
}

impl IntoComponent for Mask {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl Mask {
    pub fn child(mut self, c: impl IntoComponent) -> Self {
        self.container.children.push(c.into_component());
        self
    }
    pub fn x(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("x", v);
        self
    }
    pub fn y(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("y", v);
        self
    }
    pub fn width(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("width", v);
        self
    }
    pub fn height(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("height", v);
        self
    }
    pub fn mask_units(mut self, v: GradientUnits) -> Self {
        self.container.leaf.push_str_attr("maskUnits", v.as_str());
        self
    }
    pub fn mask_content_units(mut self, v: GradientUnits) -> Self {
        self.container
            .leaf
            .push_str_attr("maskContentUnits", v.as_str());
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.container
            .leaf
            .push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── Pattern ───────────────────────────────────────────────────────────────────

/// `<pattern>` — a tiling pattern fill. Reference on shapes with `SvgColor::url("id")`.
pub struct Pattern {
    container: SvgContainer,
}

pub fn svg_pattern(id: &str) -> Pattern {
    let mut container = SvgContainer::new("pattern");
    container.leaf.push_str_attr("id", id);
    Pattern { container }
}

impl HasSvgLeaf for Pattern {
    fn leaf(&self) -> &SvgLeaf {
        &self.container.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.container.leaf
    }
}

impl IntoComponent for Pattern {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl Pattern {
    pub fn child(mut self, c: impl IntoComponent) -> Self {
        self.container.children.push(c.into_component());
        self
    }
    pub fn x(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("x", v);
        self
    }
    pub fn y(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("y", v);
        self
    }
    pub fn width(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("width", v);
        self
    }
    pub fn height(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("height", v);
        self
    }
    pub fn pattern_units(mut self, v: GradientUnits) -> Self {
        self.container
            .leaf
            .push_str_attr("patternUnits", v.as_str());
        self
    }
    pub fn pattern_content_units(mut self, v: GradientUnits) -> Self {
        self.container
            .leaf
            .push_str_attr("patternContentUnits", v.as_str());
        self
    }
    pub fn pattern_transform(mut self, t: impl IntoSvgTransform) -> Self {
        self.container
            .leaf
            .push_str_attr("patternTransform", &t.into_svg_transform());
        self
    }
    pub fn view_box(mut self, min_x: f64, min_y: f64, w: f64, h: f64) -> Self {
        let v = format!(
            "{} {} {} {}",
            fmt_svg_f64(min_x),
            fmt_svg_f64(min_y),
            fmt_svg_f64(w),
            fmt_svg_f64(h),
        );
        self.container
            .leaf
            .push_owned_attr("viewBox".to_string(), v);
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.container
            .leaf
            .push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── Filter ────────────────────────────────────────────────────────────────────

/// `<filter>` — an SVG filter effect container. Reference on shapes with `.filter("id")`.
///
/// ```rust,ignore
/// svg_defs()
///     .child(svg_filter("blur").child(fe_gaussian_blur(4.0)))
/// ```
pub struct Filter {
    container: SvgContainer,
}

pub fn svg_filter(id: &str) -> Filter {
    let mut container = SvgContainer::new("filter");
    container.leaf.push_str_attr("id", id);
    Filter { container }
}

impl HasSvgLeaf for Filter {
    fn leaf(&self) -> &SvgLeaf {
        &self.container.leaf
    }
    fn leaf_mut(&mut self) -> &mut SvgLeaf {
        &mut self.container.leaf
    }
}

impl IntoComponent for Filter {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl Filter {
    pub fn child(mut self, c: impl IntoComponent) -> Self {
        self.container.children.push(c.into_component());
        self
    }
    pub fn x(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("x", v);
        self
    }
    pub fn y(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("y", v);
        self
    }
    pub fn width(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("width", v);
        self
    }
    pub fn height(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("height", v);
        self
    }
    pub fn color_interpolation_filters(mut self, v: &str) -> Self {
        self.container
            .leaf
            .push_str_attr("color-interpolation-filters", v);
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.container
            .leaf
            .push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── Filter primitives ─────────────────────────────────────────────────────────

/// A raw SVG filter primitive — use `fe_*` constructor functions.
pub struct FePrimitive {
    leaf: SvgLeaf,
}

impl IntoComponent for FePrimitive {
    fn into_component(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(self)
    }
}

impl FePrimitive {
    fn new(tag: &'static str) -> Self {
        FePrimitive {
            leaf: SvgLeaf::new(tag),
        }
    }

    pub fn result(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("result", v);
        self
    }
    pub fn in1(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("in", v);
        self
    }
    pub fn in2(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("in2", v);
        self
    }
    pub fn x(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("x", v);
        self
    }
    pub fn y(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("y", v);
        self
    }
    pub fn width(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("width", v);
        self
    }
    pub fn height(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("height", v);
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.leaf.push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

/// `<feGaussianBlur>` — Gaussian blur filter primitive.
pub fn fe_gaussian_blur(std_deviation: f64) -> FePrimitive {
    let mut p = FePrimitive::new("feGaussianBlur");
    p.leaf
        .push_str_attr("stdDeviation", &fmt_svg_f64(std_deviation));
    p
}

/// `<feColorMatrix>` — color matrix transformation.
pub fn fe_color_matrix(type_val: &str, values: &str) -> FePrimitive {
    let mut p = FePrimitive::new("feColorMatrix");
    p.leaf.push_str_attr("type", type_val);
    p.leaf.push_str_attr("values", values);
    p
}

/// `<feBlend>` — blends two image inputs together.
pub fn fe_blend(mode: &str) -> FePrimitive {
    let mut p = FePrimitive::new("feBlend");
    p.leaf.push_str_attr("mode", mode);
    p
}

/// `<feComposite>` — composites two image inputs.
pub fn fe_composite(operator: &str) -> FePrimitive {
    let mut p = FePrimitive::new("feComposite");
    p.leaf.push_str_attr("operator", operator);
    p
}

/// `<feOffset>` — shifts an image.
pub fn fe_offset(dx: f64, dy: f64) -> FePrimitive {
    let mut p = FePrimitive::new("feOffset");
    p.leaf.push_str_attr("dx", &fmt_svg_f64(dx));
    p.leaf.push_str_attr("dy", &fmt_svg_f64(dy));
    p
}

/// `<feMerge>` — merges multiple filter results.
pub fn fe_merge() -> FePrimitive {
    FePrimitive::new("feMerge")
}

/// `<feMergeNode>` — a node inside `<feMerge>`.
pub fn fe_merge_node(in_val: &str) -> FePrimitive {
    let mut p = FePrimitive::new("feMergeNode");
    p.leaf.push_str_attr("in", in_val);
    p
}

/// `<feFlood>` — fills a region with a color.
pub fn fe_flood(color: impl IntoSvgColor, opacity: f64) -> FePrimitive {
    let mut p = FePrimitive::new("feFlood");
    p.leaf.push_str_attr("flood-color", &color.into_svg_color());
    p.leaf.push_str_attr("flood-opacity", &fmt_svg_f64(opacity));
    p
}

/// `<feTurbulence>` — generates noise textures.
pub fn fe_turbulence(base_frequency: f64, num_octaves: u32) -> FePrimitive {
    let mut p = FePrimitive::new("feTurbulence");
    p.leaf
        .push_str_attr("baseFrequency", &fmt_svg_f64(base_frequency));
    p.leaf.push_str_attr("numOctaves", &num_octaves.to_string());
    p
}

/// `<feDisplacementMap>` — displaces pixels using another image as a map.
pub fn fe_displacement_map(scale: f64) -> FePrimitive {
    let mut p = FePrimitive::new("feDisplacementMap");
    p.leaf.push_str_attr("scale", &fmt_svg_f64(scale));
    p
}

// ── Phase 10: Brick bridge impls ────────────────────────────────────

use crate::view_components::Brick;

impl Brick for Stop {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
}
impl Brick for LinearGradient {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgContainer as Brick>::render_into(&self.container, parent)
    }
    fn attach_listeners(&self) {
        self.container.attach();
    }
}
impl Brick for RadialGradient {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgContainer as Brick>::render_into(&self.container, parent)
    }
    fn attach_listeners(&self) {
        self.container.attach();
    }
}
impl Brick for ClipPath {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgContainer as Brick>::render_into(&self.container, parent)
    }
    fn attach_listeners(&self) {
        self.container.attach();
    }
}
impl Brick for Mask {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgContainer as Brick>::render_into(&self.container, parent)
    }
    fn attach_listeners(&self) {
        self.container.attach();
    }
}
impl Brick for Pattern {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgContainer as Brick>::render_into(&self.container, parent)
    }
    fn attach_listeners(&self) {
        self.container.attach();
    }
}
impl Brick for Filter {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgContainer as Brick>::render_into(&self.container, parent)
    }
    fn attach_listeners(&self) {
        self.container.attach();
    }
}
impl Brick for FePrimitive {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn stop_renders() {
        let html = rh(&stop(0.0, "red"));
        assert!(html.contains(r#"offset="0""#));
        assert!(html.contains(r#"stop-color="red""#));
        assert!(html.ends_with("/>"));
    }

    #[test]
    fn stop_with_opacity() {
        let html = rh(&stop(1.0, "blue").opacity(0.5));
        assert!(html.contains(r#"stop-opacity="0.5""#));
    }

    #[test]
    fn linear_gradient_renders() {
        let html = rh(&linear_gradient("grad1")
            .x1(0.0)
            .y1(0.0)
            .x2(1.0)
            .y2(0.0)
            .stop(stop(0.0, "white"))
            .stop(stop(1.0, "black")));
        assert!(html.contains(r#"id="grad1""#));
        assert!(html.contains(r#"x1="0""#));
        assert!(html.contains(r#"x2="1""#));
        assert!(html.contains("<stop"));
        assert!(html.ends_with("</linearGradient>"));
    }

    #[test]
    fn radial_gradient_renders() {
        let html = rh(&radial_gradient("radial1")
            .cx(0.5)
            .cy(0.5)
            .r(0.5)
            .stop(stop(0.0, "yellow")));
        assert!(html.contains(r#"id="radial1""#));
        assert!(html.contains(r#"cx="0.5""#));
    }

    #[test]
    fn gradient_units_object_bounding_box() {
        let html = rh(&linear_gradient("g").gradient_units(GradientUnits::ObjectBoundingBox));
        assert!(html.contains(r#"gradientUnits="objectBoundingBox""#));
    }

    #[test]
    fn spread_method_reflect() {
        let html = rh(&radial_gradient("r").spread_method(SpreadMethod::Reflect));
        assert!(html.contains(r#"spreadMethod="reflect""#));
    }

    #[test]
    fn clip_path_renders() {
        use super::super::shapes::svg_rect;
        let html = rh(&clip_path("my-clip").child(svg_rect(0.0, 0.0, 100.0, 100.0)));
        assert!(html.contains(r#"id="my-clip""#));
        assert!(html.contains("<rect"));
        assert!(html.ends_with("</clipPath>"));
    }

    #[test]
    fn mask_renders() {
        use super::super::shapes::svg_rect;
        let html = rh(&svg_mask("my-mask").child(svg_rect(0.0, 0.0, 200.0, 200.0)));
        assert!(html.contains(r#"id="my-mask""#));
        assert!(html.ends_with("</mask>"));
    }

    #[test]
    fn pattern_renders() {
        use super::super::shapes::svg_circle;
        let html = rh(&svg_pattern("polka")
            .width(20.0)
            .height(20.0)
            .child(svg_circle(10.0, 10.0, 8.0)));
        assert!(html.contains(r#"id="polka""#));
        assert!(html.contains(r#"width="20""#));
        assert!(html.contains("<circle"));
    }

    #[test]
    fn filter_renders() {
        let html = rh(&svg_filter("blur-4").child(fe_gaussian_blur(4.0)));
        assert!(html.contains(r#"id="blur-4""#));
        assert!(html.contains(r#"stdDeviation="4""#));
        assert!(html.ends_with("</filter>"));
    }

    #[test]
    fn fe_color_matrix_renders() {
        let html = rh(&fe_color_matrix("saturate", "0"));
        assert!(html.contains(r#"type="saturate""#));
        assert!(html.contains(r#"values="0""#));
    }

    #[test]
    fn fe_offset_renders() {
        let html = rh(&fe_offset(3.0, 5.0));
        assert!(html.contains(r#"dx="3""#));
        assert!(html.contains(r#"dy="5""#));
    }

    #[test]
    fn drop_shadow_filter_composition() {
        let html = rh(&svg_filter("shadow")
            .child(fe_offset(2.0, 2.0).result("offset"))
            .child(fe_gaussian_blur(2.0).in1("offset").result("blur"))
            .child(fe_merge()));
        assert!(html.contains("<feOffset"));
        assert!(html.contains("<feGaussianBlur"));
        assert!(html.contains("<feMerge"));
    }
}
