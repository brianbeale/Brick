use crate::view_components::{Render, ViewComposite};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};

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
}

pub struct Hermit {
    pub root_manager: ViewComposite,
    pub dom_context: DomContext,
}

impl Hermit {
    pub fn new<T: Render>(root: T) -> Self {
        Hermit {
            root_manager: Box::new(root).render(),
            dom_context: DomContext::new(),
        }
    }
    pub fn run(mut self) {
        self.root_manager.mount(&self.dom_context.body);
        let shared_root = Rc::new(RefCell::new(self.root_manager));
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let _root = Rc::clone(&shared_root);

            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("need a window object")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
