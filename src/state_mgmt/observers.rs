use std::{cell::RefCell, fmt::Display};

use super::Observer;

#[cfg(brick_dom)]
fn find_by_class(class_name: &str) -> Option<web_sys::Element> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_elements_by_class_name(class_name)
        .item(0)
}

/// Updates an `<input>` element's value whenever the observed signal changes.
/// The element is located by its CSS class name.
pub struct InputObserver {
    class_name: String,
}
impl InputObserver {
    pub fn new(class_name: &str) -> Self {
        InputObserver {
            class_name: class_name.to_string(),
        }
    }
}
impl<T: Display> Observer<T> for InputObserver {
    fn notify(&self, _datum: &T) {
        #[cfg(brick_dom)]
        {
            use wasm_bindgen::JsCast;
            if let Some(el) = find_by_class(&self.class_name) {
                if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                    input.set_value(&_datum.to_string());
                }
            }
        }
    }
}

/// Updates a text node's content whenever the observed signal changes.
/// The element is located by its CSS class name.
#[allow(dead_code)]
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
    fn notify(&self, _datum: &T) {
        #[cfg(brick_dom)]
        {
            find_by_class(&self.class_name)
                .unwrap()
                .set_text_content(Some(&_datum.to_string()));
        }
    }
}

/// Updates an `<input type="checkbox">` element's checked state whenever
/// the observed `bool` signal changes. The element is located by CSS class name.
pub struct CheckedObserver {
    class_name: String,
}
impl CheckedObserver {
    pub fn new(class_name: &str) -> Self {
        CheckedObserver {
            class_name: class_name.to_string(),
        }
    }
}
impl Observer<bool> for CheckedObserver {
    fn notify(&self, _datum: &bool) {
        #[cfg(brick_dom)]
        {
            use wasm_bindgen::JsCast;
            if let Some(el) = find_by_class(&self.class_name) {
                if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                    input.set_checked(*_datum);
                }
            }
        }
    }
}

/// Updates a single CSS class on a DOM element identified by a stable class name.
/// Removes the previous reactive class and adds the new one on each notify.
pub struct ClassObserver {
    element_class: String,
    current: RefCell<String>,
}
impl ClassObserver {
    pub fn new(element_class: &str, initial: &str) -> Self {
        ClassObserver {
            element_class: element_class.to_string(),
            current: RefCell::new(initial.to_string()),
        }
    }
}
impl Observer<String> for ClassObserver {
    fn notify(&self, _new_class: &String) {
        #[cfg(brick_dom)]
        {
            use wasm_bindgen::JsCast;
            if let Some(el) = find_by_class(&self.element_class) {
                if let Ok(el) = el.dyn_into::<web_sys::Element>() {
                    let existing = el.get_attribute("class").unwrap_or_default();
                    let stripped: String = existing
                        .split_whitespace()
                        .filter(|&c| c != self.current.borrow().as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let updated = if _new_class.is_empty() {
                        stripped
                    } else if stripped.is_empty() {
                        _new_class.clone()
                    } else {
                        format!("{} {}", stripped, _new_class)
                    };
                    el.set_attribute("class", &updated).ok();
                    *self.current.borrow_mut() = _new_class.clone();
                }
            }
        }
    }
}

/// A general-purpose observer that runs an arbitrary closure on each notification.
///
/// `Effect` accepts `FnMut` so the callback can accumulate state across calls.
/// Use it anywhere you need a side-effect tied to a `Signal<T>` change that
/// isn't covered by the specialized observers (`InputObserver`, `ClassObserver`,
/// etc.).
///
/// ```rust,ignore
/// signal.add_observer("log", Box::new(Effect::new(|val| {
///     web_sys::console::log_1(&format!("changed: {val}").into());
/// })));
/// ```
pub struct Effect<T> {
    callback: RefCell<Box<dyn FnMut(&T)>>,
}
impl<T> Effect<T> {
    pub fn new<F: FnMut(&T) + 'static>(callback: F) -> Self {
        Effect {
            callback: RefCell::new(Box::new(callback)),
        }
    }
}
impl<T> Observer<T> for Effect<T> {
    fn notify(&self, datum: &T) {
        (self.callback.borrow_mut())(datum);
    }
}
