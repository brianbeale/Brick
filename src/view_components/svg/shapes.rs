use super::core::{HasSvgLeaf, IntoSvgF64, SvgLeaf};
use super::path::PathData;
use crate::view_components::IntoComponent;

// ── Shared macro ──────────────────────────────────────────────────────────────

macro_rules! impl_svg_leaf_elem {
    ($T:ty) => {
        impl HasSvgLeaf for $T {
            fn leaf(&self) -> &SvgLeaf {
                &self.leaf
            }
            fn leaf_mut(&mut self) -> &mut SvgLeaf {
                &mut self.leaf
            }
        }
        impl crate::view_components::Brick for $T {
            fn attach_listeners(&self) {
                self.leaf.attach_reactive();
            }
            fn render_into(&self, parent: &crate::renderer::BrickNode) {
                <SvgLeaf as crate::view_components::Brick>::render_into(&self.leaf, parent)
            }
        }
        impl IntoComponent for $T {
            fn into_component(self) -> Box<dyn crate::view_components::Brick> {
                Box::new(self)
            }
        }
    };
}

// ── SvgCircle ─────────────────────────────────────────────────────────────────

/// `<circle>` — circle centered at `(cx, cy)` with radius `r`.
pub struct SvgCircle {
    leaf: SvgLeaf,
}

/// Create a `<circle cx="{cx}" cy="{cy}" r="{r}"/>` element. All parameters
/// accept `f64`, `f32`, `i32`, `usize`, or `Signal<f64>` for reactive geometry.
pub fn svg_circle(cx: impl IntoSvgF64, cy: impl IntoSvgF64, r: impl IntoSvgF64) -> SvgCircle {
    let mut leaf = SvgLeaf::new("circle");
    leaf.push_f64("cx", cx);
    leaf.push_f64("cy", cy);
    leaf.push_f64("r", r);
    SvgCircle { leaf }
}

impl_svg_leaf_elem!(SvgCircle);

// ── SvgRect ───────────────────────────────────────────────────────────────────

/// `<rect>` — axis-aligned rectangle.
pub struct SvgRect {
    leaf: SvgLeaf,
}

/// Create a `<rect x y width height/>` element.
pub fn svg_rect(
    x: impl IntoSvgF64,
    y: impl IntoSvgF64,
    width: impl IntoSvgF64,
    height: impl IntoSvgF64,
) -> SvgRect {
    let mut leaf = SvgLeaf::new("rect");
    leaf.push_f64("x", x);
    leaf.push_f64("y", y);
    leaf.push_f64("width", width);
    leaf.push_f64("height", height);
    SvgRect { leaf }
}

impl_svg_leaf_elem!(SvgRect);

impl SvgRect {
    /// Horizontal corner radius.
    pub fn rx(mut self, r: impl IntoSvgF64) -> Self {
        self.leaf.push_f64("rx", r);
        self
    }
    /// Vertical corner radius.
    pub fn ry(mut self, r: impl IntoSvgF64) -> Self {
        self.leaf.push_f64("ry", r);
        self
    }
    /// Set both `rx` and `ry` to the same value for uniform rounded corners.
    pub fn corner_radius(mut self, r: f64) -> Self {
        self.leaf.push_f64("rx", r);
        self.leaf.push_f64("ry", r);
        self
    }
}

// ── SvgEllipse ────────────────────────────────────────────────────────────────

/// `<ellipse>` — ellipse centered at `(cx, cy)` with radii `rx` and `ry`.
pub struct SvgEllipse {
    leaf: SvgLeaf,
}

pub fn svg_ellipse(
    cx: impl IntoSvgF64,
    cy: impl IntoSvgF64,
    rx: impl IntoSvgF64,
    ry: impl IntoSvgF64,
) -> SvgEllipse {
    let mut leaf = SvgLeaf::new("ellipse");
    leaf.push_f64("cx", cx);
    leaf.push_f64("cy", cy);
    leaf.push_f64("rx", rx);
    leaf.push_f64("ry", ry);
    SvgEllipse { leaf }
}

impl_svg_leaf_elem!(SvgEllipse);

