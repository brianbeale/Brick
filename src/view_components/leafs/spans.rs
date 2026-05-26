#![allow(dead_code)]
use crate::state_mgmt::{SpanObserver, State, Subject};
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) static SPAN_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn fresh_span_class() -> String {
    format!("span_{}", SPAN_COUNTER.fetch_add(1, Ordering::SeqCst))
}

pub struct Span {
    pub class_name: String,
    pub text_content: State<String>,
}
impl Span {
    pub fn new<T: Display>(text: &T) -> Self {
        let class_name = format!("span_{}", SPAN_COUNTER.fetch_add(1, Ordering::SeqCst));
        let mut text_content = State::new(text.to_string());
        text_content.add_observer(&class_name, Box::new(SpanObserver::new(&class_name)));
        Span {
            class_name,
            text_content,
        }
    }
}
impl crate::view_components::Brick for Span {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("span");
        Renderer::set_attr(&node, "class", &self.class_name);
        Renderer::append(&node, &Renderer::text(&*self.text_content));
        Renderer::append(parent, &node);
    }
}
