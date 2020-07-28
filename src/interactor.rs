#![allow(unused)]

use crate::dom_context::DomContext;
use crate::rendering::{CreateModel, Render, ViewManager};
use crate::util::_print_type_of;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub struct Hermit {
    pub root_manager: ViewManager,
    pub dom_context: DomContext,
}

impl Hermit {
    pub fn new<T: CreateModel>(root: T) -> Self {
        Hermit {
            root_manager: root.to_model().render(),
            dom_context: DomContext::new(),
        }
    }
    pub fn run(&mut self) {
        let callbacks = self.root_manager.mount_to_body(&self.dom_context);
        // .expect("should be able to mount root component");
        // loop {}
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        // _print_type_of(&callbacks[0]);

        let dc = DomContext::new();
        let mut i = 0;
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if i > 300 {
                dc.body.set_text_content(Some("All done!"));

                // Drop our handle to this closure so that it will get cleaned
                // up once we return.
                let _ = f.borrow_mut().take();
                return;
            }
            // Set the body's text content to how many times this
            // requestAnimationFrame callback has fired.
            // i += 1;
            // let text = format!("requestAnimationFrame has been called {} times.", i);
            // dc.body.set_text_content(Some(&text));

            // Schedule ourself for another requestAnimationFrame callback.
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
