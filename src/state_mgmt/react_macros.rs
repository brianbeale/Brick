#[macro_export]

macro_rules! observe {
    ( $subject:expr, $class_name:expr ) => {
        $subject.borrow_mut().add_observer(
            $class_name,
            Box::new(crate::state_mgmt::SpanObserver::new($class_name)),
        );
    };
}

// Like observe!, but applies a format closure instead of plain Display.
// Used by live! to wire typed, formatted reactive spans.
macro_rules! observe_fmt {
    ( $subject:expr, $class_name:expr, $fmt:expr ) => {{
        let _rc_obs = ::std::rc::Rc::clone(&$subject);
        let _cn = $class_name.to_string();
        #[cfg(brick_dom)]
        {
            let _cn2 = _cn.clone();
            _rc_obs.borrow_mut().add_observer(
                &_cn,
                Box::new(crate::state_mgmt::Effect::new(move |v| {
                    let _text: String = ($fmt)(v);
                    web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_elements_by_class_name(&_cn2)
                        .item(0)
                        .unwrap()
                        .set_text_content(Some(&_text));
                })),
            );
        }
    }};
}
