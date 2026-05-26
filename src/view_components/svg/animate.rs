use super::core::{HasSvgLeaf, SvgLeaf, fmt_svg_f64};
use super::path::PathData;
use crate::view_components::{Brick, IntoComponent};

// ── impl_svg_leaf_elem (private to this module for animate types) ─────────────

macro_rules! impl_animate_elem {
    ($T:ty) => {
        impl HasSvgLeaf for $T {
            fn leaf(&self) -> &SvgLeaf {
                &self.leaf
            }
            fn leaf_mut(&mut self) -> &mut SvgLeaf {
                &mut self.leaf
            }
        }
        impl IntoComponent for $T {
            fn into_component(self) -> Box<dyn crate::view_components::Brick> {
                Box::new(self)
            }
        }
    };
}

// ── AnimateTransformType ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum AnimateTransformType {
    Translate,
    Scale,
    Rotate,
    SkewX,
    SkewY,
}

impl AnimateTransformType {
    fn as_str(self) -> &'static str {
        match self {
            AnimateTransformType::Translate => "translate",
            AnimateTransformType::Scale => "scale",
            AnimateTransformType::Rotate => "rotate",
            AnimateTransformType::SkewX => "skewX",
            AnimateTransformType::SkewY => "skewY",
        }
    }
}

// ── AnimateCalcMode ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum CalcMode {
    Discrete,
    Linear,
    Paced,
    Spline,
}

impl CalcMode {
    fn as_str(self) -> &'static str {
        match self {
            CalcMode::Discrete => "discrete",
            CalcMode::Linear => "linear",
            CalcMode::Paced => "paced",
            CalcMode::Spline => "spline",
        }
    }
}

// ── AnimateFill ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum AnimateFill {
    Freeze,
    Remove,
}

impl AnimateFill {
    fn as_str(self) -> &'static str {
        match self {
            AnimateFill::Freeze => "freeze",
            AnimateFill::Remove => "remove",
        }
    }
}

// ── Animate ───────────────────────────────────────────────────────────────────

/// `<animate>` — animates a single attribute on the parent element over time.
///
/// ```rust,ignore
/// svg_circle(50.0, 50.0, 30.0)
///     .animate(
///         animate("cx").from("50").to("150").dur("2s").repeat_count("indefinite")
///     )
/// ```
pub struct Animate {
    leaf: SvgLeaf,
}

pub fn animate(attribute_name: &str) -> Animate {
    let mut leaf = SvgLeaf::new("animate");
    leaf.push_str_attr("attributeName", attribute_name);
    Animate { leaf }
}

impl_animate_elem!(Animate);

