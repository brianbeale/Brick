use std::fmt::Display;

use super::Observer;

pub struct SpanObserver {
    class_name: String,
}
impl SpanObserver {
    pub fn new(class_name: &str) -> Self {
        SpanObserver {
            class_name: class_name.to_string(),
        }
    }
}
impl<T: Display> Observer<T> for SpanObserver {
    fn notify(&mut self, datum: &T) {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_elements_by_class_name(&self.class_name)
            .item(0)
            .unwrap()
            .set_text_content(Some(&datum.to_string()));
    }
}

pub struct Effect<T> {
    callback: Box<dyn FnMut(&T)>,
}
impl<T> Effect<T> {
    pub fn new<F: FnMut(&T) + 'static>(callback: F) -> Self {
        Effect {
            callback: Box::new(callback),
        }
    }
}
impl<T> Observer<T> for Effect<T> {
    fn notify(&mut self, datum: &T) {
        (self.callback)(datum);
    }
}
