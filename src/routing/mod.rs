use crate::view_components::Brick;

// ─── Core trait ──────────────────────────────────────────────────────────────

pub trait Route: Clone + 'static {
    fn from_path(path: &str) -> Self;
    fn to_path(&self) -> String;
}

// ─── NavLink component ───────────────────────────────────────────────────────

pub struct NavLink {
    pub path: String,
    pub children: Vec<Box<dyn crate::view_components::Brick>>,
}

impl Brick for NavLink {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("a");
        Renderer::set_attr(&node, "href", &self.path);
        Renderer::set_attr(&node, "data-brick-navigate", &self.path);
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }
}

// ─── Linkable ─────────────────────────────────────────────────────────────────

/// Anything with a canonical URL path — route types and their builders.
pub trait Linkable {
    fn to_nav_path(&self) -> String;
}

/// Content that can go inside a navigation link.
pub trait IntoNavContent {
    fn into_nav_content(self) -> Box<dyn crate::view_components::Brick>;
}

struct NavText(String);
impl Brick for NavText {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        Renderer::append(parent, &Renderer::text(&self.0));
    }
}

impl IntoNavContent for &str {
    fn into_nav_content(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(NavText(self.to_string()))
    }
}
impl IntoNavContent for String {
    fn into_nav_content(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(NavText(self))
    }
}
impl IntoNavContent for &String {
    fn into_nav_content(self) -> Box<dyn crate::view_components::Brick> {
        Box::new(NavText(self.clone()))
    }
}
impl<T: crate::view_components::Brick + 'static> IntoNavContent for Box<T> {
    fn into_nav_content(self) -> Box<dyn crate::view_components::Brick> {
        self
    }
}

/// Extension: `.link(content)` on any `Linkable` type (routes, builders).
pub trait LinkExt: Linkable + Sized {
    fn link<C: IntoNavContent>(self, content: C) -> Box<NavLink> {
        Box::new(NavLink {
            path: self.to_nav_path(),
            children: vec![content.into_nav_content()],
        })
    }
}
impl<T: Linkable> LinkExt for T {}

/// Extension: `.link(route)` on `&str` and `String`.
pub trait StrLinkExt {
    fn link<R: Linkable>(self, route: R) -> Box<NavLink>;
}
impl StrLinkExt for &str {
    fn link<R: Linkable>(self, route: R) -> Box<NavLink> {
        route.link(self)
    }
}
impl StrLinkExt for String {
    fn link<R: Linkable>(self, route: R) -> Box<NavLink> {
        route.link(self.as_str())
    }
}
impl StrLinkExt for &String {
    fn link<R: Linkable>(self, route: R) -> Box<NavLink> {
        route.link(self.as_str())
    }
}

// ─── Element builder .link() ─────────────────────────────────────────────────

impl crate::view_components::ViewLeafText {
    /// Wrap this element in a `<a href="...">` navigation link.
    pub fn link<R: Linkable>(self: Box<Self>, route: R) -> Box<NavLink> {
        Box::new(NavLink {
            path: route.to_nav_path(),
            children: vec![self as Box<dyn crate::view_components::Brick>],
        })
    }
}

// ─── SPA navigation ──────────────────────────────────────────────────────────

/// Push a new path to the history stack and fire a `popstate` event so
/// registered routers can update their signal.
#[cfg(brick_dom)]
pub fn navigate_to_path(path: &str) {
    let window = web_sys::window().unwrap();
    window
        .history()
        .unwrap()
        .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
        .unwrap();
    let event = web_sys::Event::new("popstate").unwrap();
    window.dispatch_event(&event).unwrap();
}

#[cfg(not(brick_dom))]
pub fn navigate_to_path(_path: &str) {}

#[cfg(test)]
mod tests {
    use super::{Brick, NavLink};

    fn make_link(path: &str, text: &str) -> Box<NavLink> {
        Box::new(NavLink {
            path: path.to_string(),
            children: vec![Box::new(super::NavText(text.to_string()))],
        })
    }

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn parity_nav_link_text() {
        let link = make_link("/about", "About");
        let html = rh(&*link);
        assert!(html.contains(r#"href="/about""#));
        assert!(html.contains("About"));
    }

    #[test]
    fn parity_nav_link_root() {
        let link = make_link("/", "Home");
        let html = rh(&*link);
        assert!(html.contains(r#"href="/""#));
        assert!(html.contains("Home"));
    }
}
