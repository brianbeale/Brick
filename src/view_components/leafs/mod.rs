#![allow(unused_imports)]

// Generates a public constructor function for each named HTML element that
// accepts text content. The returned `Box<ViewLeafText>` supports the full
// builder API: `.c()`, `.attr()`, `.trigger()`, `.style()`, `.live()`, etc.
//
// Usage: `tag_funcs!(p, h1, div, span)` → `pub fn p(text: &T)`, `pub fn h1(text: &T)`, …
macro_rules! tag_funcs {
    ( $( $name:ident ),* ) => {
        $(
            pub fn $name<T: ::std::fmt::Display + ?Sized>(text: &T) -> Box<crate::view_components::leafs::ViewLeafText> {
                Box::new(crate::view_components::leafs::ViewLeafText::new(
                    stringify!($name),
                    text.to_string(),
                    false,
                ))
            }
        )*
    };
}

// Generates a public constructor function for each named void HTML element
// (elements with no text content or closing tag, e.g. `<input>`, `<br>`).
//
// Usage: `tag_funcs_void!(input, br, hr)` → `pub fn input()`, `pub fn br()`, …
macro_rules! tag_funcs_void {
    ( $( $name:ident ),* ) => {
        $(
            pub fn $name() -> Box<crate::view_components::leafs::ViewLeafText> {
                Box::new(crate::view_components::leafs::ViewLeafText::new(
                    stringify!($name),
                    String::new(),
                    true,
                ))
            }
        )*
    };
}

mod element;
pub use element::{IntoClass, IntoStyle, ViewLeafText};

mod block;
pub use block::*;

mod inline;
pub use inline::*;

mod lists;
pub use lists::*;

mod tables;
pub use tables::*;

mod forms;
pub use forms::*;

mod media;
pub use media::*;

mod spans;
pub use spans::*;

pub(crate) struct NothingComponent;
impl crate::view_components::Brick for NothingComponent {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        Renderer::append(parent, &Renderer::text(""));
    }
}

/// Returns an empty, no-op component. Used internally by `load!` and `idle!`
/// to fill unused arms without allocating any DOM content.
pub fn nothing() -> Box<dyn crate::view_components::Brick> {
    Box::new(NothingComponent)
}
