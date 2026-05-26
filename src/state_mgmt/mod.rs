pub use crate::validation::Validate;

#[macro_use]
pub mod set_state;
#[macro_use]
pub mod state_constructors;
#[macro_use]
pub mod react_macros;

pub mod observers;
pub use observers::*;
mod subjects;
pub use subjects::*;
mod list;
pub use list::*;
mod signal;
pub use signal::Signal;

mod spring;
pub use spring::spring;

mod resource;
pub use resource::Resource;

/// Identifies a controller method that can be wired to a DOM event.
///
/// The `#[controller]` macro generates a `BrickAction` constant for every
/// handler method, carrying the method name and the DOM event type it listens
/// for. Pass it to `.trigger()`, `.submit()`, or `.on_event()` in views.
///
/// ```rust,ignore
/// // In a view:
/// button("Save").trigger(&my.save)
/// input().trigger(&my.update_name)  // reads action.event → "input" vs "click"
/// ```
#[derive(Copy, Clone)]
pub struct BrickAction {
    pub name: &'static str,
    pub event: &'static str,
}

/// Lifecycle state for an async data operation.
///
/// Declare model fields as `Load<MyType>` — the `#[model]` macro wraps it in
/// `Signal<Load<T, E>>` automatically. Use `wait!`, `catch!`, and `load!` in views to branch on state.
#[derive(Clone, PartialEq)]
pub enum Load<T, E = crate::ssr::BrickError> {
    Idle,
    Loading,
    Loaded(T),
    Failed(E),
}

