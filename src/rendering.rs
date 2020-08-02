use crate::dom_context::DomContext;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub trait Render {
    fn render(&self) -> ViewManager;
}

pub struct ViewManager {
    pub html: String,
    pub event_listeners: HashMap<
        &'static str, // class_name
        (
            &'static str,                       // Event type
            Closure<dyn FnMut(web_sys::Event)>, // State methods moved
        ),
    >,
}
impl ViewManager {
    pub fn mount_to_body(&mut self, dc: &DomContext) {
        dc.body.set_inner_html(&self.html);
        let listening_classes: Vec<&str> = self.event_listeners.keys().map(|x| x.clone()).collect();
        for class_name in listening_classes.iter() {
            let target = dc
                .document
                .get_elements_by_class_name(class_name)
                .item(0)
                .unwrap();
            let (name, callback) = self.event_listeners.get(class_name).unwrap();
            target
                .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
                .unwrap();
        }
    }
}

// classes should be generated for every element with
// a trigger or Event handler
#[macro_export]
macro_rules! view {
    ( $($template:expr),* ) => {};
}
