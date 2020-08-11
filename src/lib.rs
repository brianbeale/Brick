use wasm_bindgen::prelude::*;
extern crate js_sys;

#[macro_use]
mod view_components;

#[macro_use]
mod state_mgmt;

#[macro_use]
mod controller_system;

mod interactor;
use interactor::Hermit;

mod examples;
use examples::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // let h = Hermit::new(Counter { count: state!(1) });
    let h = Hermit::new(DoubleCounter { count: state!(1) });
    h.run();

    Ok(())
}
