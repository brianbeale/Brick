// Prepare methods from Rc<RefCell<Model>> to be moved to JS
#[macro_export]
macro_rules! method {
    ( $method_name:ident, $sharing_model:expr ) => {{
        let s = std::rc::Rc::clone(&$sharing_model);
        wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
            s.borrow_mut().$method_name(e)
        }) as Box<dyn FnMut(web_sys::Event)>)
    }};
}
