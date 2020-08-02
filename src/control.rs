#![allow(unused)]

use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! controller_methods {
    ( $model:ident, $( $listener:expr ),* ) => {
        let mut event_listeners = std::collections::HashMap::new();
        let sharing_model = std::rc::Rc::new(std::cell::RefCell::new(self));
        $(
            event_listeners.insert(
                stringify!($listener), ("click", method!($listener, sharing_model))
            );
        )*
        event_listeners
    }
}
