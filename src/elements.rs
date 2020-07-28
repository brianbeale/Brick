#![allow(unused)]

use wasm_bindgen::prelude::*;

use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

static SPAN_COUNTER: AtomicUsize = AtomicUsize::new(0);
static CONTAINER_COUNTER: AtomicUsize = AtomicUsize::new(0);

use crate::state_manager::{DomSubject, SpanObserver, Subject};

pub struct Span {
    class_name: String,
    pub text_content: DomSubject<String>,
}
impl Span {
    pub fn new<T: Display>(text: &T) -> Self {
        let mut text_content = DomSubject::new(text.to_string());
        let class_name = format!("span_{}", SPAN_COUNTER.fetch_add(1, Ordering::SeqCst));
        text_content.add_observer(&class_name, Box::new(SpanObserver::new(&class_name)));
        Span {
            class_name,
            text_content,
        }
    }
}
impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let outer_html = format!(
            "<span class={}>{}</span>",
            self.class_name, self.text_content.datum
        );
        write!(f, "{}", &outer_html)
    }
}

pub struct Container {
    tag: &'static str,
    class_name: String,
    children: Vec<Box<dyn Display>>,
}
impl Container {
    pub fn new(tag: &'static str, children: Vec<Box<dyn Display>>) -> Self {
        Container {
            tag,
            class_name: format!(
                "container_{}",
                CONTAINER_COUNTER.fetch_add(1, Ordering::SeqCst)
            ),
            children,
        }
    }
}
impl Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut inner = String::new();
        for s in self.children.iter().map(|x| x.to_string().clone()) {
            inner.extend(s.chars());
        }
        let outer_html = format!(
            "<{} class={}>{}</{}>",
            self.tag, self.class_name, inner, self.tag
        );
        write!(f, "{}", &outer_html)
    }
}

pub trait DomDisplay {
    fn dom_string(&self) -> String;
}
impl<T: DomDisplay> DomDisplay for Vec<T> {
    fn dom_string(&self) -> String {
        let mut ds = String::new();
        for item in self.iter().map(|x| x.dom_string()) {
            ds += &item;
        }
        ds
    }
}
impl DomDisplay for &str {
    fn dom_string(&self) -> String {
        self.to_string()
    }
}
