use wasm_bindgen::prelude::*;

pub fn _print_type_of<T>(_: &T) {
    web_sys::console::log_1(&JsValue::from_str(&format!(
        "{}",
        std::any::type_name::<T>()
    )));
}
