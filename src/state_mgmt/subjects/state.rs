use super::super::{Observer, Subject};

use std::collections::HashMap;
use std::{fmt::Display, ops::Deref};

pub struct State<T> {
    datum: T,
    observers: HashMap<String, Box<dyn Observer<T>>>,
}
impl<T> State<T> {
    pub fn new(datum: T) -> Self {
        State::<T> {
            datum,
            observers: HashMap::new(),
        }
    }
    fn refresh(&mut self) {
        for (_class_name, obs) in &mut self.observers {
            obs.notify(&self.datum);
        }
    }
}
impl<T: Clone> Subject<T> for State<T> {
    fn read(&self) -> T {
        self.datum.clone()
    }
    fn update(&mut self, new_datum: T) {
        self.datum = new_datum;
        self.refresh();
    }
    fn add_observer(&mut self, class_name: &str, obs: Box<dyn Observer<T>>) {
        self.observers.insert(class_name.to_string(), obs);
    }
    fn remove_observer(&mut self, class_name: &str) -> Option<Box<dyn Observer<T>>> {
        self.observers.remove(class_name)
    }
}
impl<T: Display> Display for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.datum.to_string())
    }
}
impl<T> Deref for State<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.datum
    }
}
