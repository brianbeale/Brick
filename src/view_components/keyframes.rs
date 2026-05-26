use crate::theme::{Color, Deg, Em, Pct, Px, Rad, Rem, Turn};
use std::sync::atomic::{AtomicUsize, Ordering};

static KEYFRAME_COUNTER: AtomicUsize = AtomicUsize::new(0);

// ── Unit traits ───────────────────────────────────────────────────────────────

pub trait CssLength {
    fn to_css(&self) -> String;
}
pub trait CssAngle {
    fn to_css(&self) -> String;
}

impl CssLength for Px {
    fn to_css(&self) -> String {
        format!("{}px", self.0)
    }
}
impl CssLength for Rem {
    fn to_css(&self) -> String {
        format!("{}rem", self.0)
    }
}
impl CssLength for Em {
    fn to_css(&self) -> String {
        format!("{}em", self.0)
    }
}
impl CssLength for Pct {
    fn to_css(&self) -> String {
        format!("{}%", self.0)
    }
}

impl CssAngle for Deg {
    fn to_css(&self) -> String {
        format!("{}deg", self.0)
    }
}
impl CssAngle for Rad {
    fn to_css(&self) -> String {
        format!("{}rad", self.0)
    }
}
impl CssAngle for Turn {
    fn to_css(&self) -> String {
        format!("{}turn", self.0)
    }
}

// ── Transform ─────────────────────────────────────────────────────────────────

/// Composable CSS `transform` value. Start with `transform()`.
pub struct Transform {
    pub(crate) fns: Vec<String>,
}

impl Transform {
    pub fn new() -> Self {
        Self { fns: Vec::new() }
    }

    pub fn translate_x(mut self, x: impl CssLength) -> Self {
        self.fns.push(format!("translateX({})", x.to_css()));
        self
    }
    pub fn translate_y(mut self, y: impl CssLength) -> Self {
        self.fns.push(format!("translateY({})", y.to_css()));
        self
    }
    pub fn translate(mut self, x: impl CssLength, y: impl CssLength) -> Self {
        self.fns
            .push(format!("translate({}, {})", x.to_css(), y.to_css()));
        self
    }
    pub fn scale(mut self, v: f32) -> Self {
        self.fns.push(format!("scale({})", v));
        self
    }
    pub fn scale_x(mut self, v: f32) -> Self {
        self.fns.push(format!("scaleX({})", v));
        self
    }
    pub fn scale_y(mut self, v: f32) -> Self {
        self.fns.push(format!("scaleY({})", v));
        self
    }
    pub fn rotate(mut self, angle: impl CssAngle) -> Self {
        self.fns.push(format!("rotate({})", angle.to_css()));
        self
    }
    pub fn skew_x(mut self, angle: impl CssAngle) -> Self {
        self.fns.push(format!("skewX({})", angle.to_css()));
        self
    }
    pub fn skew_y(mut self, angle: impl CssAngle) -> Self {
        self.fns.push(format!("skewY({})", angle.to_css()));
        self
    }

    pub fn to_css(&self) -> String {
        self.fns.join(" ")
    }
}

/// Construct a [`Transform`] builder.
pub fn transform() -> Transform {
    Transform::new()
}

// ── Stop ─────────────────────────────────────────────────────────────────────

/// A single keyframe stop. Start with `stop()`.
///
/// Transform functions (`translate_y`, `rotate`, etc.) are accumulated and
/// emitted as a single `transform:` declaration in the order they are called.
pub struct Stop {
    props: Vec<(&'static str, String)>,
    transform_fns: Vec<String>,
}

impl Stop {
    pub fn new() -> Self {
        Self {
            props: Vec::new(),
            transform_fns: Vec::new(),
        }
    }

    // ── Non-transform properties ──────────────────────────────────────────────