// ── SvgLine ───────────────────────────────────────────────────────────────────

/// `<line>` — straight line from `(x1, y1)` to `(x2, y2)`.
pub struct SvgLine {
    leaf: SvgLeaf,
}

pub fn svg_line(
    x1: impl IntoSvgF64,
    y1: impl IntoSvgF64,
    x2: impl IntoSvgF64,
    y2: impl IntoSvgF64,
) -> SvgLine {
    let mut leaf = SvgLeaf::new("line");
    leaf.push_f64("x1", x1);
    leaf.push_f64("y1", y1);
    leaf.push_f64("x2", x2);
    leaf.push_f64("y2", y2);
    SvgLine { leaf }
}

impl_svg_leaf_elem!(SvgLine);

// ── SvgPolyline ───────────────────────────────────────────────────────────────

/// `<polyline>` — an open polygon through a series of points.
pub struct SvgPolyline {
    leaf: SvgLeaf,
}

/// Create a `<polyline>` from a `points` string (e.g. `"0,0 100,50 200,0"`) or
/// from a slice of `(x, y)` tuples via `svg_polyline_pts`.
pub fn svg_polyline(points: &str) -> SvgPolyline {
    let mut leaf = SvgLeaf::new("polyline");
    leaf.push_str_attr("points", points);
    SvgPolyline { leaf }
}

/// Create a `<polyline>` from a slice of `(x, y)` coordinate pairs.
pub fn svg_polyline_pts(pts: &[(f64, f64)]) -> SvgPolyline {
    svg_polyline(&pts_to_string(pts))
}

impl_svg_leaf_elem!(SvgPolyline);

// ── SvgPolygon ────────────────────────────────────────────────────────────────

/// `<polygon>` — a closed polygon through a series of points.
pub struct SvgPolygon {
    leaf: SvgLeaf,
}

pub fn svg_polygon(points: &str) -> SvgPolygon {
    let mut leaf = SvgLeaf::new("polygon");
    leaf.push_str_attr("points", points);
    SvgPolygon { leaf }
}

/// Create a `<polygon>` from a slice of `(x, y)` coordinate pairs.
pub fn svg_polygon_pts(pts: &[(f64, f64)]) -> SvgPolygon {
    svg_polygon(&pts_to_string(pts))
}

impl_svg_leaf_elem!(SvgPolygon);

// ── SvgPath ───────────────────────────────────────────────────────────────────

/// `<path>` — arbitrary SVG path defined by a `PathData` builder or raw `d` string.
pub struct SvgPath {
    leaf: SvgLeaf,
}

pub fn svg_path(d: PathData) -> SvgPath {
    let mut leaf = SvgLeaf::new("path");
    leaf.push_str_attr("d", &d.to_string());
    SvgPath { leaf }
}

pub fn svg_path_str(d: &str) -> SvgPath {
    let mut leaf = SvgLeaf::new("path");
    leaf.push_str_attr("d", d);
    SvgPath { leaf }
}

impl_svg_leaf_elem!(SvgPath);

impl SvgPath {
    /// Replace the `d` attribute with a new `PathData`.
    pub fn d(mut self, d: PathData) -> Self {
        // Remove any existing "d" entry and push the new one.
        self.leaf.attrs.retain(|(k, _)| k != "d");
        self.leaf.push_str_attr("d", &d.to_string());
        self
    }
}

// ── Points helper ─────────────────────────────────────────────────────────────

