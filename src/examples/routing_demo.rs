use crate::routing::LinkExt;
use crate::view_components::*;

// ── Route enum ────────────────────────────────────────────────────────────────

#[routes]
pub enum AppRoute {
    #[path("/")]
    Home,
    #[path("/about")]
    About,
    #[path("/users")]
    Users(UsersRoute),
}

#[routes(AppRoute::Users)]
pub enum UsersRoute {
    #[path("/")]
    List,
    #[path("/:id")]
    Show { id: String },
}

// ── Pages ─────────────────────────────────────────────────────────────────────

#[model]
pub struct HomePage {}

#[view(HomePage)]
fn render() -> Box<ViewComposite> {
    children! {
        h2("Welcome"),
        p("This is the home page."),
        AppRoute::users.list.link("User List"),
        AppRoute::about.link("About"),
    }
}

#[model]
pub struct AboutPage {}

#[view(AboutPage)]
fn render() -> Box<ViewComposite> {
    children! {
        h2("About"),
        p("Brick: declarative DOM, reactive state, no diffing."),
        AppRoute::home.link("← Home"),
    }
}

#[model]
pub struct UsersListPage {}

#[view(UsersListPage)]
fn render() -> Box<ViewComposite> {
    children! {
        h2("Users"),
        AppRoute::users.show("1").link("User 1"),
        AppRoute::users.show("2").link("User 2"),
        AppRoute::home.link("← Home"),
    }
}

#[model]
pub struct UserShowPage {
    #[prop]
    pub id: String,
}

#[view(UserShowPage)]
fn render() -> Box<ViewComposite> {
    children! {
        h2(&my.id),
        AppRoute::users.list.link("← Users"),
    }
}

// ── Root (subroute dispatcher) ────────────────────────────────────────────────

#[model]
pub struct RoutingDemo {}

#[view(RoutingDemo)]
fn render() -> Box<ViewComposite> {
    children! {
        subroute!(app_route(), |route| match route {
            AppRoute::Home => HomePage { ..cascade() }.into_component(),
            AppRoute::About => AboutPage { ..cascade() }.into_component(),
            AppRoute::Users(UsersRoute::List) => UsersListPage { ..cascade() }.into_component(),
            AppRoute::Users(UsersRoute::Show { id }) => UserShowPage { id, ..cascade() }.into_component(),
            _ => p("Not found").into_component(),
        }),
    }
}
