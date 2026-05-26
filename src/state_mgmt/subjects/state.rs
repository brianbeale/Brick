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
    fn refresh(&self) {
        for (_class_name, obs) in &self.observers {
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
    fn write_datum(&mut self, new_datum: T) {
        self.datum = new_datum;
    }
    fn notify_observers(&self) {
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

#[cfg(test)]
mod tests {
    use super::State;
    use crate::state_mgmt::{Effect, Subject};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn read_returns_initial_value() {
        let s = State::new(42usize);
        assert_eq!(s.read(), 42);
    }

    #[test]
    fn update_changes_value() {
        let mut s = State::new(0usize);
        s.update(99);
        assert_eq!(s.read(), 99);
    }

    #[test]
    fn observer_notified_on_update() {
        let notified = Rc::new(RefCell::new(0usize));
        let notified_clone = Rc::clone(&notified);
        let mut s = State::new(0usize);
        s.add_observer(
            "obs",
            Box::new(Effect::new(move |v| *notified_clone.borrow_mut() = *v)),
        );
        s.update(7);
        assert_eq!(*notified.borrow(), 7);
    }

    #[test]
    fn multiple_observers_all_notified() {
        let a = Rc::new(RefCell::new(0usize));
        let b = Rc::new(RefCell::new(0usize));
        let a_clone = Rc::clone(&a);
        let b_clone = Rc::clone(&b);
        let mut s = State::new(0usize);
        s.add_observer(
            "a",
            Box::new(Effect::new(move |v| *a_clone.borrow_mut() = *v)),
        );
        s.add_observer(
            "b",
            Box::new(Effect::new(move |v| *b_clone.borrow_mut() = *v)),
        );
        s.update(5);
        assert_eq!(*a.borrow(), 5);
        assert_eq!(*b.borrow(), 5);
    }

    #[test]
    fn remove_observer_stops_notifications() {
        let call_count = Rc::new(RefCell::new(0u32));
        let count_clone = Rc::clone(&call_count);
        let mut s = State::new(0usize);
        s.add_observer(
            "obs",
            Box::new(Effect::new(move |_| *count_clone.borrow_mut() += 1)),
        );
        s.update(1);
        s.remove_observer("obs");
        s.update(2);
        assert_eq!(*call_count.borrow(), 1);
    }

    #[test]
    fn set_macro_addition() {
        let s = crate::state_mgmt::Signal::new(10usize);
        set!(s => + 5);
        assert_eq!(s.read(), 15);
    }

    #[test]
    fn set_macro_subtraction() {
        let s = crate::state_mgmt::Signal::new(10usize);
        set!(s => - 3);
        assert_eq!(s.read(), 7);
    }

    #[test]
    fn set_macro_multiplication() {
        let s = crate::state_mgmt::Signal::new(4usize);
        set!(s => * 3);
        assert_eq!(s.read(), 12);
    }

    #[test]
    fn set_macro_division() {
        let s = crate::state_mgmt::Signal::new(20usize);
        set!(s => / 4);
        assert_eq!(s.read(), 5);
    }

    #[test]
    fn set_macro_direct_assignment() {
        let s = crate::state_mgmt::Signal::new(0usize);
        set!(s => 42);
        assert_eq!(s.read(), 42);
    }

    #[test]
    fn set_macro_notifies_observers() {
        let notified = Rc::new(RefCell::new(0usize));
        let notified_clone = Rc::clone(&notified);
        let s = crate::state_mgmt::Signal::new(0usize);
        s.add_observer(
            "obs",
            Box::new(Effect::new(move |v| *notified_clone.borrow_mut() = *v)),
        );
        set!(s => + 10);
        assert_eq!(*notified.borrow(), 10);
    }
}
