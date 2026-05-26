use crate::ssr::BrickError;
#[cfg(brick_dom)]
use crate::state_mgmt::spawn_local;
use crate::state_mgmt::{Load, Signal};
use std::rc::Rc;

/// A self-fetching async data source backed by a `Signal<Load<T, BrickError>>`.
///
/// `Resource::new` accepts any async function that returns `Result<T, E>` where
/// `E: Into<BrickError>`, immediately kicks off the initial fetch, and exposes the
/// result as a reactive signal that views can read via `wait!`, `catch!`, or `load!`.
///
/// ```rust,ignore
/// let articles = Resource::new(|| async {
///     fetch_articles().await
/// });
/// // In a view:
/// wait!(articles.signal(), p("Loading…"),
///     catch!(articles.signal(), |e| p(&e.read()),
///         load!(articles.signal(), |data| p(&data.title))
///     )
/// )
/// ```
pub struct Resource<T: Clone + 'static> {
    signal: Signal<Load<T, BrickError>>,
    fetcher: Rc<dyn Fn()>,
}

impl<T: Clone + 'static> Resource<T> {
    /// Create a new `Resource` from an async fetch function.
    ///
    /// The fetch function is called immediately to kick off the initial load.
    /// The signal starts as `Load::Loading`.
    pub fn new<F, Fut, E>(fetch: F) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: std::future::Future<Output = Result<T, E>> + 'static,
        E: Into<BrickError> + 'static,
    {
        let signal = Signal::new(Load::Loading);
        let fetch = Rc::new(fetch);

        let signal_clone = signal.clone();
        let fetch_clone = Rc::clone(&fetch);
        let fetcher: Rc<dyn Fn()> = Rc::new(move || {
            let sig = signal_clone.clone();
            let fut = fetch_clone();
            #[cfg(brick_dom)]
            spawn_local(async move {
                match fut.await {
                    Ok(v) => sig.set(Load::Loaded(v)),
                    Err(e) => sig.set(Load::Failed(e.into())),
                }
            });
            #[cfg(not(brick_dom))]
            let _ = (sig, fut);
        });

        let resource = Resource { signal, fetcher };
        resource.refetch();
        resource
    }

    /// Create a lazy `Resource` that starts in `Load::Idle` and does NOT auto-fetch.
    ///
    /// Call `refetch()` explicitly to trigger the first load. Useful with `idle!` for
    /// search-on-demand or click-to-load patterns.
    pub fn lazy<F, Fut, E>(fetch: F) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: std::future::Future<Output = Result<T, E>> + 'static,
        E: Into<BrickError> + 'static,
    {
        let signal = Signal::new(Load::Idle);
        let fetch = Rc::new(fetch);

        let signal_clone = signal.clone();
        let fetch_clone = Rc::clone(&fetch);
        let fetcher: Rc<dyn Fn()> = Rc::new(move || {
            let sig = signal_clone.clone();
            let fut = fetch_clone();
            #[cfg(brick_dom)]
            spawn_local(async move {
                match fut.await {
                    Ok(v) => sig.set(Load::Loaded(v)),
                    Err(e) => sig.set(Load::Failed(e.into())),
                }
            });
            #[cfg(not(brick_dom))]
            let _ = (sig, fut);
        });

        // Do NOT call refetch() — stays Idle until the caller triggers it.
        Resource { signal, fetcher }
    }

    /// Return a clone of the underlying `Signal<Load<T, BrickError>>`.
    pub fn signal(&self) -> Signal<Load<T, BrickError>> {
        self.signal.clone()
    }

    /// Reset the signal to `Load::Loading` and re-run the fetcher.
    pub fn refetch(&self) {
        self.signal.set(Load::Loading);
        (self.fetcher)();
    }

    /// Snapshot the current `Load<T, BrickError>` state.
    pub fn read(&self) -> Load<T, BrickError> {
        self.signal.read()
    }

    /// Returns `true` when the resource is in the `Load::Idle` state (not yet fetched).
    pub fn is_idle(&self) -> bool {
        matches!(self.signal.read(), Load::Idle)
    }
}

impl<T: Clone + 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Resource {
            signal: self.signal.clone(),
            fetcher: Rc::clone(&self.fetcher),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_resource() -> Resource<String> {
        Resource::new(|| async { Ok::<String, BrickError>("hello".to_string()) })
    }

    #[test]
    fn new_starts_loading() {
        let r = make_resource();
        assert!(matches!(r.read(), Load::Loading));
    }

    #[test]
    fn read_delegates_to_signal() {
        let r = make_resource();
        // Both .read() and .signal().read() should agree
        assert_eq!(
            matches!(r.read(), Load::Loading),
            matches!(r.signal().read(), Load::Loading),
        );
        r.signal().set(Load::Loaded("world".to_string()));
        assert_eq!(
            matches!(r.read(), Load::Loaded(_)),
            matches!(r.signal().read(), Load::Loaded(_)),
        );
    }

    #[test]
    fn clone_shares_signal() {
        let r1 = make_resource();
        let r2 = r1.clone();
        // Simulate a resolved response via the shared signal
        r1.signal().set(Load::Loaded("data".to_string()));
        assert!(matches!(r2.read(), Load::Loaded(_)));
        if let Load::Loaded(v) = r2.read() {
            assert_eq!(v, "data");
        }
    }

    #[test]
    fn refetch_resets_to_loading() {
        let r = make_resource();
        // Manually simulate a resolved state
        r.signal().set(Load::Loaded("cached".to_string()));
        assert!(matches!(r.read(), Load::Loaded(_)));
        // refetch should reset to Loading (spawn_local is a no-op in tests)
        r.refetch();
        assert!(matches!(r.read(), Load::Loading));
    }

    #[test]
    fn lazy_starts_idle() {
        let r: Resource<String> =
            Resource::lazy(|| async { Ok::<_, BrickError>("lazy".to_string()) });
        assert!(
            matches!(r.read(), Load::Idle),
            "lazy resource should start as Load::Idle"
        );
    }

    #[test]
    fn lazy_does_not_auto_fetch() {
        // In test mode spawn_local is a no-op anyway, but the signal should stay
        // Idle because lazy() never calls refetch().
        let r: Resource<String> =
            Resource::lazy(|| async { Ok::<_, BrickError>("data".to_string()) });
        // Signal remains Idle — no automatic fetch was triggered.
        assert!(matches!(r.read(), Load::Idle));
    }

    #[test]
    fn string_error_converts_via_into() {
        let r: Resource<String> =
            Resource::new(|| async { Err::<String, String>("oops".to_string()) });
        // In test mode the future doesn't run, so signal stays Loading.
        // This test mainly verifies the type signature compiles with E=String.
        assert!(matches!(r.read(), Load::Loading));
    }
}
