use super::{Observer, State, Subject};
use std::{cell::RefCell, rc::Rc};

use std::sync::atomic::{AtomicUsize, Ordering};

static BOUND_STATE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct BoundState<T: Clone> {
    name: String,
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
        let x = self.inner_state.borrow().read();
        x
    }
    fn update(&mut self, new_datum: T) {
        self.outer_state.borrow_mut().update(new_datum);
    }
    fn add_observer(&mut self, name: &str, obs: Box<dyn Observer<T>>) {
        self.inner_state.borrow_mut().add_observer(name, obs);
    }
    fn remove_observer(&mut self, name: &str) -> Option<Box<dyn Observer<T>>> {
        self.inner_state.borrow_mut().remove_observer(name)
    }
}

pub struct Effect<T> {
    callback: Box<dyn FnMut(&T)>,
}
impl<T> Effect<T> {
    pub fn new<F: FnMut(&T) + 'static>(callback: F) -> Self {
        Effect {
            callback: Box::new(callback),
        }
    }
}
impl<T> Observer<T> for Effect<T> {
    fn notify(&mut self, datum: &T) {
        (self.callback)(datum);
    }
}
