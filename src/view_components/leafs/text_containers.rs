use std::fmt::Display;

use super::super::ViewComponent;

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
        let mut class_insert = "".to_string();
        if self.class_name.len() > 0 {
            class_insert = format!(r#"class="{}""#, self.class_name);
        }
        format!(
            r#"<{} {}>{}</{}>"#,
            self.tag, class_insert, self.text_content, self.tag
        )
    }
    fn attach_listeners(&self) {}
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
tag_funcs!(p, h1, h2);
pub fn button<T: Display + ?Sized>(text: &T) -> Box<ViewLeafText> {
    Box::new(ViewLeafText {
        tag: "button",
        class_name: text.to_string(),
        text_content: text.to_string(),
    })
}
