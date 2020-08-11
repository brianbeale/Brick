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
    let count = sharing_model.borrow().count.clone();
    children! { h2("My Counter").c("classy"),
        p(react!("The count is: {}", count)),
        button("+").c("increment"),
    }
}
