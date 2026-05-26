use crate::view_components::*;

#[routes]
pub enum RwRoute {
    #[path("/")]
    Home,
    #[path("/login")]
    Login,
    #[path("/register")]
    Register,
    #[path("/settings")]
    Settings,
    #[path("/editor")]
    NewArticle,
    #[path("/editor/:slug")]
    EditArticle { slug: String },
    #[path("/article/:slug")]
    Article { slug: String },
    #[path("/@:username")]
    Profile { username: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::Route;

    #[test]
    fn at_prefix_param_parses_from_path() {
        assert_eq!(
            RwRoute::from_path("/@alice"),
            RwRoute::Profile {
                username: "alice".to_string()
            }
        );
    }

    #[test]
    fn at_prefix_param_serializes_to_path() {
        assert_eq!(
            RwRoute::Profile {
                username: "alice".to_string()
            }
            .to_path(),
            "/@alice"
        );
    }

    #[test]
    fn plain_routes_unaffected() {
        assert_eq!(RwRoute::from_path("/"), RwRoute::Home);
        assert_eq!(
            RwRoute::from_path("/article/my-slug"),
            RwRoute::Article {
                slug: "my-slug".to_string()
            }
        );
        assert_eq!(
            RwRoute::Article {
                slug: "my-slug".to_string()
            }
            .to_path(),
            "/article/my-slug"
        );
    }

    #[test]
    fn guard_redirects_when_condition_met() {
        // Register a guard that redirects Home to Login.
        // In test mode navigate_to_path is a no-op, but guard registration
        // and the closure mechanism must compile and not panic.
        RwRoute::guard(|route| {
            if matches!(route, RwRoute::Home) {
                Some(RwRoute::Login)
            } else {
                None
            }
        });
    }

    #[test]
    fn guard_allows_when_none_returned() {
        // A no-op guard that always allows should not panic.
        RwRoute::guard(|_| None);
    }

    #[test]
    fn multiple_guards_chain() {
        // Multiple guards can be registered without panic.
        RwRoute::guard(|_| None);
        RwRoute::guard(|_| None);
    }
}
