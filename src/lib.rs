use wasm_bindgen::prelude::*;
extern crate js_sys;
// #[macro_use]
// extern crate hermit_proc_macros;
// use hermit_proc_macros::show_streams;

#[macro_use]
mod macros;
mod composites;
mod dom_context;
mod elements;

#[macro_use]
mod state_manager;
mod rendering;

mod interactor;
use interactor::Hermit;

mod counter;
use counter::Counter;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let h = Hermit::new(Counter { count: 0 });
    h.run();

    Ok(())
}
