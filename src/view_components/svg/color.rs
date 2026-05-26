use crate::theme::Color;

/// A reference to a gradient defined in `<defs>` by id.
pub struct GradientRef(pub &'static str);

/// A reference to a pattern defined in `<defs>` by id.
pub struct PatternRef(pub &'static str);

/// SVG color value with named constructors for every form the spec allows.
pub enum SvgColor {
    Hex(String),
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f64),
    Hsl(f64, f64, f64),
    Hsla(f64, f64, f64, f64),
    /// `url(#id)` — references a gradient, pattern, or other paint server.
    Url(String),
    /// `currentColor` — inherits from the CSS `color` property.
    CurrentColor,
    None,
}

impl SvgColor {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        SvgColor::Rgb(r, g, b)
    }
    pub fn rgba(r: u8, g: u8, b: u8, a: f64) -> Self {
        SvgColor::Rgba(r, g, b, a)
    }
    pub fn hsl(h: f64, s: f64, l: f64) -> Self {
        SvgColor::Hsl(h, s, l)
    }
    pub fn hsla(h: f64, s: f64, l: f64, a: f64) -> Self {
        SvgColor::Hsla(h, s, l, a)
    }
    pub fn url(id: &str) -> Self {
        SvgColor::Url(id.to_string())
    }
    pub fn current_color() -> Self {
        SvgColor::CurrentColor
    }
    pub fn none() -> Self {
        SvgColor::None
    }
}

impl std::fmt::Display for SvgColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SvgColor::Hex(s) => write!(f, "{}", s),
            SvgColor::Rgb(r, g, b) => write!(f, "rgb({},{},{})", r, g, b),
            SvgColor::Rgba(r, g, b, a) => write!(f, "rgba({},{},{},{})", r, g, b, a),
            SvgColor::Hsl(h, s, l) => write!(f, "hsl({},{}%,{}%)", h, s, l),
            SvgColor::Hsla(h, s, l, a) => write!(f, "hsla({},{}%,{}%,{})", h, s, l, a),
            SvgColor::Url(id) => write!(f, "url(#{})", id),
            SvgColor::CurrentColor => write!(f, "currentColor"),
            SvgColor::None => write!(f, "none"),
        }
    }
}

/// Anything that can be used as an SVG paint value: color strings, `SvgColor`,
/// theme `Color`, `GradientRef`, `PatternRef`.
pub trait IntoSvgColor {
    fn into_svg_color(self) -> String;
}

impl IntoSvgColor for &str {
    fn into_svg_color(self) -> String {
        self.to_string()
    }
}
impl IntoSvgColor for String {
    fn into_svg_color(self) -> String {
        self
    }
}
impl IntoSvgColor for &String {
    fn into_svg_color(self) -> String {
        self.clone()
    }
}
impl IntoSvgColor for SvgColor {
    fn into_svg_color(self) -> String {
        self.to_string()
    }
}
impl IntoSvgColor for Color {
    fn into_svg_color(self) -> String {
        self.to_hex()
    }
}
impl IntoSvgColor for GradientRef {
    fn into_svg_color(self) -> String {
        format!("url(#{})", self.0)
    }
}
impl IntoSvgColor for PatternRef {
    fn into_svg_color(self) -> String {
        format!("url(#{})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_formats_correctly() {
        assert_eq!(
            SvgColor::rgb(255, 128, 0).into_svg_color(),
            "rgb(255,128,0)"
        );
    }

    #[test]
    fn rgba_formats_correctly() {
        assert_eq!(
            SvgColor::rgba(0, 0, 0, 0.5).into_svg_color(),
            "rgba(0,0,0,0.5)"
        );
    }

    #[test]
    fn hsl_formats_correctly() {
        assert_eq!(
            SvgColor::hsl(200.0, 80.0, 50.0).into_svg_color(),
            "hsl(200,80%,50%)"
        );
    }

    #[test]
    fn url_formats_correctly() {
        assert_eq!(
            SvgColor::url("my-gradient").into_svg_color(),
            "url(#my-gradient)"
        );
    }

    #[test]
    fn current_color() {
        assert_eq!(SvgColor::current_color().into_svg_color(), "currentColor");
    }

    #[test]
    fn none_color() {
        assert_eq!(SvgColor::none().into_svg_color(), "none");
    }

    #[test]
    fn str_passthrough() {
        assert_eq!("steelblue".into_svg_color(), "steelblue");
    }

    #[test]
    fn gradient_ref_produces_url() {
        assert_eq!(GradientRef("grad1").into_svg_color(), "url(#grad1)");
    }

    #[test]
    fn pattern_ref_produces_url() {
        assert_eq!(PatternRef("pat1").into_svg_color(), "url(#pat1)");
    }
}
