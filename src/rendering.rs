#![allow(unused)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::collections::HashMap;
use std::fmt::Display;

use crate::dom_context::DomContext;
use crate::state_manager::Subject;

pub trait CreateModel {
    fn to_model(self) -> Box<dyn Render>;
}
pub trait Render {
    fn render(self: Box<Self>) -> ViewManager;
}

// DOM triggers should be attachable to State Subjects (or already attached?)
// Event listeners should be attachable to DOM Elements
pub struct ViewManager {
    pub html: String,
    pub listening_classes: Vec<&'static str>,
    pub event_listeners: HashMap<
        &'static str, // class_name or Element type
        HashMap<
            &'static str,                       // Event type
            Closure<dyn FnMut(web_sys::Event)>, // State methods moved
        >,
    >,
}
impl ViewManager {
    pub fn mount_to_body(&mut self, dc: &DomContext) {
        dc.body.set_inner_html(&self.html);
        // let mut returned_callbacks = Vec::new();
        for class_name in self.listening_classes.iter() {
            let target = dc
                .document
                .get_elements_by_class_name(class_name)
                .item(0)
                .unwrap();
            let mut class_listeners = self.event_listeners.remove(class_name).unwrap();
            for (name, mut callback) in class_listeners.into_iter() {
                target
                    .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
                    .unwrap();
                // TODO: probably don't want to do this (memory leak)
                callback.forget();
            }
        }
        // Ok()
    }
}

// classes should be generated for every element with
// a trigger or Event handler
#[macro_export]
macro_rules! view {
    ( $($template:expr),* ) => {};
}

// rendering is responsible for the initial paint,
// but also for attaching triggers to Subjects of State,
// which will use DomContext to acheive the corresponding
// update of the Document

// attaching Event listeners?
