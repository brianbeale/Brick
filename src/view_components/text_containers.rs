use std::fmt::Display;

use super::ViewComponent;

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
        format!(
            r#"<{} class="{}">{}</{}>"#,
            self.tag, self.class_name, self.text_content, self.tag
        )
    }
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
tag_funcs!(p, h1);
pub fn button<T: Display + ?Sized>(text: &T) -> Box<ViewLeafText> {
    Box::new(ViewLeafText {
        tag: "button",
        class_name: text.to_string(),
        text_content: text.to_string(),
    })
}
