use crate::components::Render;
use crate::components::ViewComposite;
use crate::dom_context::DomContext;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
            // if i > 300 {
            //     dc.body.set_text_content(Some("All done!"));

            //     // Drop our handle to this closure so that it will get cleaned
            //     // up once we return.
            //     let _ = f.borrow_mut().take();
            //     return;
            // }
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
