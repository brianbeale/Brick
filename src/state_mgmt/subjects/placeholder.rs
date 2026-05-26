use super::super::{Observer, Subject};
use std::marker::PhantomData;

pub struct Placeholder<T>(PhantomData<T>);

impl<T> Placeholder<T> {
    pub fn new() -> Self {
        Placeholder(PhantomData)
    }
}

impl<T> Subject<T> for Placeholder<T> {
    fn read(&self) -> T {
        panic!("Placeholder subject read before wire_cascade — did controller_methods() run?")
    }
    fn update(&mut self, _: T) {}
    fn write_datum(&mut self, _: T) {}
    fn notify_observers(&self) {}
    fn add_observer(&mut self, _: &str, _: Box<dyn Observer<T>>) {}
    fn remove_observer(&mut self, _: &str) -> Option<Box<dyn Observer<T>>> {
        None
    }
}