impl Animate {
    pub fn from_val(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("from", v);
        self
    }
    pub fn to_val(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("to", v);
        self
    }
    pub fn by(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("by", v);
        self
    }
    pub fn values(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("values", v);
        self
    }
    pub fn dur(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("dur", v);
        self
    }
    pub fn begin(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("begin", v);
        self
    }
    pub fn end(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("end", v);
        self
    }
    pub fn repeat_count(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("repeatCount", v);
        self
    }
    pub fn repeat_dur(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("repeatDur", v);
        self
    }
    pub fn fill_mode(mut self, m: AnimateFill) -> Self {
        self.leaf.push_str_attr("fill", m.as_str());
        self
    }
    pub fn calc_mode(mut self, m: CalcMode) -> Self {
        self.leaf.push_str_attr("calcMode", m.as_str());
        self
    }
    pub fn key_times(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("keyTimes", v);
        self
    }
    pub fn key_splines(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("keySplines", v);
        self
    }
    pub fn additive(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("additive", v);
        self
    }
    pub fn accumulate(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("accumulate", v);
        self
    }
    pub fn attribute_type(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("attributeType", v);
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.leaf.push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── AnimateTransform ──────────────────────────────────────────────────────────

/// `<animateTransform>` — animates the transform attribute of the parent element.
///
/// ```rust,ignore
/// svg_circle(100.0, 100.0, 50.0)
///     .animate_transform(
///         animate_transform(AnimateTransformType::Rotate)
///             .from_val("0 100 100")
///             .to_val("360 100 100")
///             .dur("4s")
///             .repeat_count("indefinite")
///     )
/// ```
pub struct AnimateTransform {
    leaf: SvgLeaf,
}

pub fn animate_transform(transform_type: AnimateTransformType) -> AnimateTransform {
    let mut leaf = SvgLeaf::new("animateTransform");
    leaf.push_str_attr("attributeName", "transform");
    leaf.push_str_attr("type", transform_type.as_str());
    AnimateTransform { leaf }
}

impl_animate_elem!(AnimateTransform);

impl AnimateTransform {
    pub fn from_val(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("from", v);
        self
    }
    pub fn to_val(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("to", v);
        self
    }
    pub fn by(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("by", v);
        self
    }
    pub fn values(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("values", v);
        self
    }
    pub fn dur(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("dur", v);
        self
    }
    pub fn begin(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("begin", v);
        self
    }
    pub fn end(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("end", v);
        self
    }
    pub fn repeat_count(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("repeatCount", v);
        self
    }
    pub fn fill_mode(mut self, m: AnimateFill) -> Self {
        self.leaf.push_str_attr("fill", m.as_str());
        self
    }
    pub fn calc_mode(mut self, m: CalcMode) -> Self {
        self.leaf.push_str_attr("calcMode", m.as_str());
        self
    }
    pub fn key_times(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("keyTimes", v);
        self
    }
    pub fn key_splines(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("keySplines", v);
        self
    }
    pub fn additive(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("additive", v);
        self
    }
    pub fn accumulate(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("accumulate", v);
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.leaf.push_owned_attr(k.to_string(), v.to_string());
        self
    }
}

// ── AnimateMotion ─────────────────────────────────────────────────────────────

/// `<animateMotion>` — moves the parent element along a path over time.
///
/// ```rust,ignore
/// svg_circle(0.0, 0.0, 8.0)
///     .animate_motion(
///         animate_motion()
///             .path(PathData::new().move_to(0.0, 0.0).arc(50.0, 50.0, 0.0, false, true, 100.0, 0.0))
///             .dur("3s")
///             .repeat_count("indefinite")
///             .rotate("auto")
///     )
/// ```
pub struct AnimateMotion {
    leaf: SvgLeaf,
}

pub fn animate_motion() -> AnimateMotion {
    AnimateMotion {
        leaf: SvgLeaf::new("animateMotion"),
    }
}

impl_animate_elem!(AnimateMotion);

impl AnimateMotion {
    pub fn path(mut self, d: PathData) -> Self {
        self.leaf.push_str_attr("path", &d.to_string());
        self
    }
    pub fn path_str(mut self, d: &str) -> Self {
        self.leaf.push_str_attr("path", d);
        self
    }
    pub fn dur(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("dur", v);
        self
    }
    pub fn begin(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("begin", v);
        self
    }
    pub fn end(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("end", v);
        self
    }
    pub fn repeat_count(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("repeatCount", v);
        self
    }
    pub fn fill_mode(mut self, m: AnimateFill) -> Self {
        self.leaf.push_str_attr("fill", m.as_str());
        self
    }
    pub fn rotate(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("rotate", v);
        self
    }
    pub fn key_times(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("keyTimes", v);
        self
    }
    pub fn key_points(mut self, v: &str) -> Self {
        self.leaf.push_str_attr("keyPoints", v);
        self
    }
    pub fn calc_mode(mut self, m: CalcMode) -> Self {
        self.leaf.push_str_attr("calcMode", m.as_str());
        self
    }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        self.leaf.push_owned_attr(k.to_string(), v.to_string());
        self
    }

    /// A child `<mpath>` pointing to a `<path>` element by id.
    pub fn mpath(mut self, href: &str) -> Self {
        let mut mp = SvgLeaf::new("mpath");
        mp.push_str_attr("href", href);
        self.leaf.children.push(Box::new(MPath { leaf: mp }));
        self
    }
}

/// `<mpath>` — references an external `<path>` element as the motion path.
struct MPath {
    leaf: SvgLeaf,
}

impl Brick for Animate {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
    fn attach_listeners(&self) {
        self.leaf.attach_reactive();
    }
}
impl Brick for AnimateTransform {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
    fn attach_listeners(&self) {
        self.leaf.attach_reactive();
    }
}
impl Brick for AnimateMotion {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
    fn attach_listeners(&self) {
        self.leaf.attach_reactive();
    }
}
impl Brick for MPath {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        <SvgLeaf as Brick>::render_into(&self.leaf, parent)
    }
    fn attach_listeners(&self) {
        self.leaf.attach_reactive();
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
    fn animate_renders_attribute_name() {
        let a = animate("cx").from_val("50").to_val("150").dur("2s");
        let html = rh(&a);
        assert!(html.contains(r#"attributeName="cx""#));
        assert!(html.contains(r#"from="50""#));
        assert!(html.contains(r#"to="150""#));
        assert!(html.contains(r#"dur="2s""#));
        assert!(html.ends_with("/>"), "self-closing");
    }

    #[test]
    fn animate_repeat_indefinite() {
        let html = rh(&animate("opacity")
            .values("1;0;1")
            .dur("1s")
            .repeat_count("indefinite"));
        assert!(html.contains(r#"repeatCount="indefinite""#));
    }

    #[test]
    fn animate_transform_renders_type() {
        let html = rh(&animate_transform(AnimateTransformType::Rotate)
            .from_val("0 50 50")
            .to_val("360 50 50")
            .dur("3s"));
        assert!(html.contains(r#"type="rotate""#));
        assert!(html.contains(r#"attributeName="transform""#));
        assert!(html.contains(r#"from="0 50 50""#));
    }

    #[test]
    fn animate_motion_with_path() {
        let html = rh(&animate_motion()
            .path(PathData::new().move_to(0.0, 0.0).line_to(100.0, 100.0))
            .dur("4s")
            .rotate("auto"));
        assert!(html.contains(r#"path="M0,0 L100,100""#));
        assert!(html.contains(r#"rotate="auto""#));
        assert!(html.contains(r#"dur="4s""#));
    }

    #[test]
    fn animate_motion_mpath() {
        let html = rh(&animate_motion().mpath("#my-path").dur("2s"));
        assert!(html.contains("<mpath"));
        assert!(html.contains(r##"href="#my-path""##));
    }

    #[test]
    fn calc_mode_spline() {
        let html = rh(&animate("r")
            .calc_mode(CalcMode::Spline)
            .key_splines("0.4 0 0.6 1"));
        assert!(html.contains(r#"calcMode="spline""#));
        assert!(html.contains(r#"keySplines="0.4 0 0.6 1""#));
    }

    #[test]
    fn fill_mode_freeze() {
        let html = rh(&animate("cy").fill_mode(AnimateFill::Freeze));
        assert!(html.contains(r#"fill="freeze""#));
    }
}
