use super::Counter;
use crate::view_components::*;

#[model]
pub struct DoubleCounter {
    pub count: isize,
}

#[view(DoubleCounter)]
fn render() -> Box<ViewComposite> {
    children! {
        h2("Shared State"),
        Counter { count: bind!(my.count) },
        Counter { count: bind!(my.count) },
    }
}
