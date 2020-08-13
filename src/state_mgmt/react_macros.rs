#[macro_export]

// Helper macro for attaching SpanObservers to State Subjects
macro_rules! observe {
    ( $subject:expr, $class_name:expr ) => {
        $subject.borrow_mut().add_observer(
            $class_name,
            Box::new(crate::state_mgmt::SpanObserver::new($class_name)),
        );
    };
}

// Reactive text node with format!-like interface
// macro_rules! react {
//     ( $template:expr, $subject:expr ) => {{
//         let s = crate::view_components::Span::new(&Rc::clone(&$subject).borrow_mut().read());
//         observe!(Rc::clone(&$subject), &s.class_name);
//         &format!($template, s.html())
//     }};
// }

macro_rules! react_text {
    ( $template:expr, $model:ident . $state_var:ident ) => {{
        let s = crate::view_components::Span::new(
            &Rc::clone(&$model.borrow().$state_var).borrow().read(),
        );
        observe!(Rc::clone(&$model.borrow().$state_var), &s.class_name);
        &format!($template, s.html())
    }};
}
