#[macro_export]
macro_rules! state {
    ( $lit:expr ) => {
        std::rc::Rc::new(std::cell::RefCell::new(Box::new(
            crate::state_mgmt::State::new($lit),
        )))
    };
}
