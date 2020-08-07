use crate::components::*;

#[derive(Model)]
pub struct Counter {
    pub count: usize,
}
#[controller]
impl CounterModel {
    pub fn increment(&mut self, _e: web_sys::Event) {
        set!(self.count => + 1);
    }
}
#[view(Counter)]
fn render() -> ViewComposite {
    let x = true;
    children! { if x {h1("My Counter")} else { h1("")},
        p(react!("The count is: {}", model.count)),
        p(&format!("The count starts at: {}", props.count)),
        button("+").c("increment"),
    }
}
