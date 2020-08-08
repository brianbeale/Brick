#[macro_use]
pub mod set_state;
#[macro_use]
pub mod react_macros;

mod observers;
pub use observers::*;
mod subjects;
pub use subjects::*;

pub trait Observer<T> {
    fn notify(&mut self, datum: &T);
}
pub trait Subject<T> {
    fn update(&mut self, new_datum: T);
    fn add_observer(&mut self, name: &str, obs: Box<dyn Observer<T>>);
    fn remove_observer(&mut self, name: &str) -> Option<Box<dyn Observer<T>>>;
}
