use std::sync::atomic::{AtomicBool, Ordering};
use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};

use super::Brick;
use crate::renderer::BrickEvent;
use crate::state_mgmt::BrickLifecycle;

// ── Render-scope event listener registry ─────────────────────────────────────
//
// Populated by ViewComposite::render_into before rendering children; cleared
// after. Element builders call lookup_scope_listener during their own
// render_into to wire Renderer::on_event directly, replacing the old
// document-level CSS-query delegation approach.

thread_local! {
    static RENDER_SCOPE_LISTENERS: RefCell<HashMap<&'static str, (&'static str, Rc<dyn Fn(BrickEvent)>)>>
        = RefCell::new(HashMap::new());
}

/// Push `listeners` as the active render scope, call `f`, then restore the
/// previous scope. Handles nesting correctly so inner components don't inherit
/// outer component listeners.
pub(crate) fn with_scope_listeners<F: FnOnce()>(
    listeners: &HashMap<&'static str, (&'static str, Rc<dyn Fn(BrickEvent)>)>,
    f: F,
) {
    let prev = RENDER_SCOPE_LISTENERS.with(|sl| {
        std::mem::replace(&mut *sl.borrow_mut(), listeners.clone())
    });
    f();
    RENDER_SCOPE_LISTENERS.with(|sl| {
        *sl.borrow_mut() = prev;
    });
}

/// Look up a listener for `action` in the current render scope.
pub(crate) fn lookup_scope_listener(
    action: &str,
) -> Option<(&'static str, Rc<dyn Fn(BrickEvent)>)> {
    RENDER_SCOPE_LISTENERS.with(|sl| {
        sl.borrow().get(action).map(|(et, cb)| (*et, Rc::clone(cb)))
    })
}

#[cfg(brick_dom)]
static NAV_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Register a single document-level click listener that handles all NavLink
/// navigation in the app. Safe to call repeatedly — only registers once.
#[cfg(brick_dom)]
fn ensure_nav_delegation() {
    if NAV_REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }
    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        let target = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok());
        if let Some(el) = target {
            if let Some(nav_el) = el.closest("[data-brick-navigate]").ok().flatten() {
                if let Some(path) = nav_el.get_attribute("data-brick-navigate") {
                    e.prevent_default();
                    crate::routing::navigate_to_path(&path);
                }
            }
        }
    }) as Box<dyn FnMut(web_sys::Event)>);
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        doc.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .ok();
    }
    closure.forget();
}

pub struct ViewComposite {
    pub tag: &'static str,
    pub class_name: String,
    #[allow(dead_code)]
    pub model: Rc<RefCell<dyn Any>>,
    pub children: Vec<Box<dyn Brick>>,
    /// True when the single child is a BrickFragment. Causes render_into to
    /// propagate the instance class directly onto the fragment's top-level
    /// children rather than wrapping them in a div.
    pub is_fragment_root: bool,
    pub event_listeners: HashMap<&'static str, (&'static str, Rc<dyn Fn(BrickEvent)>)>,
    pub lifecycle: BrickLifecycle,
    pub intervals: Vec<(i32, Rc<dyn Fn()>)>,
    pub interval_handles: Rc<RefCell<Vec<i32>>>,
    pub raf_callbacks: Vec<Rc<dyn Fn()>>,
    pub raf_handles: Rc<RefCell<Vec<i32>>>,
}
impl ViewComposite {
    pub fn mount(&self, anchor: &web_sys::HtmlElement) {
        anchor.set_inner_html(&crate::view_components::render_to_html(self));
        self.attach_listeners();
    }

