use crate::rendering::{Render, ViewManager};
use wasm_bindgen::prelude::*;
#[macro_use]
use crate::{rendering, state_manager};
// #[Props]
pub struct MyCounter {
    initial_count: usize,
}

pub struct Model {
    count: usize,
}

impl Model {
    pub fn new(mc: MyCounter) -> Self {
        Model {
            count: mc.initial_count,
        }
    }
}

// /#[Controller]
impl Model {
    fn button_click(&mut self, e: web_sys::Event) {
        set!(count += 1); // expands to: self.count.publish(self.count+1);
        console_log!(format!("The count is {}", self.count.to_string()));
    }
}

impl Render for Model {
    fn render(self) -> ViewManager {
        view! {
            h1("My Counter Component"),
            p("the count is just so: {self.count}"),
            button("Increment")
        }
    }
}
