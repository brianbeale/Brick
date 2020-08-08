pub use hermit_proc_macros::{controller, view, Model};

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

pub trait ViewComponent {
    fn html(&self) -> String;
}
pub trait Render {
    fn render(&self) -> ViewComposite;
}
