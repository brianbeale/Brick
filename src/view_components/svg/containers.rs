use super::core::{HasSvgLeaf, IntoSvgF64, SvgContainer, SvgLeaf, fmt_svg_f64};
use crate::view_components::IntoComponent;

// ── Shared macro ──────────────────────────────────────────────────────────────

macro_rules! impl_svg_container_elem {
    ($T:ty) => {
        impl HasSvgLeaf for $T {
            fn leaf(&self) -> &SvgLeaf {
                &self.container.leaf
            }
            fn leaf_mut(&mut self) -> &mut SvgLeaf {
                &mut self.container.leaf
            }
        }
        impl crate::view_components::Brick for $T {
            fn render_into(&self, parent: &crate::renderer::BrickNode) {
                <SvgContainer as crate::view_components::Brick>::render_into(
                    &self.container,
                    parent,
                )
            }
            fn attach_listeners(&self) {
                self.container.attach();
            }
        }
        impl IntoComponent for $T {
            fn into_component(self) -> Box<dyn crate::view_components::Brick> {
                Box::new(self)
            }
        }
        impl $T {
            /// Append a child element.
            pub fn child(mut self, c: impl IntoComponent) -> Self {
                self.container.children.push(c.into_component());
                self
            }
        }
    };
}

// ── SvgRoot ───────────────────────────────────────────────────────────────────

/// `<svg>` root element with `xmlns` set automatically.
///
/// ```rust,ignore
/// svg_root("400", "300")
///     .view_box(0.0, 0.0, 400.0, 300.0)
///     .child(svg_circle(200.0, 150.0, 100.0).fill("steelblue"))
/// ```
pub struct SvgRoot {
    container: SvgContainer,
}

/// Create the root `<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">`.
/// Width and height accept any string (`"400"`, `"100%"`, `"auto"`).
pub fn svg_root(width: &str, height: &str) -> SvgRoot {
    let mut container = SvgContainer::new("svg");
    container
        .leaf
        .push_str_attr("xmlns", "http://www.w3.org/2000/svg");
    container.leaf.push_str_attr("width", width);
    container.leaf.push_str_attr("height", height);
    SvgRoot { container }
}

impl_svg_container_elem!(SvgRoot);

impl SvgRoot {
    /// Set the `viewBox` attribute from four floats.
    pub fn view_box(mut self, min_x: f64, min_y: f64, width: f64, height: f64) -> Self {
        let v = format!(
            "{} {} {} {}",
            fmt_svg_f64(min_x),
            fmt_svg_f64(min_y),
            fmt_svg_f64(width),
            fmt_svg_f64(height),
        );
        self.container
            .leaf
            .push_owned_attr("viewBox".to_string(), v);
        self
    }

    pub fn preserve_aspect_ratio(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("preserveAspectRatio", v);
        self
    }
}

// ── SvgGroup ──────────────────────────────────────────────────────────────────

/// `<g>` — group element. Apply shared transform or presentation attrs to
/// all children at once.
pub struct SvgGroup {
    container: SvgContainer,
}

pub fn svg_group() -> SvgGroup {
    SvgGroup {
        container: SvgContainer::new("g"),
    }
}

impl_svg_container_elem!(SvgGroup);

// ── SvgDefs ───────────────────────────────────────────────────────────────────

/// `<defs>` — non-rendered definitions container. Place gradients, clip paths,
/// filters, patterns, and symbols here and reference them by id.
pub struct SvgDefs {
    container: SvgContainer,
}

pub fn svg_defs() -> SvgDefs {
    SvgDefs {
        container: SvgContainer::new("defs"),
    }
}

impl_svg_container_elem!(SvgDefs);

// ── SvgSymbol ─────────────────────────────────────────────────────────────────

/// `<symbol>` — reusable SVG fragment referenced by `<use>`. Define in `<defs>`.
pub struct SvgSymbol {
    container: SvgContainer,
}

pub fn svg_symbol(id: &str) -> SvgSymbol {
    let mut container = SvgContainer::new("symbol");
    container.leaf.push_str_attr("id", id);
    SvgSymbol { container }
}

impl_svg_container_elem!(SvgSymbol);

impl SvgSymbol {
    pub fn view_box(mut self, min_x: f64, min_y: f64, width: f64, height: f64) -> Self {
        let v = format!(
            "{} {} {} {}",
            fmt_svg_f64(min_x),
            fmt_svg_f64(min_y),
            fmt_svg_f64(width),
            fmt_svg_f64(height),
        );
        self.container
            .leaf
            .push_owned_attr("viewBox".to_string(), v);
        self
    }

    pub fn overflow(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("overflow", v);
        self
    }
}

// ── SvgUse ────────────────────────────────────────────────────────────────────

/// `<use>` — instantiates a `<symbol>` or any other element by `href`.
///
/// ```rust,ignore
/// svg_use("#icon-star").x(10.0).y(10.0).width(24.0).height(24.0)
/// ```
pub struct SvgUse {
    container: SvgContainer,
}

