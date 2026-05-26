/// Show `$fallback(err)` when `$signal` is `Load::Failed`; show `$children` otherwise.
///
/// The closure parameter `$err` is a `Signal<String>` containing the error message.
/// Use `&$err.read()` for a snapshot or `live!("{err}")` for reactive text.
///
/// ```rust,ignore
/// catch!(articles.signal(), |err| p(&err.read()), load!(articles.signal(), |data| p(&data.title)))
/// ```
#[allow(unused_macros)]
macro_rules! catch {
    ($signal:expr, |$err:ident| $fallback:expr, $children:expr $(,)?) => {{
        use $crate::view_components::IntoComponent as _;
        let __err_sig = ($signal).map(|l| match l {
            $crate::state_mgmt::Load::Failed(e) => e.to_string(),
            _ => String::new(),
        });
        let __has_err = ($signal).map(|l| l.is_failed());
        let $err = __err_sig.clone();
        Box::new($crate::view_components::ViewConditional::new(
            __has_err,
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
    fn catch_shows_fallback_when_failed() {
        let sig: Signal<Load<String, String>> = Signal::new(Load::Failed("boom".to_string()));
        let result = catch!(sig, |err| p(&err.read()), p("content"));
        let html = rh(&*result);
        assert!(
            html.contains("boom"),
            "error text should appear when Failed"
        );
        assert!(
            !html.contains("content"),
            "content should not appear when Failed"
        );
    }

    #[test]
    fn catch_shows_content_when_loaded() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loaded("data".to_string()));
        let result = catch!(sig, |err| p(&err.read()), p("content"));
        let html = rh(&*result);
        assert!(
            html.contains("content"),
            "content should appear when Loaded"
        );
        assert!(!html.contains("boom"), "no error text when Loaded");
    }

    #[test]
    fn catch_shows_content_when_loading() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loading);
        let result = catch!(sig, |err| p(&err.read()), p("content"));
        let html = rh(&*result);
        assert!(
            html.contains("content"),
            "content should appear when Loading"
        );
    }
}
