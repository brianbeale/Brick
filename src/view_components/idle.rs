/// Show `$content` when `$signal` is `Load::Idle`; show nothing otherwise.
///
/// Pairs with `Resource::lazy()` to display a placeholder until the user triggers a fetch.
///
/// ```rust,ignore
/// idle!(my.results, p("Enter a search term to begin"))
/// ```
#[allow(unused_macros)]
macro_rules! idle {
    ($signal:expr, $content:expr $(,)?) => {{
        use $crate::view_components::IntoComponent as _;
        let __is_idle = ($signal).map(|l| l.is_idle());
        Box::new($crate::view_components::ViewConditional::new(
            __is_idle,
            ($content).into_component(),
            None,
        ))
    }};
}

#[cfg(test)]
mod tests {
    use crate::state_mgmt::{Load, Signal};
    use crate::view_components::Brick;
    use crate::view_components::leafs::p;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn idle_shows_content_when_idle() {
        let sig: Signal<Load<String>> = Signal::new(Load::Idle);
        let result = idle!(sig, p("start here"));
        let html = rh(&*result);
        assert!(
            html.contains("<p>start here</p>"),
            "content should appear when Idle"
        );
        assert!(
            !html.contains("hidden"),
            "wrapper should not be hidden when Idle"
        );
    }

    #[test]
    fn idle_shows_nothing_when_loading() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loading);
        let result = idle!(sig, p("start here"));
        let html = rh(&*result);
        assert!(
            html.contains(" hidden"),
            "wrapper should be hidden when Loading"
        );
    }

    #[test]
    fn idle_shows_nothing_when_loaded() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loaded("data".to_string()));
        let result = idle!(sig, p("start here"));
        let html = rh(&*result);
        assert!(
            html.contains(" hidden"),
            "wrapper should be hidden when Loaded"
        );
    }
}