fn pts_to_string(pts: &[(f64, f64)]) -> String {
    use super::core::fmt_svg_f64;
    pts.iter()
        .map(|(x, y)| format!("{},{}", fmt_svg_f64(*x), fmt_svg_f64(*y)))
        .collect::<Vec<_>>()
        .join(" ")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::presentation::SvgElemMethods;
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn svg_circle_renders() {
        let html = rh(&svg_circle(50.0, 50.0, 30.0));
        assert_eq!(html, r#"<circle cx="50" cy="50" r="30"/>"#);
    }

    #[test]
    fn svg_circle_float_coords() {
        let html = rh(&svg_circle(10.5, 20.25, 5.0));
        assert!(html.contains(r#"cx="10.5""#));
        assert!(html.contains(r#"cy="20.25""#));
    }

    #[test]
    fn svg_circle_with_fill() {
        let html = rh(&svg_circle(50.0, 50.0, 30.0).fill("steelblue").done());
        assert!(html.contains(r#"fill="steelblue""#));
    }

    #[test]
    fn svg_circle_with_stroke_chain() {
        let html = rh(&svg_circle(50.0, 50.0, 30.0)
            .stroke("black")
            .width(2.0)
            .done());
        assert!(html.contains(r#"stroke="black""#));
        assert!(html.contains(r#"stroke-width="2""#));
    }

    #[test]
    fn svg_rect_renders() {
        let html = rh(&svg_rect(10.0, 20.0, 100.0, 50.0));
        assert_eq!(html, r#"<rect x="10" y="20" width="100" height="50"/>"#);
    }

    #[test]
    fn svg_rect_corner_radius() {
        let html = rh(&svg_rect(0.0, 0.0, 100.0, 100.0).corner_radius(8.0));
        assert!(html.contains(r#"rx="8""#));
        assert!(html.contains(r#"ry="8""#));
    }

    #[test]
    fn svg_ellipse_renders() {
        let html = rh(&svg_ellipse(100.0, 50.0, 80.0, 30.0));
        assert_eq!(html, r#"<ellipse cx="100" cy="50" rx="80" ry="30"/>"#);
    }

    #[test]
    fn svg_line_renders() {
        let html = rh(&svg_line(0.0, 0.0, 100.0, 100.0));
        assert_eq!(html, r#"<line x1="0" y1="0" x2="100" y2="100"/>"#);
    }

    #[test]
    fn svg_polyline_renders() {
        let html = rh(&svg_polyline("0,0 50,100 100,0"));
        assert!(html.contains(r#"points="0,0 50,100 100,0""#));
    }

    #[test]
    fn svg_polyline_pts_converts() {
        let html = rh(&svg_polyline_pts(&[
            (0.0, 0.0),
            (50.0, 100.0),
            (100.0, 0.0),
        ]));
        assert!(html.contains(r#"points="0,0 50,100 100,0""#));
    }

    #[test]
    fn svg_polygon_renders() {
        let html = rh(&svg_polygon("0,0 100,0 50,100"));
        assert!(html.starts_with("<polygon"));
        assert!(html.contains(r#"points="0,0 100,0 50,100""#));
    }

    #[test]
    fn svg_path_from_builder() {
        let d = PathData::new()
            .move_to(10.0, 20.0)
            .line_to(100.0, 20.0)
            .close();
        let html = rh(&svg_path(d));
        assert!(html.contains(r#"d="M10,20 L100,20 Z""#));
    }

    #[test]
    fn svg_path_from_str() {
        let html = rh(&svg_path_str("M0,0 L100,100"));
        assert!(html.contains(r#"d="M0,0 L100,100""#));
    }

    #[test]
    fn svg_circle_into_component() {
        let c = svg_circle(10.0, 10.0, 5.0).into_component();
        assert!(rh(&*c).contains(r#"r="5""#));
    }

    #[test]
    fn svg_circle_reactive_geometry() {
        use crate::state_mgmt::Signal;
        let cx = Signal::new(50.0f64);
        let html = rh(&svg_circle(cx, 50.0, 30.0));
        assert!(html.contains(r#"cx="50""#));
        assert!(
            html.contains("brick-svg-"),
            "identity class injected for reactive attr"
        );
    }

    #[test]
    fn svg_circle_transform() {
        use super::super::transform::Transform;
        let html = rh(&svg_circle(50.0, 50.0, 20.0).transform(Transform::translate(10.0, 0.0)));
        assert!(html.contains(r#"transform="translate(10,0)""#));
    }

    #[test]
    fn svg_path_d_replaces() {
        let html = rh(&svg_path(PathData::new().move_to(0.0, 0.0))
            .d(PathData::new().move_to(10.0, 10.0).line_to(50.0, 50.0)));
        assert!(html.contains(r#"d="M10,10 L50,50""#));
        assert!(!html.contains("M0,0"), "old d replaced");
    }
}
