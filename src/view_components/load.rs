/// Render content from a `Signal<Load<T>>` when loaded; render nothing for other states.
///
/// ```rust,ignore
/// load!(my.articles, |articles| {
///     list!(articles, |a| p(&a.title))
/// })
/// ```
#[allow(unused_macros)]
macro_rules! load {
    ($signal:expr, |$val:ident| $body:expr $(,)?) => {{
        use $crate::view_components::IntoComponent as _;
        $crate::view_components::ViewResolve::new(
            ($signal).clone(),
            std::rc::Rc::new(|| $crate::view_components::nothing()),
            std::rc::Rc::new(|| $crate::view_components::nothing()),
            std::rc::Rc::new(move |$val: &_| ($body).into_component()),
            std::rc::Rc::new(|_: &_| $crate::view_components::nothing()),
        )
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
    fn load_shows_content_when_loaded() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loaded("hello".to_string()));
        let result = load!(sig, |val| p(val.as_str()));
        let html = rh(&*result);
        assert!(html.contains("<p>hello</p>"), "should render loaded value");
    }

    #[test]
    fn load_shows_nothing_when_loading() {
        let sig: Signal<Load<String>> = Signal::new(Load::Loading);
        let result = load!(sig, |val| p(val.as_str()));
        let html = rh(&*result);
        assert!(!html.contains("<p>"), "should render nothing when Loading");
    }

    #[test]
    fn load_shows_nothing_when_idle() {
        let sig: Signal<Load<String>> = Signal::new(Load::Idle);
        let result = load!(sig, |val| p(val.as_str()));
        let html = rh(&*result);
        assert!(!html.contains("<p>"), "should render nothing when Idle");
    }
}
