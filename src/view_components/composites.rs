use std::{any::Any, collections::HashMap};
use wasm_bindgen::{prelude::*, JsCast};

use super::ViewComponent;

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
    pub fn mount(&self, anchor: &web_sys::HtmlElement) {
        anchor.set_inner_html(&self.html());
        self.attach_listeners();
        for child in self.children.iter() {
            child.attach_listeners();
        }
    }
    pub fn c<T: std::fmt::Display + ?Sized>(mut self: Box<Self>, name: &T) -> Box<Self> {
        self.class_name = name.to_string();
        self
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
    fn attach_listeners(&self) {
        let listening_classes: Vec<&str> = self.event_listeners.keys().map(|x| x.clone()).collect();
        let anchor = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_elements_by_class_name(&self.class_name)
            .item(0)
            .unwrap();
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
