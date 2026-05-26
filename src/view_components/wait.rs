/// Show `$fallback` while `$signal` is in `Load::Loading` state; show `$children` otherwise.
///
/// ```rust,ignore
/// wait!(articles.signal(), p("Loading…"), load!(articles.signal(), |data| p(&data.title)))
/// ```
#[allow(unused_macros)]
macro_rules! wait {
    ($signal:expr, $fallback:expr, $children:expr $(,)?) => {{
        use $crate::view_components::IntoComponent as _;
        let __loading = ($signal).map(|l| l.is_loading());
        Box::new($crate::view_components::ViewConditional::new(
            __loading,
            ($fallback).into_component(),
            Some(($children).into_component()),
        ))
        .mount()
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
    fn wait_shows_fallback_while_loading() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loading);
        let result = wait!(sig, p("loading…"), p("content"));
        let html = rh(&*result);
        assert!(
            html.contains("loading…"),
            "fallback should appear when Loading"
        );
        assert!(
            !html.contains("content"),
            "content should not appear when Loading"
        );
    }

    #[test]
    fn wait_shows_content_when_loaded() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loaded("ok".to_string()));
        let result = wait!(sig, p("loading…"), p("content"));
        let html = rh(&*result);
        assert!(
            !html.contains("loading…"),
            "fallback should not appear when Loaded"
        );
        assert!(
            html.contains("content"),
            "content should appear when Loaded"
        );
    }

    #[test]
    fn wait_shows_content_when_failed() {
        let sig: Signal<Load<String, String>> = Signal::new(Load::Failed("err".to_string()));
        let result = wait!(sig, p("loading…"), p("content"));
        let html = rh(&*result);
        assert!(
            !html.contains("loading…"),
            "fallback should not appear when Failed"
        );
        assert!(
            html.contains("content"),
            "content should appear when Failed (catch! handles errors)"
        );
    }
}
