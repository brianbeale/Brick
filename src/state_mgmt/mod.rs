#[macro_use]
pub mod set_state;
#[macro_use]
pub mod state_constructors;
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
    fn read(&self) -> T;
    fn update(&mut self, new_datum: T);
    fn add_observer(&mut self, name: &str, obs: Box<dyn Observer<T>>);
    fn remove_observer(&mut self, name: &str) -> Option<Box<dyn Observer<T>>>;
}

// Side effects will probably be Observers,
// Stores will be subjects

// Parent-Child communication may require Mediation
// to coordinate complex update semantics
// DagChangeManager?

// Computed Properties will need to have Observer and Subject trait implementors
