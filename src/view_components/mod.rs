pub use hermit_proc_macros::{controller, model, view, Model};

#[macro_use]
pub mod view_helpers;

mod composites;
pub use composites::*;

mod text_containers;
pub use text_containers::*;

mod spans;
pub use spans::*;

mod media;
pub use media::*;

mod custom;
pub use custom::*;

pub trait ViewComponent {
    fn html(&self) -> String;
    fn attach_listeners(&self);
}
pub trait Render {
    fn render(self) -> Box<ViewComposite>;
}

// Each html Element (and potentially custom built-ins) should have
// own top-level constructor and chainable builder methods
