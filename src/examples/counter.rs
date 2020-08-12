use crate::view_components::*;

#[model]
pub struct Counter {
    count: usize,
}
#[controller]
impl Counter {
    pub fn increment(&mut self, _e: web_sys::Event) {
        set!(self.count => + 1);
    }
}
#[view(Counter)]
fn render() -> Box<ViewComposite> {
    children! { h2("My Counter").c("classy"),
        p(react_text!("The count is: {}", model.count)),
        button("+").c("increment"),
    }
}
