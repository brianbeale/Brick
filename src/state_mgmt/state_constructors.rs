#[macro_export]
macro_rules! state {
    ( $lit:expr ) => {
        crate::state_mgmt::Signal::new($lit)
    };
}

#[macro_export]
macro_rules! computed {
    ( $my:ident . $field:ident, $func:expr ) => {
        crate::state_mgmt::Signal::from_rc(crate::state_mgmt::compute($my.$field.rc(), $func))
    };
    ( $source:expr, $func:expr ) => {
        crate::state_mgmt::Signal::from_rc(crate::state_mgmt::compute($source.rc(), $func))
    };
}

#[macro_export]
macro_rules! bind {
    ( $my:ident . $field:ident ) => {
        crate::state_mgmt::Signal::from_rc(std::rc::Rc::new(std::cell::RefCell::new(Box::new(
            crate::state_mgmt::BoundState::new($my.$field.rc()),
        ))))
    };
}
