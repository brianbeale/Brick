#[macro_export]
macro_rules! bind {
    ( $state_var_rc:expr ) => {
        std::rc::Rc::new(std::cell::RefCell::new(Box::new(
            crate::state_mgmt::BoundState::new(std::rc::Rc::clone(&$state_var_rc)),
        )))
    };
}
