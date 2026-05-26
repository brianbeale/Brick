use brick_proc_macros::controller;

struct MyModel;

#[controller]
impl MyModel {
    #[on(watch)]
    fn on_change(&mut self) {}
}

fn main() {}
