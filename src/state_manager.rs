#![allow(unused)]

use crate::dom_context::DomContext;
use wasm_bindgen::prelude::*;

use std::collections::HashMap;
use std::fmt::Display;

pub trait Observer<T> {
    // Todo: abstract context?
    fn notify(&mut self, datum: &T, context: &DomContext);
}
pub trait Subject<T> {
    fn update(&mut self, new_datum: T);
    fn add_observer(&mut self, name: &str, obs: Box<dyn Observer<T>>);
    fn remove_observer(&mut self, name: &str);
}

pub struct SpanObserver {
    class_name: String,
}
impl SpanObserver {
    pub fn new(class_name: &str) -> Self {
        SpanObserver {
            class_name: class_name.to_string(),
        }
    }
}
impl<T: Display> Observer<T> for SpanObserver {
    fn notify(&mut self, datum: &T, context: &DomContext) {
        context
            .document
            .get_elements_by_class_name(&self.class_name)
            .item(0)
            .unwrap()
            .set_text_content(Some(&datum.to_string()));
    }
}
pub struct DomSubject<T> {
    pub datum: T,
    context: DomContext,
    observers: HashMap<String, Box<dyn Observer<T>>>,
}
impl<T> DomSubject<T> {
    pub fn new(datum: T) -> Self {
        DomSubject::<T> {
            datum,
            context: DomContext::new(),
            observers: HashMap::new(),
        }
    }
}
impl<T> Subject<T> for DomSubject<T> {
    fn update(&mut self, new_datum: T) {
        self.datum = new_datum;
        for (class_name, obs) in &mut self.observers {
            obs.notify(&self.datum, &self.context);
        }
    }
    fn add_observer(&mut self, class_name: &str, obs: Box<dyn Observer<T>>) {
        self.observers.insert(class_name.to_string(), obs);
    }
    fn remove_observer(&mut self, class_name: &str) {
        self.observers.remove(class_name);
    }
}
impl<T: Display> Display for DomSubject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.datum.to_string())
    }
}

#[macro_export]
macro_rules! set {
    ( $var:ident $op:tt $change:expr ) => {};
}

// state_manager is responsible for values with multiple interested units,
// represented as trigger functions. updating the Subject calls all triggers
// (Single Producer, Multiple Consumer)
// With a Singleton (Event Bus...?) receiving update notificatios on topics,
// this could be extended to be arbitrary many-to-many mappings
