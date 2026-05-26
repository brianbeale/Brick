use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! console_log {
    ( $text:expr ) => {
        web_sys::console::log_1(&wasm_bindgen::prelude::JsValue::from_str(&$text));
    };
}

pub fn _print_type_of<T>(_: &T) {
    web_sys::console::log_1(&JsValue::from_str(&format!(
        "{}",
        std::any::type_name::<T>()
    )));
}
