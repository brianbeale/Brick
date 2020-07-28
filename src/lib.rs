#![allow(unused_imports)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
extern crate js_sys;

#[macro_use]
mod macros;
mod util;

mod dom_context;
use dom_context::DomContext;
// mod elements;
// use elements::*;
#[macro_use]
mod state_manager;
use state_manager::{DomSubject, SpanObserver, Subject};

#[macro_use]
mod rendering;
use rendering::*;

mod interactor;
use interactor::Hermit;

// mod example;
// use example::*;
mod verbose_ex;
use verbose_ex::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let mut p = DomSubject::new(42.to_string());
    let so = SpanObserver::new("test-span");
    p.add_observer("test-span", Box::new(so));
    p.update("Observers are nice and fancy".to_string());

    let callback = cb! { move |_| console_log!{"side effect"} };

    let dc = DomContext::new();
    dc.create_with_append("p", "a paragraph mounted by my method.");
    dc.create_with_append("button", "effect")
        .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;

    callback.forget();

    console_log!(p.datum.to_string());

    let mut h = Hermit::new(MyCounter { initial_count: 0 });
    h.run();

    Ok(())
}
