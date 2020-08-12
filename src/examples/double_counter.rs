use super::Counter;
use crate::view_components::*;

#[model]
pub struct DoubleCounter {
    count: usize,
}

// TODO: controller should be optional
#[controller]
impl DoubleCounter {}

#[view(DoubleCounter)]
fn render() -> Box<ViewComposite> {
    children! { h1("Double Counter"),
        Counter { count: bind!(model.count) }.render(),
        Counter { count: bind!(model.count) }.render().c("Counter_2"),
    }
}
