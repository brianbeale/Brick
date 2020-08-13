use super::super::ViewComponent;
use crate::state_mgmt::{SpanObserver, State, Subject};
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

static SPAN_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct Span {
    pub class_name: String,
    pub text_content: State<String>,
}
impl Span {
    pub fn new<T: Display>(text: &T) -> Self {
        let class_name = format!("span_{}", SPAN_COUNTER.fetch_add(1, Ordering::SeqCst));
        let mut text_content = State::new(text.to_string());
        text_content.add_observer(&class_name, Box::new(SpanObserver::new(&class_name)));
        Span {
            class_name,
            text_content,
        }
    }
}
impl ViewComponent for Span {
    fn html(&self) -> String {
        format!(
            r#"<span class="{}">{}</span>"#,
            self.class_name, *self.text_content
        )
    }
    fn attach_listeners(&self) {}
}
