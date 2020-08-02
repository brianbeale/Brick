#![allow(unused)]

#[macro_export]
macro_rules! console_log {
    ( $text:expr ) => {
        web_sys::console::log_1(&wasm_bindgen::prelude::JsValue::from_str(&$text));
    };
}

// Prepare methods from Rc<RefCell<Model>> to be moved to JS
macro_rules! method {
    ( $method_name:ident, $sharing_model:expr ) => {{
        let s = std::rc::Rc::clone(&$sharing_model);
        Closure::wrap(
            Box::new(move |e: web_sys::Event| s.borrow_mut().$method_name(e))
                as Box<dyn FnMut(web_sys::Event)>,
        )
    }};
}

// Helper macro for attaching SpanObservers to State Subjects
macro_rules! observe {
    ( $subject:expr, $class_name:expr ) => {
        $subject.add_observer(
            $class_name,
            Box::new(crate::state_manager::SpanObserver::new($class_name)),
        );
    };
}

// Reactive text node with format!-like interface
macro_rules! react {
    ( $template:expr, $subject:expr ) => {{
        let s = crate::elements::Span::new(&$subject);
        observe!($subject, &s.class_name);
        &format!($template, s)
    }};
}
