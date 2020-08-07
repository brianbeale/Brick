// #![allow(unused)]
use crate::state_mgmt::{SpanObserver, State, Subject};
pub use hermit_proc_macros::{controller, view, Model};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub trait ViewComponent {
    fn html(&self) -> String;
}
pub trait Render {
    fn render(&self) -> ViewComposite;
}

// Shell?
pub struct ViewComposite {
    pub tag: &'static str,
    pub class_name: String,
    pub model: std::rc::Rc<std::cell::RefCell<dyn Any>>,
    pub children: Vec<Box<dyn ViewComponent>>,
    pub event_listeners: HashMap<
        &'static str, // class_name
        (
            &'static str,                       // Event type
            Closure<dyn FnMut(web_sys::Event)>, // State methods moved into closure
        ),
    >,
}
impl ViewComposite {
    pub fn mount(&mut self, anchor: &web_sys::HtmlElement) {
        anchor.set_inner_html(&self.html());
        let listening_classes: Vec<&str> = self.event_listeners.keys().map(|x| x.clone()).collect();
        for class_name in listening_classes.iter() {
            let target = anchor
                .get_elements_by_class_name(class_name)
                .item(0) // TODO: extend to multiple class matches
                .unwrap();
            let (name, callback) = self.event_listeners.get(class_name).unwrap();
            target
                .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
                .unwrap();
        }
    }
}
impl ViewComponent for ViewComposite {
    fn html(&self) -> String {
        let inner: String = self.children.iter().map(|x| x.html()).collect();
        format!(
            r#"<{} class="{}">{}</{}>"#,
            self.tag, self.class_name, inner, self.tag
        )
    }
}
pub struct ViewLeafText {
    tag: &'static str,
    class_name: String,
    text_content: String,
}
// Builder methods
impl ViewLeafText {
    pub fn _class_name<T: Display + ?Sized>(mut self: Box<Self>, name: &T) -> Box<Self> {
        self.class_name = name.to_string();
        self
    }
    // Duplicates class_name
    pub fn c<T: Display + ?Sized>(mut self: Box<Self>, name: &T) -> Box<Self> {
        self.class_name = name.to_string();
        self
    }
}
impl ViewComponent for ViewLeafText {
    fn html(&self) -> String {
        format!(
            r#"<{} class="{}">{}</{}>"#,
            self.tag, self.class_name, self.text_content, self.tag
        )
    }
}
macro_rules! tag_funcs {
    ( $( $name:ident ),* ) => {
        $(
            pub fn $name<T: Display + ?Sized>(text: &T) -> Box<ViewLeafText> {
                Box::new(ViewLeafText {
                    tag: stringify!($name),
                    class_name: "".to_string(),
                    text_content: text.to_string(),
                })
            }
        )*
    };
}
tag_funcs!(p, h1);
pub fn button<T: Display + ?Sized>(text: &T) -> Box<ViewLeafText> {
    Box::new(ViewLeafText {
        tag: "button",
        class_name: text.to_string(),
        text_content: text.to_string(),
    })
}

static SPAN_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct ViewLeafSpan {
    class_name: String,
    pub text_content: State<String>,
}
impl ViewLeafSpan {
    pub fn new<T: Display>(text: &T) -> Self {
        let class_name = format!("span_{}", SPAN_COUNTER.fetch_add(1, Ordering::SeqCst));
        let mut text_content = State::new(text.to_string());
        text_content.add_observer(&class_name, Box::new(SpanObserver::new(&class_name)));
        ViewLeafSpan {
            class_name,
            text_content,
        }
    }
}
impl ViewComponent for ViewLeafSpan {
    fn html(&self) -> String {
        format!(
            r#"<span class="{}">{}</span>"#,
            self.class_name, *self.text_content
        )
    }
}

pub struct ViewLeafMedia {
    tag: &'static str,
    url: String,
    attrs: HashMap<&'static str, &'static str>,
}
