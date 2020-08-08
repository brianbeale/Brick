use wasm_bindgen::prelude::*;
extern crate js_sys;

mod elements;

#[macro_use]
mod view_components;

#[macro_use]
mod state_mgmt;

#[macro_use]
mod controller_system;

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
