#[macro_export]
macro_rules! state {
    ( $lit:expr ) => {
        std::rc::Rc::new(std::cell::RefCell::new(Box::new(
            crate::state_mgmt::State::new($lit),
        )))
    };
}

#[macro_export]
macro_rules! bind {
    ( $sharing_model:ident . $state_var:ident ) => {
        std::rc::Rc::new(std::cell::RefCell::new(Box::new(
            crate::state_mgmt::BoundState::new(std::rc::Rc::clone(
                &$sharing_model.borrow().$state_var,
            )),
        )))
    };
}
