use super::super::{Effect, Subject};
use super::State;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static COMPUTED_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn compute<In, Out>(
    source: Rc<RefCell<Box<dyn Subject<In>>>>,
    func: impl Fn(In) -> Out + 'static,
) -> Rc<RefCell<Box<dyn Subject<Out>>>>
where
    In: Clone + 'static,
    Out: Clone + 'static,
{
    let initial = func(source.borrow().read());
    let derived: Rc<RefCell<Box<dyn Subject<Out>>>> =
        Rc::new(RefCell::new(Box::new(State::new(initial))));
    let derived_clone = Rc::clone(&derived);
    let name = format!(
        "computed_{}",
        COMPUTED_COUNTER.fetch_add(1, Ordering::SeqCst)
    );
    source.borrow_mut().add_observer(
        &name,
        Box::new(Effect::new(move |val: &In| {
            derived_clone.borrow_mut().update(func(val.clone()));
        })),
    );
    derived
}

#[cfg(test)]
mod tests {
    use super::compute;
    use crate::state_mgmt::{Effect, State, Subject};
    use std::{cell::RefCell, rc::Rc};

    fn make_source(initial: usize) -> Rc<RefCell<Box<dyn Subject<usize>>>> {
        Rc::new(RefCell::new(Box::new(State::new(initial))))
    }

    #[test]
    fn initial_value_is_transformed() {
        let source = make_source(5);
        let derived = compute(Rc::clone(&source), |v| v * 2);
        assert_eq!(derived.borrow().read(), 10);
    }

    #[test]
    fn updates_when_source_updates() {
        let source = make_source(3);
        let derived = compute(Rc::clone(&source), |v| v + 1);
        source.borrow_mut().update(9);
        assert_eq!(derived.borrow().read(), 10);
    }

    #[test]
    fn observers_on_derived_are_notified() {
        let source = make_source(1);
        let derived = compute(Rc::clone(&source), |v| v * 10);
        let captured = Rc::new(RefCell::new(0usize));
        let captured_clone = Rc::clone(&captured);
        derived.borrow_mut().add_observer(
            "watcher",
            Box::new(Effect::new(move |v| *captured_clone.borrow_mut() = *v)),
        );
        source.borrow_mut().update(4);
        assert_eq!(*captured.borrow(), 40);
    }

    #[test]
    fn type_transform_usize_to_string() {
        let source = make_source(7);
        let derived = compute(Rc::clone(&source), |v| format!("val={}", v));
        assert_eq!(derived.borrow().read(), "val=7");
        source.borrow_mut().update(42);
        assert_eq!(derived.borrow().read(), "val=42");
    }

    #[test]
    fn multiple_derived_from_same_source() {
        let source = make_source(5);
        let doubled = compute(Rc::clone(&source), |v| v * 2);
        let squared = compute(Rc::clone(&source), |v| v * v);
        source.borrow_mut().update(4);
        assert_eq!(doubled.borrow().read(), 8);
        assert_eq!(squared.borrow().read(), 16);
    }

    #[test]
    fn chained_computes() {
        let source = make_source(3);
        let doubled = compute(Rc::clone(&source), |v| v * 2);
        let quadrupled = compute(Rc::clone(&doubled), |v| v * 2);
        source.borrow_mut().update(5);
        assert_eq!(doubled.borrow().read(), 10);
        assert_eq!(quadrupled.borrow().read(), 20);
    }

    #[test]
    fn computed_macro_creates_derived() {
        let source = crate::state_mgmt::Signal::new(6usize);
        let derived = computed!(source, |v| v + 100);
        assert_eq!(derived.read(), 106);
        source.set(10);
        assert_eq!(derived.read(), 110);
    }
}
