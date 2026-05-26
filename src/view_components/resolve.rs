use super::Brick;
#[cfg(brick_dom)]
use crate::state_mgmt::Effect;
use crate::state_mgmt::{Load, Signal};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static RESOLVE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A view component that branches on a `Signal<Load<T, E>>`, re-rendering
/// the appropriate child whenever the state changes. Used internally by `load!`.
pub struct ViewResolve<T: Clone + 'static, E: Clone + 'static = crate::ssr::BrickError> {
    name: String,
    source: Signal<Load<T, E>>,
    idle: Rc<dyn Fn() -> Box<dyn Brick>>,
    loading: Rc<dyn Fn() -> Box<dyn Brick>>,
    loaded: Rc<dyn Fn(&T) -> Box<dyn Brick>>,
    failed: Rc<dyn Fn(&E) -> Box<dyn Brick>>,
}

impl<T: Clone + 'static, E: Clone + 'static> ViewResolve<T, E> {
    pub fn new(
        source: Signal<Load<T, E>>,
        idle: Rc<dyn Fn() -> Box<dyn Brick>>,
        loading: Rc<dyn Fn() -> Box<dyn Brick>>,
        loaded: Rc<dyn Fn(&T) -> Box<dyn Brick>>,
        failed: Rc<dyn Fn(&E) -> Box<dyn Brick>>,
    ) -> Box<Self> {
        Box::new(ViewResolve {
            name: format!(
                "brick-resolve-{}",
                RESOLVE_COUNTER.fetch_add(1, Ordering::SeqCst)
            ),
            source,
            idle,
            loading,
            loaded,
            failed,
        })
    }

    fn branch_html(&self, state: &Load<T, E>) -> String {
        let b: Box<dyn Brick> = match state {
            Load::Idle => (self.idle)(),
            Load::Loading => (self.loading)(),
            Load::Loaded(v) => (self.loaded)(v),
            Load::Failed(e) => (self.failed)(e),
        };
        crate::view_components::render_to_html(&*b)
    }
}

impl<T: Clone + 'static, E: Clone + 'static> Brick for ViewResolve<T, E> {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, HtmlNode, Renderer};
        let wrapper = Renderer::element("div");
        Renderer::set_attr(&wrapper, "data-brick-resolve", &self.name);
        let inner = self.branch_html(&self.source.read());
        if !inner.is_empty() {
            Renderer::append(&wrapper, &HtmlNode::raw(inner));
        }
        Renderer::append(parent, &wrapper);
    }

    fn attach_listeners(&self) {
        #[cfg(brick_dom)]
        {
            let name = self.name.clone();
            let idle = Rc::clone(&self.idle);
            let loading = Rc::clone(&self.loading);
            let loaded = Rc::clone(&self.loaded);
            let failed = Rc::clone(&self.failed);

            self.source.add_observer(
                &self.name,
                Box::new(Effect::new(move |state: &Load<T, E>| {
                    let html = match state {
                        Load::Idle => crate::view_components::render_to_html(&*(idle)()),
                        Load::Loading => crate::view_components::render_to_html(&*(loading)()),
                        Load::Loaded(v) => crate::view_components::render_to_html(&*(loaded)(v)),
                        Load::Failed(e) => crate::view_components::render_to_html(&*(failed)(e)),
                    };
                    let doc = web_sys::window().unwrap().document().unwrap();
                    if let Some(el) = doc
                        .query_selector(&format!("[data-brick-resolve=\"{}\"]", name))
                        .ok()
                        .flatten()
                    {
                        el.set_inner_html(&html);
                    }
                })),
            );
        }
    }

    fn detach(&self) {
        self.source.remove_observer(&self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::leafs::p;
    use std::rc::Rc;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    fn make_resolve(signal: Signal<Load<String, String>>) -> Box<ViewResolve<String, String>> {
        ViewResolve::new(
            signal,
            Rc::new(|| -> Box<dyn Brick> { p("idle") }),
            Rc::new(|| -> Box<dyn Brick> { p("loading") }),
            Rc::new(|v: &String| -> Box<dyn Brick> { p(v.as_str()) }),
            Rc::new(|e: &String| -> Box<dyn Brick> { p(e.as_str()) }),
        )
    }

    #[test]
    fn renders_idle_state() {
        let r = make_resolve(Signal::new(Load::Idle));
        assert!(rh(&*r).contains("<p>idle</p>"));
    }

    #[test]
    fn renders_loading_state() {
        let r = make_resolve(Signal::new(Load::Loading));
        assert!(rh(&*r).contains("<p>loading</p>"));
    }

    #[test]
    fn renders_loaded_state() {
        let r = make_resolve(Signal::new(Load::Loaded("done".to_string())));
        assert!(rh(&*r).contains("<p>done</p>"));
    }

    #[test]
    fn renders_failed_state() {
        let r = make_resolve(Signal::new(Load::Failed("oops".to_string())));
        assert!(rh(&*r).contains("<p>oops</p>"));
    }

    #[test]
    fn html_wraps_in_data_brick_resolve() {
        let r = make_resolve(Signal::new(Load::Idle));
        assert!(rh(&*r).contains("data-brick-resolve="));
    }

    #[test]
    fn two_resolves_get_unique_wrappers() {
        let a = make_resolve(Signal::new(Load::Idle));
        let b = make_resolve(Signal::new(Load::Idle));
        assert_ne!(
            rh(&*a),
            rh(&*b),
            "each resolve gets a unique identity attribute"
        );
    }

    #[test]
    fn detach_removes_observer() {
        let sig: Signal<Load<String, String>> = Signal::new(Load::Idle);
        let r = make_resolve(sig.clone());
        <ViewResolve<String, String> as Brick>::detach(&*r);
        // After detach, setting signal should not panic (no observer referencing dropped data)
        sig.set(Load::Loading);
    }
}