    pub fn opacity(mut self, v: f32) -> Self {
        self.props.push(("opacity", v.to_string()));
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.props.push(("color", c.to_hex()));
        self
    }
    pub fn background_color(mut self, c: Color) -> Self {
        self.props.push(("background-color", c.to_hex()));
        self
    }
    pub fn width(mut self, v: impl CssLength) -> Self {
        self.props.push(("width", v.to_css()));
        self
    }
    pub fn height(mut self, v: impl CssLength) -> Self {
        self.props.push(("height", v.to_css()));
        self
    }
    /// Escape hatch for any property not covered by the typed methods.
    pub fn raw(mut self, prop: &'static str, val: &str) -> Self {
        self.props.push((prop, val.to_string()));
        self
    }

    // ── Transform functions ───────────────────────────────────────────────────

    pub fn translate_x(mut self, x: impl CssLength) -> Self {
        self.transform_fns
            .push(format!("translateX({})", x.to_css()));
        self
    }
    pub fn translate_y(mut self, y: impl CssLength) -> Self {
        self.transform_fns
            .push(format!("translateY({})", y.to_css()));
        self
    }
    pub fn translate(mut self, x: impl CssLength, y: impl CssLength) -> Self {
        self.transform_fns
            .push(format!("translate({}, {})", x.to_css(), y.to_css()));
        self
    }
    pub fn scale(mut self, v: f32) -> Self {
        self.transform_fns.push(format!("scale({})", v));
        self
    }
    pub fn scale_x(mut self, v: f32) -> Self {
        self.transform_fns.push(format!("scaleX({})", v));
        self
    }
    pub fn scale_y(mut self, v: f32) -> Self {
        self.transform_fns.push(format!("scaleY({})", v));
        self
    }
    pub fn rotate(mut self, angle: impl CssAngle) -> Self {
        self.transform_fns
            .push(format!("rotate({})", angle.to_css()));
        self
    }
    pub fn skew_x(mut self, angle: impl CssAngle) -> Self {
        self.transform_fns
            .push(format!("skewX({})", angle.to_css()));
        self
    }
    pub fn skew_y(mut self, angle: impl CssAngle) -> Self {
        self.transform_fns
            .push(format!("skewY({})", angle.to_css()));
        self
    }
    /// Apply a pre-built [`Transform`] value (e.g. one constructed programmatically).
    pub fn apply_transform(mut self, t: Transform) -> Self {
        self.transform_fns.extend(t.fns);
        self
    }