pub fn svg_use(href: &str) -> SvgUse {
    let mut container = SvgContainer::new("use");
    container.leaf.push_str_attr("href", href);
    SvgUse { container }
}

impl_svg_container_elem!(SvgUse);

impl SvgUse {
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
}

// ── SvgMarker ─────────────────────────────────────────────────────────────────

/// `<marker>` — defines arrowheads or other path-end decorations. Place in `<defs>`.
pub struct SvgMarker {
    container: SvgContainer,
}

pub fn svg_marker(id: &str) -> SvgMarker {
    let mut container = SvgContainer::new("marker");
    container.leaf.push_str_attr("id", id);
    SvgMarker { container }
}

impl_svg_container_elem!(SvgMarker);

impl SvgMarker {
    pub fn view_box(mut self, min_x: f64, min_y: f64, width: f64, height: f64) -> Self {
        let v = format!(
            "{} {} {} {}",
            fmt_svg_f64(min_x),
            fmt_svg_f64(min_y),
            fmt_svg_f64(width),
            fmt_svg_f64(height),
        );
        self.container
            .leaf
            .push_owned_attr("viewBox".to_string(), v);
        self
    }
    pub fn ref_x(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("refX", v);
        self
    }
    pub fn ref_y(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("refY", v);
        self
    }
    pub fn marker_width(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("markerWidth", v);
        self
    }
    pub fn marker_height(mut self, v: impl IntoSvgF64) -> Self {
        self.container.leaf.push_f64("markerHeight", v);
        self
    }
    pub fn orient(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("orient", v);
        self
    }
    pub fn marker_units(mut self, v: &str) -> Self {
        self.container.leaf.push_str_attr("markerUnits", v);
        self
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::presentation::SvgElemMethods;
    use super::super::shapes::svg_circle;
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn svg_root_renders_xmlns() {
        let html = rh(&svg_root("400", "300"));
        assert!(html.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
        assert!(html.contains(r#"width="400""#));
        assert!(html.contains(r#"height="300""#));
        assert!(html.ends_with("</svg>"));
    }

    #[test]
    fn svg_root_view_box() {
        let html = rh(&svg_root("200", "200").view_box(0.0, 0.0, 200.0, 200.0));
        assert!(html.contains(r#"viewBox="0 0 200 200""#));
    }

    #[test]
    fn svg_root_with_children() {
        let html = rh(&svg_root("100", "100").child(svg_circle(50.0, 50.0, 40.0)));
        assert!(html.contains(r#"<circle cx="50" cy="50" r="40"/>"#));
    }

    #[test]
    fn svg_group_wraps_children() {
        let html = rh(&svg_group()
            .child(svg_circle(10.0, 10.0, 5.0))
            .child(svg_circle(20.0, 20.0, 5.0)));
        assert!(html.starts_with("<g>"));
        assert!(html.ends_with("</g>"));
        let count = html.matches("<circle").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn svg_group_with_transform() {
        use super::super::transform::Transform;
        let html = rh(&svg_group()
            .transform(Transform::translate(100.0, 0.0))
            .child(svg_circle(0.0, 0.0, 10.0)));
        assert!(html.contains(r#"transform="translate(100,0)""#));
    }

    #[test]
    fn svg_defs_renders_as_defs() {
        let html = rh(&svg_defs());
        assert_eq!(html, "<defs></defs>");
    }

    #[test]
    fn svg_symbol_with_id_and_viewbox() {
        let html = rh(&svg_symbol("icon").view_box(0.0, 0.0, 24.0, 24.0));
        assert!(html.contains(r#"id="icon""#));
        assert!(html.contains(r#"viewBox="0 0 24 24""#));
        assert!(html.ends_with("</symbol>"));
    }

    #[test]
    fn svg_use_renders_href() {
        let html = rh(&svg_use("#icon").x(10.0).y(10.0).width(24.0).height(24.0));
        assert!(html.contains(r##"href="#icon""##));
        assert!(html.contains(r#"x="10""#));
        assert!(html.contains(r#"width="24""#));
    }

    #[test]
    fn svg_marker_attrs() {
        let html = rh(&svg_marker("arrow")
            .view_box(0.0, 0.0, 10.0, 10.0)
            .ref_x(5.0)
            .ref_y(5.0)
            .marker_width(6.0)
            .marker_height(6.0)
            .orient("auto"));
        assert!(html.contains(r#"id="arrow""#));
        assert!(html.contains(r#"orient="auto""#));
        assert!(html.contains(r#"refX="5""#));
    }

    #[test]
    fn svg_group_into_component() {
        let c = svg_group()
            .child(svg_circle(0.0, 0.0, 1.0))
            .into_component();
        assert!(rh(&*c).contains("<g>"));
    }
}
