#[allow(unused)]
use super::{Observer, State, Subject};
use std::{cell::RefCell, rc::Rc};

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

use std::sync::atomic::{AtomicUsize, Ordering};

static BINDER_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct StateBinder<T: Clone> {
    name: String,
    subject_a: Rc<RefCell<State<T>>>,
    subject_b: Rc<RefCell<State<T>>>,
}
impl<T: Clone + 'static> StateBinder<T> {
    pub fn new(subject_a: Rc<RefCell<State<T>>>, subject_b: Rc<RefCell<State<T>>>) -> Self {
        let name = format!("binder_{}", BINDER_COUNTER.fetch_add(1, Ordering::SeqCst));

        subject_a.borrow_mut().add_observer(
            &name,
            Box::new(Effect::new(move |x: &T| {
                Rc::clone(&subject_b).borrow_mut().update(x.clone())
            })),
        );
        subject_b.borrow_mut().add_observer(
            &name,
            Box::new(Effect::new(move |x: &T| {
                Rc::clone(&subject_a).borrow_mut().update(x.clone())
            })),
        );
        StateBinder {
            name,

            subject_a,
            subject_b,
        }
    }
}
