#![allow(unused)]

use crate::rendering::{CreateModel, Render, ViewManager};
use crate::state_manager::{DomSubject, SpanObserver, Subject};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Props
pub struct MyCounter {
    pub initial_count: usize,
}
impl CreateModel for MyCounter {
    fn to_model(self) -> Box<dyn Render> {
        Box::new(Model {
            count: DomSubject::new(self.initial_count),
        })
    }
}
// State
pub struct Model {
    pub count: DomSubject<usize>,
}
// Controller
impl Model {
    pub fn increment(&mut self, e: web_sys::Event) {
        self.count.update(self.count.datum + 1);
        console_log!("Increment");
        console_log!(format!("self.count: {}", self.count));
    }
    pub fn decrement(&mut self, e: web_sys::Event) {
        if self.count.datum == 0 {
            return;
        }
        self.count.update(self.count.datum - 1);
        console_log!("Decrement");
    }
}
// How to unwrap the View Magic?
impl Render for Model {
    fn render(mut self: Box<Self>) -> ViewManager {
        let html = format!(
            r#"<h1>MyCounter</h1>
            <p>The count is: <span class="example_span">{}</span></p>
            <button class="increment">Increment</button>
            <button class="decrement">Decrement</button>"#,
            self.count.datum
        );
        self.count
            .add_observer("example_span", Box::new(SpanObserver::new("example_span")));
        let mut event_listeners = HashMap::new();
        let mut inc_handlers = HashMap::new();
        let sharing_model = Rc::new(RefCell::new(self));
        let s = Rc::clone(&sharing_model);
        let inc_fn = Closure::wrap(
            Box::new(move |e: web_sys::Event| s.borrow_mut().increment(e))
                as Box<dyn FnMut(web_sys::Event)>,
        );
        inc_handlers.insert("click", inc_fn);
        event_listeners.insert("increment", inc_handlers);
        let s = Rc::clone(&sharing_model);
        let dec_fn = Closure::wrap(
            Box::new(move |e: web_sys::Event| s.borrow_mut().decrement(e))
                as Box<dyn FnMut(web_sys::Event)>,
        );
        let mut dec_handlers = HashMap::new();
        dec_handlers.insert("click", dec_fn);
        event_listeners.insert("decrement", dec_handlers);
        console_log!(format!("event_listeners: {:?}", event_listeners));
        ViewManager {
            html,
            listening_classes: vec!["increment", "decrement"],
            event_listeners,
        }
    }
}