impl<T: Clone, E: Clone> Load<T, E> {
    pub fn is_idle(&self) -> bool {
        matches!(self, Load::Idle)
    }
    pub fn is_loading(&self) -> bool {
        matches!(self, Load::Loading)
    }
    pub fn is_loaded(&self) -> bool {
        matches!(self, Load::Loaded(_))
    }
    pub fn is_failed(&self) -> bool {
        matches!(self, Load::Failed(_))
    }
    pub fn loaded(&self) -> Option<&T> {
        if let Load::Loaded(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn err(&self) -> Option<&E> {
        if let Load::Failed(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

#[cfg(brick_dom)]
pub use wasm_bindgen_futures::spawn_local;

/// Inject global store handles as local variables. Works in views, controllers,
/// and any function. Each name becomes a local holding `TypeName::get()`.
///
/// ```rust,ignore
/// globals!(auth: AuthStore, theme: ThemeStore);
/// // equivalent to:
/// let auth = AuthStore::get();
/// let theme = ThemeStore::get();
/// ```
#[macro_export]
macro_rules! globals {
    ( $( $name:ident : $ty:ty ),+ $(,)? ) => {
        $( let $name = <$ty>::get(); )+
    };
}

/// Blanket fallback for `wrap_for_view()`. Every model gets this by default;
/// `#[controller]` overrides it with an inherent method returning a view wrapper.
pub trait BrickWrapDefault: Sized {
    fn wrap_for_view(self) -> Self {
        self
    }
}
impl<T: Sized> BrickWrapDefault for T {}

/// Lifecycle callbacks for a mounted component. Returned by `controller_methods()`
/// and consumed by `make_composite!` to wire them into the `ViewComposite`.
pub struct BrickLifecycle {
    /// Runs once after the component's HTML is in the DOM and listeners are attached.
    pub on_mount: Vec<std::rc::Rc<dyn Fn()>>,
    /// Runs at the start of `detach()`, before children are torn down.
    pub on_before_unmount: Vec<std::rc::Rc<dyn Fn()>>,
    /// Runs at the end of `detach()`, after children and intervals are cleaned up.
    pub on_unmount: Vec<std::rc::Rc<dyn Fn()>>,
}

impl Default for BrickLifecycle {
    fn default() -> Self {
        BrickLifecycle {
            on_mount: Vec::new(),
            on_before_unmount: Vec::new(),
            on_unmount: Vec::new(),
        }
    }
}

pub trait BrickModel: Sized + Clone {
    fn wire_cascade(&mut self);

    fn controller_methods(
        mut self,
    ) -> (
        std::collections::HashMap<
            &'static str,
            (&'static str, std::rc::Rc<dyn Fn(crate::renderer::BrickEvent)>),
        >,
        BrickLifecycle,
        Vec<(i32, std::rc::Rc<dyn Fn()>)>,
        std::rc::Rc<std::cell::RefCell<Self>>,
        Vec<std::rc::Rc<dyn Fn()>>,
    ) {
        self.wire_cascade();
        (
            std::collections::HashMap::new(),
            BrickLifecycle::default(),
            Vec::new(),
            std::rc::Rc::new(std::cell::RefCell::new(self)),
            Vec::new(),
        )
    }
}

pub fn make_state<T: Clone + 'static>(
    value: T,
) -> std::rc::Rc<std::cell::RefCell<Box<dyn Subject<T>>>> {
    std::rc::Rc::new(std::cell::RefCell::new(
        Box::new(State::new(value)) as Box<dyn Subject<T>>
    ))
}

pub fn make_placeholder<T: 'static>() -> std::rc::Rc<std::cell::RefCell<Box<dyn Subject<T>>>> {
    std::rc::Rc::new(std::cell::RefCell::new(
        Box::new(Placeholder::<T>::new()) as Box<dyn Subject<T>>
    ))
}

pub trait Cascade: Sized {
    fn cascade_defaults() -> Self;
}

pub fn cascade<T: Cascade>() -> T {
    T::cascade_defaults()
}

/// Receives notifications when a `Subject<T>` value changes. Use `Effect` for
/// arbitrary side-effects; prefer the typed observer structs in `observers` for
/// common DOM update patterns.
pub trait Observer<T> {
    fn notify(&self, datum: &T);
}

/// A reactive value that notifies registered `Observer<T>`s on change.
///
/// `Signal<T>` is the primary user-facing wrapper. The lower-level `State`,
/// `BoundState`, and `Computed` types all implement this trait.
///
/// `update` is the standard mutation path (stores new value and notifies).
/// `write_datum` + `notify_observers` is the two-phase split used internally
/// by `Signal::set` to avoid `RefCell` reentrancy when an observer itself
/// triggers another update.
pub trait Subject<T> {
    fn read(&self) -> T;
    fn update(&mut self, new_datum: T);
    /// Store `new_datum` without notifying observers.
    fn write_datum(&mut self, new_datum: T);
    /// Notify all registered observers with the current datum.
    fn notify_observers(&self);
    fn add_observer(&mut self, name: &str, obs: Box<dyn Observer<T>>);
    fn remove_observer(&mut self, name: &str) -> Option<Box<dyn Observer<T>>>;
}

#[cfg(test)]
mod tests {
    use super::Load;

    #[test]
    fn is_loading_true_for_loading() {
        let l: Load<String> = Load::Loading;
        assert!(l.is_loading());
        assert!(!Load::<String>::Idle.is_loading());
        assert!(!Load::<String, String>::Loaded("x".to_string()).is_loading());
        assert!(!Load::<String, String>::Failed("e".to_string()).is_loading());
    }

    #[test]
    fn is_loaded_true_for_loaded() {
        let l: Load<u32> = Load::Loaded(42u32);
        assert!(l.is_loaded());
        assert!(!Load::<u32>::Loading.is_loaded());
    }

    #[test]
    fn is_failed_true_for_failed() {
        let l: Load<String, String> = Load::Failed("boom".to_string());
        assert!(l.is_failed());
        assert!(!Load::<String>::Loading.is_failed());
    }

    #[test]
    fn loaded_returns_value() {
        let l: Load<u32> = Load::Loaded(99u32);
        assert_eq!(l.loaded(), Some(&99u32));
        assert_eq!(Load::<u32>::Loading.loaded(), None);
    }

    #[test]
    fn err_returns_error() {
        let l: Load<String, String> = Load::Failed("oops".to_string());
        assert_eq!(l.err(), Some(&"oops".to_string()));
        assert_eq!(Load::<String>::Loading.err(), None);
    }
}
