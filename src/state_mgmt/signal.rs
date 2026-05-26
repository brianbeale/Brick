use super::{Observer, Subject, make_placeholder, make_state};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// A handle to a reactive value. Cheap to clone (Rc increment). All reactive
/// fields on `#[model]`, `#[store]`, and `#[global]` structs are `Signal<T>`.
pub struct Signal<T: Clone + 'static>(pub(crate) Rc<RefCell<Box<dyn Subject<T>>>>);

impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal(make_state(value))
    }

    pub fn placeholder() -> Self {
        Signal(make_placeholder::<T>())
    }

    pub fn from_rc(rc: Rc<RefCell<Box<dyn Subject<T>>>>) -> Self {
        Signal(rc)
    }

    /// Read the current value (snapshot — not reactive).
    pub fn read(&self) -> T {
        self.0.borrow().read()
    }

    /// Replace the current value and notify all observers.
    pub fn set(&self, value: T) {
        {
            self.0.borrow_mut().write_datum(value);
        }
        self.0.borrow().notify_observers();
    }

    /// Mutate the current value in place and notify observers.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        let mut val = self.0.borrow().read();
        f(&mut val);
        {
            self.0.borrow_mut().write_datum(val);
        }
        self.0.borrow().notify_observers();
    }

    /// Returns a cloned Rc handle to the inner Subject — for macros and
    /// framework internals that still need the raw Rc.
    pub fn rc(&self) -> Rc<RefCell<Box<dyn Subject<T>>>> {
        Rc::clone(&self.0)
    }

    pub fn add_observer(&self, name: &str, obs: Box<dyn Observer<T>>) {
        self.0.borrow_mut().add_observer(name, obs);
    }

    /// Remove the observer registered under `name`, if any. Returns without
    /// error if the name is not found. Call this from `detach()` to prevent
    /// leaked closures keeping model data alive after a component is unmounted.
    pub fn remove_observer(&self, name: &str) {
        self.0.borrow_mut().remove_observer(name);
    }

    /// Return a new `Signal<U>` whose value is `f(&current)`, recomputed
    /// whenever this signal changes. Backed by the existing `compute` graph.
    pub fn map<U: Clone + 'static>(&self, f: impl Fn(&T) -> U + 'static) -> Signal<U> {
        Signal(super::compute(Rc::clone(&self.0), move |v| f(&v)))
    }
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal(Rc::clone(&self.0))
    }
}

impl<T: Clone + 'static + fmt::Display> fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.read())
    }
}

#[cfg(test)]
mod tests {
    use super::Signal;
    use crate::state_mgmt::Effect;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn new_and_read() {
        let s = Signal::new(42usize);
        assert_eq!(s.read(), 42);
    }

    #[test]
    fn set_updates_value() {
        let s = Signal::new(0usize);
        s.set(99);
        assert_eq!(s.read(), 99);
    }

    #[test]
    fn update_mutates_in_place() {
        let s = Signal::new(10usize);
        s.update(|v| *v += 5);
        assert_eq!(s.read(), 15);
    }

    #[test]
    fn clone_shares_underlying_state() {
        let a = Signal::new(0usize);
        let b = a.clone();
        a.set(7);
        assert_eq!(b.read(), 7);
    }

    #[test]
    fn observer_notified_on_set() {
        let out = Rc::new(RefCell::new(0usize));
        let out2 = Rc::clone(&out);
        let s = Signal::new(0usize);
        s.add_observer(
            "obs",
            Box::new(Effect::new(move |v| *out2.borrow_mut() = *v)),
        );
        s.set(42);
        assert_eq!(*out.borrow(), 42);
    }

    #[test]
    fn remove_observer_stops_notifications() {
        let out = Rc::new(RefCell::new(0usize));
        let out2 = Rc::clone(&out);
        let s = Signal::new(0usize);
        s.add_observer(
            "obs",
            Box::new(Effect::new(move |v| *out2.borrow_mut() = *v)),
        );
        s.set(10);
        assert_eq!(*out.borrow(), 10);
        s.remove_observer("obs");
        s.set(99);
        assert_eq!(
            *out.borrow(),
            10,
            "observer should no longer fire after removal"
        );
    }

    #[test]
    fn set_macro_works_with_signal() {
        let s = Signal::new(10usize);
        set!(s => + 5);
        assert_eq!(s.read(), 15);
        set!(s => - 3);
        assert_eq!(s.read(), 12);
        set!(s => * 2);
        assert_eq!(s.read(), 24);
        set!(s => / 4);
        assert_eq!(s.read(), 6);
        set!(s => 0);
        assert_eq!(s.read(), 0);
    }

    #[test]
    fn map_initial_value_is_transformed() {
        let src = Signal::new(3usize);
        let derived = src.map(|v| v * 2);
        assert_eq!(derived.read(), 6);
    }

    #[test]
    fn map_updates_when_source_changes() {
        let src = Signal::new(1usize);
        let derived = src.map(|v| v + 10);
        src.set(5);
        assert_eq!(derived.read(), 15);
    }

    #[test]
    fn map_observer_fires_on_source_change() {
        let out = Rc::new(RefCell::new(0usize));
        let out2 = Rc::clone(&out);
        let src = Signal::new(1usize);
        let derived = src.map(|v| v * 3);
        derived.add_observer(
            "obs",
            Box::new(Effect::new(move |v| *out2.borrow_mut() = *v)),
        );
        src.set(4);
        assert_eq!(*out.borrow(), 12);
    }

    #[test]
    fn map_type_transform() {
        let src = Signal::new(42usize);
        let s = src.map(|v| v.to_string());
        assert_eq!(s.read(), "42");
        src.set(99);
        assert_eq!(s.read(), "99");
    }

    #[test]
    #[should_panic(expected = "Placeholder subject read before wire_cascade")]
    fn placeholder_panics_on_read() {
        let p = Signal::<i32>::placeholder();
        let _ = p.read();
    }

    #[test]
    fn display_formats_inner_value() {
        let s = Signal::new(7usize);
        assert_eq!(format!("{s}"), "7");
    }

    #[test]
    fn from_rc_wraps_and_reads_back() {
        let s = Signal::new(42i32);
        let rc = s.rc();
        let s2 = Signal::from_rc(rc);
        assert_eq!(s2.read(), 42);
        s.set(99);
        assert_eq!(s2.read(), 99);
    }
}