    pub fn to_css(&self) -> String {
        let mut parts: Vec<String> = self
            .props
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();
        if !self.transform_fns.is_empty() {
            parts.push(format!("transform: {}", self.transform_fns.join(" ")));
        }
        parts.join("; ")
    }
}

/// Construct a [`Stop`] builder.
pub fn stop() -> Stop {
    Stop::new()
}

// ── KeyframeName ──────────────────────────────────────────────────────────────

/// A generated, collision-safe animation name returned by `keyframes()` and `keyframes!()`.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeName(String);

impl KeyframeName {
    pub fn generate() -> Self {
        let n = KEYFRAME_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self(format!("brick-kf-{}", n))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for KeyframeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Keyframes ─────────────────────────────────────────────────────────────────

/// Builder for `@keyframes` animation rules. Start with `keyframes()`.
pub struct Keyframes {
    stops: Vec<(u8, String)>,
}

impl Keyframes {
    pub fn new() -> Self {
        Self { stops: Vec::new() }
    }

    pub fn at(mut self, percent: u8, stop: Stop) -> Self {
        self.stops.push((percent.min(100), stop.to_css()));
        self
    }

    pub fn build(&self, name: &str) -> String {
        let stops: String = self
            .stops
            .iter()
            .map(|(pct, props)| format!("  {}% {{ {} }}\n", pct, props))
            .collect();
        format!("@keyframes {} {{\n{}}}", name, stops)
    }

    /// Finalise the definition: inject the `@keyframes` rule and return its generated name.
    pub fn end(self) -> KeyframeName {
        let name = KeyframeName::generate();
        crate::view_components::inject_global_style(&self.build(name.as_str()));
        name
    }
}

/// Construct a [`Keyframes`] builder.
pub fn keyframes() -> Keyframes {
    Keyframes::new()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Color;

    fn name(s: &str) -> KeyframeName {
        KeyframeName(s.to_string())
    }

    #[test]
    fn transform_single_fn() {
        let css = transform().translate_y(Px(-20.0)).to_css();
        assert_eq!(css, "translateY(-20px)");
    }

    #[test]
    fn transform_composed() {
        let css = transform()
            .translate_y(Px(-20.0))
            .rotate(Deg(45.0))
            .to_css();
        assert_eq!(css, "translateY(-20px) rotate(45deg)");
    }

    #[test]
    fn transform_all_lengths() {
        assert_eq!(
            transform().translate_x(Rem(1.5)).to_css(),
            "translateX(1.5rem)"
        );
        assert_eq!(transform().translate_x(Em(2.0)).to_css(), "translateX(2em)");
        assert_eq!(
            transform().translate_x(Pct(50.0)).to_css(),
            "translateX(50%)"
        );
    }

    #[test]
    fn transform_all_angles() {
        assert_eq!(transform().rotate(Rad(1.57)).to_css(), "rotate(1.57rad)");
        assert_eq!(transform().rotate(Turn(0.5)).to_css(), "rotate(0.5turn)");
    }

    #[test]
    fn transform_scale() {
        assert_eq!(transform().scale(1.5).to_css(), "scale(1.5)");
        assert_eq!(
            transform().scale_x(2.0).scale_y(0.5).to_css(),
            "scaleX(2) scaleY(0.5)"
        );
    }

    #[test]
    fn stop_opacity() {
        assert_eq!(stop().opacity(0.0).to_css(), "opacity: 0");
        assert_eq!(stop().opacity(1.0).to_css(), "opacity: 1");
    }

    #[test]
    fn stop_single_transform_fn() {
        let css = stop().translate_y(Px(-20.0)).to_css();
        assert_eq!(css, "transform: translateY(-20px)");
    }

    #[test]
    fn stop_composed_transform() {
        let css = stop().translate_y(Px(-20.0)).rotate(Deg(45.0)).to_css();
        assert_eq!(css, "transform: translateY(-20px) rotate(45deg)");
    }

    #[test]
    fn stop_opacity_and_transform() {
        let css = stop().opacity(0.0).translate_x(Px(-20.0)).to_css();
        assert_eq!(css, "opacity: 0; transform: translateX(-20px)");
    }

    #[test]
    fn stop_apply_transform_programmatic() {
        let t = transform().scale(1.5).rotate(Deg(90.0));
        let css = stop().apply_transform(t).to_css();
        assert_eq!(css, "transform: scale(1.5) rotate(90deg)");
    }

    #[test]
    fn stop_color() {
        let css = stop().color(Color::rgb(255, 0, 0)).to_css();
        assert_eq!(css, "color: #ff0000");
    }

    #[test]
    fn stop_raw_escape_hatch() {
        let css = stop().raw("clip-path", "inset(0 100% 0 0)").to_css();
        assert_eq!(css, "clip-path: inset(0 100% 0 0)");
    }

    #[test]
    fn keyframes_build() {
        let kf = keyframes()
            .at(0, stop().opacity(0.0))
            .at(100, stop().opacity(1.0));
        let css = kf.build("fade");
        assert!(css.starts_with("@keyframes fade {"));
        assert!(css.contains("0% { opacity: 0 }"));
        assert!(css.contains("100% { opacity: 1 }"));
    }

    #[test]
    fn keyframes_inject_returns_unique_names() {
        let a = keyframes().at(0, stop().opacity(0.0)).end();
        let b = keyframes().at(0, stop().opacity(0.0)).end();
        assert!(a.as_str().starts_with("brick-kf-"));
        assert_ne!(a, b);
    }

    #[test]
    fn percent_clamped_to_100() {
        let kf = keyframes().at(255, stop().opacity(1.0));
        let css = kf.build("t");
        assert!(css.contains("100% {"));
        assert!(!css.contains("255%"));
    }

    #[test]
    fn keyframe_name_display_matches_as_str() {
        let n = keyframes().at(0, stop().opacity(0.0)).end();
        assert_eq!(format!("{}", n), n.as_str());
    }
}
