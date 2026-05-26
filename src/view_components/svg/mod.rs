//! Brick SVG — a complete, type-safe SVG authoring library.
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use brick::prelude::*;
//!
//! let graphic = svg_root("400", "300")
//!     .view_box(0.0, 0.0, 400.0, 300.0)
//!     .child(
//!         svg_defs()
//!             .child(linear_gradient("sky").x1(0.0).y1(0.0).x2(0.0).y2(1.0)
//!                 .stop(stop(0.0, "#87ceeb"))
//!                 .stop(stop(1.0, "#1e3a5f")))
//!     )
//!     .child(
//!         svg_rect(0.0, 0.0, 400.0, 300.0).fill(GradientRef("sky"))
//!     )
//!     .child(
//!         svg_circle(200.0, 150.0, 80.0)
//!             .fill("white")
//!             .stroke("steelblue").width(3.0).linecap(Linecap::Round)
//!             .transform(Transform::translate(0.0, 0.0))
//!     )
//!     .child(
//!         svg_text(200.0, 155.0)
//!             .content("Hello")
//!             .text_anchor("middle")
//!             .dominant_baseline("central")
//!             .font_size("24px")
//!             .fill("steelblue")
//!     );
//! ```

pub mod animate;
pub mod color;
pub mod containers;
pub mod core;
pub mod effects;
pub mod path;
pub mod presentation;
pub mod shapes;
pub mod text;
pub mod transform;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use animate::{
    Animate, AnimateFill, AnimateMotion, AnimateTransform, AnimateTransformType, CalcMode, animate,
    animate_motion, animate_transform,
};
pub use color::{GradientRef, IntoSvgColor, PatternRef, SvgColor};
pub use containers::{
    SvgDefs, SvgGroup, SvgMarker, SvgRoot, SvgSymbol, SvgUse, svg_defs, svg_group, svg_marker,
    svg_root, svg_symbol, svg_use,
};
pub use core::{HasSvgLeaf, IntoSvgF64};
pub use effects::{
    ClipPath, FePrimitive, Filter, GradientUnits, LinearGradient, Mask, Pattern, RadialGradient,
    SpreadMethod, Stop, clip_path, fe_blend, fe_color_matrix, fe_composite, fe_displacement_map,
    fe_flood, fe_gaussian_blur, fe_merge, fe_merge_node, fe_offset, fe_turbulence, linear_gradient,
    radial_gradient, stop, svg_filter, svg_mask, svg_pattern,
};
pub use path::PathData;
pub use presentation::{FillBuilder, FillRule, Linecap, Linejoin, StrokeBuilder, SvgElemMethods};
pub use shapes::{
    SvgCircle, SvgEllipse, SvgLine, SvgPath, SvgPolygon, SvgPolyline, SvgRect, svg_circle,
    svg_ellipse, svg_line, svg_path, svg_path_str, svg_polygon, svg_polygon_pts, svg_polyline,
    svg_polyline_pts, svg_rect,
};
pub use text::{SvgText, SvgTspan, svg_text, svg_text_str, svg_tspan};
pub use transform::{IntoSvgTransform, Transform};

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Escape special characters in SVG text content.
pub(crate) fn escape_svg_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_svg_text_replaces_specials() {
        assert_eq!(escape_svg_text("<>&\"'"), "&lt;&gt;&amp;&quot;&#39;");
    }

    #[test]
    fn escape_svg_text_noop_on_plain() {
        assert_eq!(escape_svg_text("Hello world"), "Hello world");
    }
}
