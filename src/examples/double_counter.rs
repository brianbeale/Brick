use super::Counter;
use crate::view_components::*;

#[model]
pub struct DoubleCounter {
    count: usize,
}

#[controller]
impl DoubleCounter {}

#[view(DoubleCounter)]
fn render() -> Box<ViewComposite> {
    let count = sharing_model.borrow().count.clone();
    children! { h1("Double Counter"),
        Counter { count: bind!(count) }.render(),
        Counter { count: bind!(count) }.render().c("Counter_2"),
    }
}
