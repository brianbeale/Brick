#![allow(unused)]

pub struct DomContext {
    pub window: web_sys::Window,
    pub document: web_sys::Document,
    pub body: web_sys::HtmlElement,
}

impl DomContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("window object not found");
        let document = window.document().expect("no document found");
        let body = document.body().expect("document should have a body");

        DomContext {
            window,
            document,
            body,
        }
    }

    pub fn create_with_append(&self, tag: &str, inner: &str) -> web_sys::Element {
        let el = self.document.create_element(tag).unwrap();
        el.set_inner_html(inner);
        self.body.append_child(&el).unwrap();
        el
    }

    pub fn new_element(&self, tag: &str) -> web_sys::Element {
        self.document.create_element(tag).unwrap()
    }
}
