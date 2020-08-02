#![allow(unused)]
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use wasm_bindgen::prelude::*;

pub trait ViewComponent {
    fn html(&self) -> String;
}

pub struct ViewComposite {
    pub own_html: &'static str,
    pub children: Vec<Box<dyn ViewComponent>>,
    pub model: Box<dyn Any>,
    pub listening_classes: Vec<&'static str>,
    pub event_listeners: HashMap<
        &'static str, // class_name
        (
            &'static str,                       // Event type
            Closure<dyn FnMut(web_sys::Event)>, // State methods moved into closure
        ),
    >,
}

pub struct TextLeaf {
    tag: &'static str,
    text_content: String,
}

pub struct MediaLeaf {
    tag: &'static str,
    url: String,
    attrs: HashMap<&'static str, &'static str>,
}
