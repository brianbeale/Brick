use crate::elements::*;
use crate::rendering::{Render, ViewManager};
use crate::state_manager::{State, Subject};
use hermit_proc_macros::{controller, Model};
use wasm_bindgen::prelude::*;

#[derive(Model)]
pub struct Counter {
    pub count: usize,
}

#[controller]
impl CounterModel {
    pub fn increment(&mut self, _e: web_sys::Event) {
        set!(self.count => + 1);
    }
    pub fn decrement(&mut self, _e: web_sys::Event) {
        if *self.count != 0 {
            set!(self.count => - 1);
        }
    }
}
impl Render for Counter {
    fn render(&self) -> ViewManager {
        let mut model = CounterModel::new(self);
        let html = h1("MyCounter")
            + &p(react!("The count is: {}", model.count))
            + r#"<button class="increment">Increment</button>"#
            + r#"<button class="decrement">Decrement</button>"#;
        ViewManager {
            html,
            event_listeners: model.controller_methods(),
        }
    }
}
