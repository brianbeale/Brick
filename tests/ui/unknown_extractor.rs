use brick_proc_macros::controller;

struct MyModel;

#[controller]
impl MyModel {
    #[on(click, bogus)]
    fn handle_click(&mut self) {}
}

fn main() {}
