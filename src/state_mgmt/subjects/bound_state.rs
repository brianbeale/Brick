#![allow(dead_code)]
use super::super::{Effect, Observer, State, Subject};
use std::{cell::RefCell, rc::Rc};

use std::sync::atomic::{AtomicUsize, Ordering};

static BOUND_STATE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct BoundState<T: Clone> {
    name: String, // TODO: impl Drop
    inner_state: Rc<RefCell<Box<dyn Subject<T>>>>,
    outer_state: Rc<RefCell<Box<dyn Subject<T>>>>,
}
impl<T: Clone + 'static> BoundState<T> {
    pub fn new(outer_state: Rc<RefCell<Box<dyn Subject<T>>>>) -> Self {
        let name = format!(
            "bound_state_{}",
            BOUND_STATE_COUNTER.fetch_add(1, Ordering::SeqCst)
        );

        let inner_state = Rc::new(RefCell::new(
            Box::new(State::new(outer_state.borrow_mut().read())) as Box<dyn Subject<T>>,
        ));
        let inner_state_clone = Rc::clone(&inner_state);
        outer_state.borrow_mut().add_observer(
            &name,
            Box::new(Effect::new(move |x: &T| {
                inner_state_clone.borrow_mut().update(x.clone())
            })),
        );
        BoundState {
            name,
            inner_state,
            outer_state,
        }
    }
}
impl<T: Clone + 'static> Subject<T> for BoundState<T> {
    fn read(&self) -> T {
        self.inner_state.borrow().read()
    }
    fn update(&mut self, new_datum: T) {
        self.outer_state.borrow_mut().update(new_datum);
    }
    fn write_datum(&mut self, new_datum: T) {
        self.outer_state.borrow_mut().write_datum(new_datum.clone());
        self.inner_state.borrow_mut().write_datum(new_datum);
    }
    fn notify_observers(&self) {
        self.inner_state.borrow().notify_observers();
    }
    fn add_observer(&mut self, name: &str, obs: Box<dyn Observer<T>>) {
        self.inner_state.borrow_mut().add_observer(name, obs);
    }
    fn remove_observer(&mut self, name: &str) -> Option<Box<dyn Observer<T>>> {
        self.inner_state.borrow_mut().remove_observer(name)
    }
}

#[cfg(test)]
mod tests {
    use super::BoundState;
    use crate::state_mgmt::{Effect, State, Subject};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn make_outer(val: usize) -> Rc<RefCell<Box<dyn Subject<usize>>>> {
        Rc::new(RefCell::new(
            Box::new(State::new(val)) as Box<dyn Subject<usize>>
        ))
    }

    #[test]
    fn read_mirrors_outer_initial_value() {
        let outer = make_outer(7);
        let bound = BoundState::new(Rc::clone(&outer));
        assert_eq!(bound.read(), 7);
    }

    #[test]
    fn update_propagates_to_outer() {
        let outer = make_outer(0);
        let mut bound = BoundState::new(Rc::clone(&outer));
        bound.update(42);
        assert_eq!(outer.borrow().read(), 42);
    }

    #[test]
    fn outer_update_propagates_to_inner() {
        let outer = make_outer(0);
        let bound = BoundState::new(Rc::clone(&outer));
        outer.borrow_mut().update(99);
        assert_eq!(bound.read(), 99);
    }

    #[test]
    fn two_bound_states_both_receive_outer_updates() {
        let outer = make_outer(0);
        let bound_a = BoundState::new(Rc::clone(&outer));
        let bound_b = BoundState::new(Rc::clone(&outer));
        outer.borrow_mut().update(55);
        assert_eq!(bound_a.read(), 55);
        assert_eq!(bound_b.read(), 55);
    }

    #[test]
    fn bound_observer_notified_when_outer_updates() {
        let outer = make_outer(0);
        let mut bound = BoundState::new(Rc::clone(&outer));
        let notified = Rc::new(RefCell::new(0usize));
        let notified_clone = Rc::clone(&notified);
        bound.add_observer(
            "obs",
            Box::new(Effect::new(move |v| *notified_clone.borrow_mut() = *v)),
        );
        outer.borrow_mut().update(21);
        assert_eq!(*notified.borrow(), 21);
    }

    #[test]
    fn chained_bound_states_propagate() {
        let outer = make_outer(0);
        let mid = Rc::new(RefCell::new(
            Box::new(BoundState::new(Rc::clone(&outer))) as Box<dyn Subject<usize>>
        ));
        let leaf = BoundState::new(Rc::clone(&mid));
        outer.borrow_mut().update(33);
        assert_eq!(leaf.read(), 33);
    }
}
