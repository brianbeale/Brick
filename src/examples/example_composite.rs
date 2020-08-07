#![allow(unused)]
use crate::components::*;
mod counter;
use counter::Counter;

#[derive(Model)]
pub struct DoubleCounter {
    count: usize,
    counter_1: Counter,
    counter_2: Counter,
}

#[view(DoubleCounter)]
fn render(mut self: Box<Self>) -> ViewManager {
        
}
