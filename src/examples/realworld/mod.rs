pub mod api;
pub mod auth;
mod pages;
pub mod routes;

pub use auth::AuthStore;
pub use pages::*;
pub use routes::RwRoute;

#[allow(unused_imports)]
use crate::routing::LinkExt;
use crate::view_components::*;
use auth::is_logged_in;
use routes::rw_route;

#[model]
pub struct RealWorld {}

#[controller]
impl RealWorld {}

#[view(RealWorld)]
fn render() -> Box<ViewComposite> {
    style! {
        .rw-nav { background: white; border-bottom: 1px solid #ddd; padding: 0 1rem; display: flex; align-items: center; justify-content: space-between; height: 56px; }
        .rw-nav .brand { font-size: 1.2rem; font-weight: 700; color: #5cb85c; text-decoration: none; }
        .rw-nav-links { display: flex; align-items: center; gap: 1rem; }
        .rw-nav-links a { color: #555; text-decoration: none; font-size: 0.95rem; }
        .rw-nav-links a:hover { color: #222; }
        .rw-content { min-height: calc(100vh - 56px); }
    }
    let auth = AuthStore::get();
    let logged_in = is_logged_in();
    let username = auth.username.read();
    let username_href = format!("/profile/{}", username);
    let nav_links: Box<dyn Brick> = if logged_in {
        let mut children: Vec<Box<dyn Brick>> = Vec::new();
        children.push(a("New Article").attr("href", "/editor").into_component());
        children.push(a("Settings").attr("href", "/settings").into_component());
        children.push(a(&username).attr("href", &username_href).into_component());
        Box::new(BrickContainer {
            class: "rw-nav-links",
            children,
        })
    } else {
        let mut children: Vec<Box<dyn Brick>> = Vec::new();
        children.push(a("Sign in").attr("href", "/login").into_component());
        children.push(a("Sign up").attr("href", "/register").into_component());
        Box::new(BrickContainer {
            class: "rw-nav-links",
            children,
        })
    };
    children! {
        div {
            class("rw-nav"),
            a("conduit").c("brand").attr("href", "/"),
            nav_links,
        },
        div {
            class("rw-content"),
            subroute!(rw_route(), |route| match route {
                RwRoute::Home             => HomePage { ..cascade() }.into_component(),
                RwRoute::Login            => LoginPage { ..cascade() }.into_component(),
                RwRoute::Register         => RegisterPage { ..cascade() }.into_component(),
                RwRoute::Settings         => SettingsPage { ..cascade() }.into_component(),
                RwRoute::NewArticle       => EditorPage { slug: String::new(), ..cascade() }.into_component(),
                RwRoute::EditArticle { slug } => EditorPage { slug, ..cascade() }.into_component(),
                RwRoute::Article { slug } => ArticlePage { slug, ..cascade() }.into_component(),
                RwRoute::Profile { username } => ProfilePage { username, ..cascade() }.into_component(),
                _ => p("Not found").into_component(),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    #[test]
    fn renders_nav() {
        let app = RealWorld { ..cascade() };
        let html = crate::view_components::render_to_html(&*app.into_component());
        assert!(html.contains("conduit"));
    }

    #[test]
    fn renders_sign_in_link_when_logged_out() {
        let app = RealWorld { ..cascade() };
        let html = crate::view_components::render_to_html(&*app.into_component());
        assert!(html.contains("Sign in"));
    }
}