    #[allow(dead_code)]
    pub fn c<T: std::fmt::Display + ?Sized>(mut self: Box<Self>, name: &T) -> Box<Self> {
        self.class_name = name.to_string();
        self
    }
}
impl Brick for ViewComposite {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        with_scope_listeners(&self.event_listeners, || {
            if self.is_fragment_root {
                let before = Renderer::child_count(parent);
                for child in &self.children {
                    child.render_into(parent);
                }
                let after = Renderer::child_count(parent);
                for i in before..after {
                    if let Some(node) = Renderer::nth_child(parent, i) {
                        Renderer::add_class(&node, &self.class_name);
                    }
                }
            } else {
                let root = Renderer::element(self.tag);
                Renderer::set_attr(&root, "class", &self.class_name);
                for child in &self.children {
                    child.render_into(&root);
                }
                Renderer::append(parent, &root);
            }
        });
        for cb in &self.lifecycle.on_mount {
            cb();
        }
    }

    fn attach_listeners(&self) {
        for cb in &self.lifecycle.on_mount {
            cb();
        }

        #[cfg(brick_dom)]
        ensure_nav_delegation();

        #[cfg(brick_dom)]
        for (ms, cb) in &self.intervals {
            let cb = Rc::clone(cb);
            let closure = Closure::wrap(Box::new(move || cb()) as Box<dyn FnMut()>);
            let handle = web_sys::window()
                .unwrap()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    *ms,
                )
                .unwrap();
            self.interval_handles.borrow_mut().push(handle);
            closure.forget();
        }

        #[cfg(brick_dom)]
        for cb in &self.raf_callbacks {
            let cb = Rc::clone(cb);
            let raf_handles = Rc::clone(&self.raf_handles);
            let holder: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
            let holder_inner = Rc::clone(&holder);
            *holder.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                cb();
                let window = web_sys::window().unwrap();
                let borrow = holder_inner.borrow();
                let handle = window
                    .request_animation_frame(borrow.as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap();
                drop(borrow);
                let mut h = raf_handles.borrow_mut();
                if let Some(last) = h.last_mut() {
                    *last = handle;
                }
            }) as Box<dyn FnMut()>));
            let window = web_sys::window().unwrap();
            let first_handle = {
                let borrow = holder.borrow();
                window
                    .request_animation_frame(borrow.as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap()
            };
            self.raf_handles.borrow_mut().push(first_handle);
            std::mem::forget(holder);
        }
        // Event wiring now happens inline in render_into via Renderer::on_event.
        // No document-level delegation needed.
    }

    fn detach(&self) {
        for cb in &self.lifecycle.on_before_unmount {
            cb();
        }
        for child in &self.children {
            child.detach();
        }
        #[cfg(brick_dom)]
        for handle in self.interval_handles.borrow().iter() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(*handle);
            }
        }
        #[cfg(brick_dom)]
        for handle in self.raf_handles.borrow().iter() {
            if let Some(window) = web_sys::window() {
                window.cancel_animation_frame(*handle).ok();
            }
        }
        for cb in &self.lifecycle.on_unmount {
            cb();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ViewComposite;
    use crate::state_mgmt::BrickLifecycle;
    use crate::view_components::{
        Brick,
        leafs::{button, h1, p},
    };
    use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    fn dummy_model() -> Rc<RefCell<dyn Any>> {
        Rc::new(RefCell::new(()))
    }

    fn empty_composite(class_name: &str) -> ViewComposite {
        ViewComposite {
            tag: "div",
            class_name: class_name.to_string(),
            model: dummy_model(),
            children: vec![],
            is_fragment_root: false,
            event_listeners: HashMap::new(),
            lifecycle: BrickLifecycle::default(),
            intervals: vec![],
            interval_handles: Rc::new(RefCell::new(vec![])),
            raf_callbacks: vec![],
            raf_handles: Rc::new(RefCell::new(vec![])),
        }
    }

    fn make_flag() -> (Rc<RefCell<bool>>, Rc<dyn Fn()>) {
        let flag = Rc::new(RefCell::new(false));
        let flag2 = Rc::clone(&flag);
        (flag, Rc::new(move || *flag2.borrow_mut() = true))
    }

    #[test]
    fn empty_composite_html() {
        let c = empty_composite("MyComponent");
        assert_eq!(rh(&c), r#"<div class="MyComponent"></div>"#);
    }

    #[test]
    fn composite_with_leaf_children() {
        let c = ViewComposite {
            tag: "div",
            class_name: "Counter".to_string(),
            model: dummy_model(),
            children: vec![h1("Title"), p("Body")],
            is_fragment_root: false,
            event_listeners: HashMap::new(),
            lifecycle: BrickLifecycle::default(),
            intervals: vec![],
            interval_handles: Rc::new(RefCell::new(vec![])),
            raf_callbacks: vec![],
            raf_handles: Rc::new(RefCell::new(vec![])),
        };
        assert_eq!(
            rh(&c),
            r#"<div class="Counter"><h1>Title</h1><p>Body</p></div>"#
        );
    }

    #[test]
    fn composite_with_classed_children() {
        let c = ViewComposite {
            tag: "div",
            class_name: "Counter".to_string(),
            model: dummy_model(),
            children: vec![button("+").c("increment")],
            is_fragment_root: false,
            event_listeners: HashMap::new(),
            lifecycle: BrickLifecycle::default(),
            intervals: vec![],
            interval_handles: Rc::new(RefCell::new(vec![])),
            raf_callbacks: vec![],
            raf_handles: Rc::new(RefCell::new(vec![])),
        };
        assert_eq!(
            rh(&c),
            r#"<div class="Counter"><button class="increment">+</button></div>"#
        );
    }

    #[test]
    fn nested_composites_html() {
        let inner = Box::new(empty_composite("Inner"));
        let outer = ViewComposite {
            tag: "div",
            class_name: "Outer".to_string(),
            model: dummy_model(),
            children: vec![inner],
            is_fragment_root: false,
            event_listeners: HashMap::new(),
            lifecycle: BrickLifecycle::default(),
            intervals: vec![],
            interval_handles: Rc::new(RefCell::new(vec![])),
            raf_callbacks: vec![],
            raf_handles: Rc::new(RefCell::new(vec![])),
        };
        assert_eq!(
            rh(&outer),
            r#"<div class="Outer"><div class="Inner"></div></div>"#
        );
    }

    #[test]
    fn c_builder_changes_class_name() {
        let c = Box::new(empty_composite("Original")).c("Renamed");
        assert_eq!(rh(&*c), r#"<div class="Renamed"></div>"#);
    }

    #[test]
    fn detach_fires_before_unmount_then_unmount_in_order() {
        let order = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let o1 = Rc::clone(&order);
        let o2 = Rc::clone(&order);
        let mut c = empty_composite("Comp");
        c.lifecycle.on_before_unmount = vec![Rc::new(move || o1.borrow_mut().push("before"))];
        c.lifecycle.on_unmount = vec![Rc::new(move || o2.borrow_mut().push("unmount"))];
        <ViewComposite as Brick>::detach(&c);
        assert_eq!(*order.borrow(), vec!["before", "unmount"]);
    }

    #[test]
    fn detach_recurses_to_children() {
        let (flag, cb) = make_flag();
        let mut inner = empty_composite("Inner");
        inner.lifecycle.on_unmount = vec![cb];
        let mut outer = empty_composite("Outer");
        outer.children = vec![Box::new(inner)];
        <ViewComposite as Brick>::detach(&outer);
        assert!(
            *flag.borrow(),
            "child on_unmount should fire when parent is detached"
        );
    }

    #[test]
    fn before_unmount_fires_before_children_detach() {
        let order = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let o1 = Rc::clone(&order);
        let o2 = Rc::clone(&order);

        let mut inner = empty_composite("Inner");
        inner.lifecycle.on_unmount = vec![Rc::new(move || o2.borrow_mut().push("child"))];

        let mut outer = empty_composite("Outer");
        outer.lifecycle.on_before_unmount = vec![Rc::new(move || o1.borrow_mut().push("before"))];
        outer.children = vec![Box::new(inner)];
        <ViewComposite as Brick>::detach(&outer);

        assert_eq!(*order.borrow(), vec!["before", "child"]);
    }

    #[test]
    fn on_mount_fires_during_render() {
        use std::cell::Cell;
        let flag = Rc::new(Cell::new(false));
        let flag2 = Rc::clone(&flag);
        let mut c = empty_composite("M");
        c.lifecycle.on_mount = vec![Rc::new(move || flag2.set(true))];
        let _ = rh(&c);
        assert!(flag.get(), "on_mount should fire during render_into");
    }

    #[test]
    fn fragment_root_propagates_class_to_children() {
        use crate::view_components::BrickFragment;
        let mut c = empty_composite("Def");
        c.children = vec![BrickFragment::boxed(vec![p("a"), p("b")])];
        c.is_fragment_root = true;
        let html = rh(&c);
        assert!(
            !html.contains("display:contents"),
            "no display:contents wrapper"
        );
        assert!(
            html.contains(r#"class="Def""#),
            "instance class propagated to children"
        );
        assert!(html.contains("<p") && html.contains(">a</p>"));
        assert!(html.contains(">b</p>"));
    }

    #[test]
    fn non_fragment_root_has_no_display_contents() {
        let mut c = empty_composite("Counter");
        c.children = vec![p("x")];
        let html = rh(&c);
        assert!(!html.contains("display:contents"));
    }

    #[test]
    fn make_composite_detects_fragment_root() {
        use crate::state_mgmt::BrickLifecycle;
        use crate::view_components::{BrickFragment, IntoComponent};
        use std::collections::HashMap;
        // Simulate what #[view] generates for a fragment view
        let children: Vec<Box<dyn Brick>> = vec![BrickFragment::boxed(vec![p("dt"), p("dd")])];
        let is_fragment_root = children.len() == 1 && children[0].is_fragment();
        assert!(
            is_fragment_root,
            "single BrickFragment child should be detected as fragment root"
        );
    }
}
