#[macro_export]
macro_rules! bind_old {
    ( $state_var_rc:expr ) => {
        std::rc::Rc::new(std::cell::RefCell::new(Box::new(
            crate::state_mgmt::BoundState::new(std::rc::Rc::clone(&$state_var_rc)),
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
