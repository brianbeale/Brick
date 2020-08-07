use wasm_bindgen::prelude::*;
extern crate js_sys;
// #[macro_use]
// extern crate hermit_proc_macros;
// use hermit_proc_macros::show_streams;

#[macro_use]
mod macros;
mod components;
mod dom_context;
mod elements;

mod state_mgmt;

mod interactor;
use interactor::Hermit;

mod examples;
use examples::Counter;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let h = Hermit::new(Counter { count: 1 });
    h.run();

    Ok(())
}
