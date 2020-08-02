#![allow(unused)]

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
mod verbose_ex;
use verbose_ex::MyCounter;

pub struct MyDoubleCounter {
    count: usize,
    counter_1: MyCounter,
    counter_2: MyCounter,
}

pub struct Model {
    pub count: State<usize>,
}
impl Model {
    pub fn new(&mdc: MyDoubleCounter) -> Self {
        Model {
            count: State::new(mdc.count),
        }
    }
}

impl Render for MyDoubleCounter {
    fn render(mut self: Box<Self>) -> ViewManager {
        let model = Rc::new(RefCell::new(Model::new(&self)));

        let html = format!(
            r#"<div class="MyDoubleCounter">{}{}</div>"#,
            Box::new(self.counter_1).render().html,
            Box::new(self.counter_2).render().html
        );

        ViewManager {
            html,
            listening_classes: vec![],
            event_listeners: HashMap::new(),
        }
    }
}
