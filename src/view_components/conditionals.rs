#![allow(dead_code)]
#[cfg(brick_dom)]
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::Brick;
#[cfg(brick_dom)]
use crate::state_mgmt::Effect;
use crate::state_mgmt::Signal;

static COND_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub enum ConditionalMode {
    Hide,
    Mount,
}

pub struct ViewConditional {
    name: String,
    source: Signal<bool>,
    true_branch: Rc<dyn Brick>,
    false_branch: Option<Rc<dyn Brick>>,
    mode: ConditionalMode,
    enter_class: Option<String>,
    exit_class: Option<String>,
    pub(crate) use_view_transition: bool,
}

impl ViewConditional {
    pub fn new(
        source: Signal<bool>,
        true_branch: Box<dyn Brick>,
        false_branch: Option<Box<dyn Brick>>,
    ) -> Self {
        ViewConditional {
            name: format!("cond_{}", COND_COUNTER.fetch_add(1, Ordering::SeqCst)),
            source,
            true_branch: Rc::from(true_branch),
            false_branch: false_branch.map(|b| Rc::from(b) as Rc<dyn Brick>),
            mode: ConditionalMode::Hide,
            enter_class: None,
            exit_class: None,
            use_view_transition: false,
        }
    }

    pub fn hide(mut self: Box<Self>) -> Box<Self> {
        self.mode = ConditionalMode::Hide;
        self
    }

    pub fn mount(mut self: Box<Self>) -> Box<Self> {
        self.mode = ConditionalMode::Mount;
        self
    }

    pub fn enter(mut self: Box<Self>, class: &str) -> Box<Self> {
        self.enter_class = Some(class.to_string());
        self
    }

    pub fn exit(mut self: Box<Self>, class: &str) -> Box<Self> {
        self.exit_class = Some(class.to_string());
        self
    }

    /// Enable the View Transitions API for this conditional swap. Implies mount mode.
    pub fn transition(mut self: Box<Self>) -> Box<Self> {
        self.mode = ConditionalMode::Mount;
        self.use_view_transition = true;
        self
    }

    fn branch_html(&self, is_true: bool) -> String {
        if is_true {
            crate::view_components::render_to_html(&*self.true_branch)
        } else {
            self.false_branch
                .as_ref()
                .map(|b| crate::view_components::render_to_html(&**b))
                .unwrap_or_default()
        }
    }
}

/// Adds `exit_cls` to `el`, then waits for `animationend` or `transitionend`
/// (whichever fires first, with `{ once: true }`), then calls `done`.
#[cfg(brick_dom)]
fn animate_exit(el: &web_sys::Element, exit_cls: &str, done: impl FnOnce() + 'static) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;

    el.class_list().add_1(exit_cls).ok();

    let fired = Rc::new(std::cell::Cell::new(false));
    let done = Rc::new(RefCell::new(Some(Box::new(done) as Box<dyn FnOnce()>)));

    let opts = web_sys::AddEventListenerOptions::new();
    opts.set_once(true);

    for event in &["animationend", "transitionend"] {
        let fired = Rc::clone(&fired);
        let done = Rc::clone(&done);
        let handler = Closure::wrap(Box::new(move || {
            if !fired.replace(true) {
                if let Some(f) = done.borrow_mut().take() {
                    f();
                }
            }
        }) as Box<dyn FnMut()>);
        el.add_event_listener_with_callback_and_add_event_listener_options(
            event,
            handler.as_ref().unchecked_ref(),
            &opts,
        )
        .ok();
        handler.forget();
    }
}

#[cfg(brick_dom)]
fn find_cond(name: &str) -> web_sys::Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector(&format!("[data-brick-cond=\"{}\"]", name))
        .unwrap()
        .unwrap()
}

// ── Brick implementation ──────────────────────────────────────────────

impl Brick for ViewConditional {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let is_true = self.source.read();

        match self.mode {
            ConditionalMode::Hide => {
                if self.false_branch.is_none() {
                    let wrapper = Renderer::element("div");
                    Renderer::set_attr(&wrapper, "data-brick-cond", &self.name);
                    if !is_true {
                        Renderer::set_attr(&wrapper, "hidden", "");
                    }
                    self.true_branch.render_into(&wrapper);
                    let node_clone = wrapper.clone();
                    let name = self.name.clone();
                    use crate::state_mgmt::observers::Effect;
                    self.source.add_observer(
                        &name,
                        Box::new(Effect::new(move |is_true: &bool| {
                            if *is_true {
                                Renderer::remove_attr(&node_clone, "hidden");
                            } else {
                                Renderer::set_attr(&node_clone, "hidden", "");
                            }
                        })),
                    );
                    Renderer::append(parent, &wrapper);
                } else {
                    // Two-branch hide: append two sibling wrappers to parent.
                    let t_div = Renderer::element("div");
                    Renderer::set_attr(&t_div, "data-brick-cond", &format!("{}-t", self.name));
                    if !is_true {
                        Renderer::set_attr(&t_div, "hidden", "");
                    }
                    self.true_branch.render_into(&t_div);
                    Renderer::append(parent, &t_div);

                    let f_div = Renderer::element("div");
                    Renderer::set_attr(&f_div, "data-brick-cond", &format!("{}-f", self.name));
                    if is_true {
                        Renderer::set_attr(&f_div, "hidden", "");
                    }
                    if let Some(fb) = &self.false_branch {
                        fb.render_into(&f_div);
                    }
                    Renderer::append(parent, &f_div);
                }
            }
            ConditionalMode::Mount => {
                let wrapper = Renderer::element("div");
                Renderer::set_attr(&wrapper, "data-brick-cond", &self.name);
                if is_true {
                    self.true_branch.render_into(&wrapper);
                } else if let Some(fb) = &self.false_branch {
                    fb.render_into(&wrapper);
                }
                Renderer::append(parent, &wrapper);
            }
        }
    }

    fn attach_listeners(&self) {
        #[cfg(brick_dom)]
        {
            use wasm_bindgen::JsCast;
            let name = self.name.clone();
            match self.mode {
                ConditionalMode::Hide => {
                    if self.false_branch.is_none() {
                        let name_c = name.clone();
                        self.source.add_observer(
                            &name,
                            Box::new(Effect::new(move |is_true: &bool| {
                                let el = find_cond(&name_c);
                                if *is_true {
                                    el.remove_attribute("hidden").unwrap();
                                } else {
                                    el.set_attribute("hidden", "").unwrap();
                                }
                            })),
                        );
                    } else {
                        let name_t = format!("{}-t", name);
                        let name_f = format!("{}-f", name);
                        self.source.add_observer(
                            &name,
                            Box::new(Effect::new(move |is_true: &bool| {
                                let el_t = find_cond(&name_t);
                                let el_f = find_cond(&name_f);
                                if *is_true {
                                    el_t.remove_attribute("hidden").unwrap();
                                    el_f.set_attribute("hidden", "").unwrap();
                                } else {
                                    el_t.set_attribute("hidden", "").unwrap();
                                    el_f.remove_attribute("hidden").unwrap();
                                }
                            })),
                        );
                    }
                }
                ConditionalMode::Mount => {
                    let true_branch = Rc::clone(&self.true_branch);
                    let false_branch = self.false_branch.as_ref().map(Rc::clone);
                    let is_true_shown = Rc::new(RefCell::new(self.source.read()));
                    let name_c = name.clone();
                    let is_true_shown_c = Rc::clone(&is_true_shown);
                    let enter_class = self.enter_class.clone();
                    let exit_class = self.exit_class.clone();
                    let use_view_transition = self.use_view_transition;
                    self.source.add_observer(
                        &name,
                        Box::new(Effect::new(move |is_true: &bool| {
                            let was_true = *is_true_shown_c.borrow();
                            *is_true_shown_c.borrow_mut() = *is_true;

                            let true_branch = Rc::clone(&true_branch);
                            let false_branch = false_branch.as_ref().map(Rc::clone);
                            let name_c = name_c.clone();
                            let enter_class = enter_class.clone();
                            let is_true = *is_true;

                            let name_for_exit = name_c.clone();
                            let do_swap = move || {
                                if was_true {
                                    true_branch.detach();
                                } else if let Some(fb) = &false_branch {
                                    fb.detach();
                                }
                                let new_html = if is_true {
                                    crate::view_components::render_to_html(&*true_branch)
                                } else {
                                    false_branch
                                        .as_ref()
                                        .map(|b| crate::view_components::render_to_html(&**b))
                                        .unwrap_or_default()
                                };
                                let el = find_cond(&name_c);
                                el.set_inner_html(&new_html);
                                if let Some(cls) = &enter_class {
                                    el.class_list().add_1(cls).ok();
                                }
                            };

                            if use_view_transition {
                                crate::view_components::start_view_transition(do_swap);
                            } else if let Some(ref cls) = exit_class {
                                let el = find_cond(&name_for_exit);
                                animate_exit(&el, cls, do_swap);
                            } else {
                                do_swap();
                            }
                        })),
                    );
                }
            }
        }
    }

    fn detach(&self) {
        self.source.remove_observer(&self.name);
        self.true_branch.detach();
        if let Some(fb) = &self.false_branch {
            fb.detach();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        state_mgmt::Signal,
        view_components::leafs::{button, p},
    };

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn single_branch_shows_when_true() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("hi"), None));
        let html = rh(&*c);
        assert!(html.contains("data-brick-cond="), "should have cond attr");
        assert!(!html.contains("hidden"), "should not be hidden when true");
        assert!(html.contains("<p>hi</p>"));
    }

    #[test]
    fn single_branch_hidden_when_false() {
        let c = Box::new(ViewConditional::new(Signal::new(false), p("hi"), None));
        let html = rh(&*c);
        assert!(html.contains(" hidden"), "wrapper should carry hidden attr");
        assert!(html.contains("<p>hi</p>"), "content still rendered in DOM");
    }

    #[test]
    fn two_branch_true_shows_true_hides_false() {
        let c = Box::new(ViewConditional::new(
            Signal::new(true),
            p("yes"),
            Some(p("no")),
        ));
        let html = rh(&*c);
        assert!(html.contains(r#"<p>yes</p>"#));
        assert!(html.contains(r#"<p>no</p>"#));
        // true wrapper has no hidden, false wrapper does
        let t_idx = html.find("-t\"").unwrap();
        let f_idx = html.find("-f\"").unwrap();
        assert!(t_idx < f_idx, "true wrapper comes first");
        let before_f = &html[f_idx..];
        assert!(
            before_f.contains("hidden"),
            "false wrapper is hidden when source is true"
        );
        let between = &html[t_idx..f_idx];
        assert!(!between.contains("hidden"), "true wrapper not hidden");
    }

    #[test]
    fn two_branch_false_shows_false_hides_true() {
        let c = Box::new(ViewConditional::new(
            Signal::new(false),
            p("yes"),
            Some(p("no")),
        ));
        let html = rh(&*c);
        let t_idx = html.find("-t\"").unwrap();
        let f_idx = html.find("-f\"").unwrap();
        let between = &html[t_idx..f_idx];
        assert!(
            between.contains("hidden"),
            "true wrapper hidden when source is false"
        );
        let after_f = &html[f_idx..];
        assert!(
            !after_f.starts_with("-f\" hidden"),
            "false wrapper not hidden"
        );
    }

    #[test]
    fn mount_shows_true_branch_initially() {
        let c = Box::new(ViewConditional::new(
            Signal::new(true),
            p("on"),
            Some(p("off")),
        ))
        .mount();
        let html = rh(&*c);
        assert!(html.contains("<p>on</p>"), "true branch content visible");
        assert!(!html.contains("<p>off</p>"), "false branch not in DOM");
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn mount_shows_false_branch_initially() {
        let c = Box::new(ViewConditional::new(
            Signal::new(false),
            p("on"),
            Some(p("off")),
        ))
        .mount();
        let html = rh(&*c);
        assert!(!html.contains("<p>on</p>"));
        assert!(html.contains("<p>off</p>"));
    }

    #[test]
    fn mount_single_branch_empty_when_false() {
        let c = Box::new(ViewConditional::new(Signal::new(false), p("on"), None)).mount();
        let html = rh(&*c);
        assert!(
            !html.contains("<p>"),
            "no content when false and no false branch"
        );
        assert!(html.contains("data-brick-cond="));
    }

    #[test]
    fn hide_is_default_mode() {
        let c = Box::new(ViewConditional::new(Signal::new(false), button("x"), None));
        let html = rh(&*c);
        assert!(html.contains(" hidden"));
    }

    #[test]
    fn builder_chaining_works_on_box() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("hi"), None))
            .hide()
            .mount()
            .hide();
        let html = rh(&*c);
        assert!(!html.contains(" hidden"));
    }

    #[test]
    fn detach_smoke_test_does_not_panic() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("yes"), None));
        <ViewConditional as Brick>::detach(&*c); // must not panic
    }

    #[test]
    fn detach_two_branch_conditional_does_not_panic() {
        let c = Box::new(ViewConditional::new(
            Signal::new(true),
            p("yes"),
            Some(p("no")),
        ));
        <ViewConditional as Brick>::detach(&*c); // detach removes observer and recurses both branches
    }

    #[test]
    fn enter_class_stored_on_builder() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("hi"), None))
            .mount()
            .enter("fade-in");
        assert_eq!(c.enter_class.as_deref(), Some("fade-in"));
    }

    #[test]
    fn exit_class_stored_on_builder() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("hi"), None))
            .mount()
            .exit("fade-out");
        assert_eq!(c.exit_class.as_deref(), Some("fade-out"));
    }

    #[test]
    fn enter_exit_chain_together() {
        let c = Box::new(ViewConditional::new(
            Signal::new(false),
            p("on"),
            Some(p("off")),
        ))
        .mount()
        .enter("slide-in")
        .exit("slide-out");
        assert_eq!(c.enter_class.as_deref(), Some("slide-in"));
        assert_eq!(c.exit_class.as_deref(), Some("slide-out"));
        assert!(rh(&*c).contains("<p>off</p>"));
    }

    #[test]
    fn default_no_animation_classes() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("hi"), None));
        assert!(c.enter_class.is_none());
        assert!(c.exit_class.is_none());
    }

    #[test]
    fn transition_implies_mount_mode() {
        let c = Box::new(ViewConditional::new(
            Signal::new(true),
            p("on"),
            Some(p("off")),
        ))
        .transition();
        let html = rh(&*c);
        assert!(html.contains("<p>on</p>"), "active branch rendered");
        assert!(!html.contains("<p>off</p>"), "inactive branch not in DOM");
        assert!(!html.contains("hidden"), "mount mode — no hidden attr");
    }

    #[test]
    fn transition_sets_flag() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("hi"), None)).transition();
        assert!(c.use_view_transition);
    }

    #[test]
    fn transition_false_initial_shows_false_branch() {
        let c = Box::new(ViewConditional::new(
            Signal::new(false),
            p("on"),
            Some(p("off")),
        ))
        .transition();
        let html = rh(&*c);
        assert!(html.contains("<p>off</p>"));
        assert!(!html.contains("<p>on</p>"));
    }

    // ── Parity tests ──────────────────────────────────────────────────────────

    #[test]
    fn parity_single_branch_true() {
        let c = Box::new(ViewConditional::new(Signal::new(true), p("hi"), None));
        let html = rh(&*c);
        assert!(html.contains("data-brick-cond="));
        assert!(!html.contains("hidden"));
        assert!(html.contains("<p>hi</p>"));
    }

    #[test]
    fn parity_single_branch_false() {
        let c = Box::new(ViewConditional::new(Signal::new(false), p("hi"), None));
        let html = rh(&*c);
        assert!(html.contains("data-brick-cond="));
        assert!(html.contains(" hidden"));
        assert!(html.contains("<p>hi</p>"));
    }

    #[test]
    fn parity_two_branch_hide_true() {
        let c = Box::new(ViewConditional::new(
            Signal::new(true),
            p("yes"),
            Some(p("no")),
        ));
        let html = rh(&*c);
        assert!(html.contains("<p>yes</p>"));
        assert!(html.contains("<p>no</p>"));
    }

    #[test]
    fn parity_two_branch_hide_false() {
        let c = Box::new(ViewConditional::new(
            Signal::new(false),
            p("yes"),
            Some(p("no")),
        ));
        let html = rh(&*c);
        assert!(html.contains("<p>yes</p>"));
        assert!(html.contains("<p>no</p>"));
    }

    #[test]
    fn parity_mount_mode_true() {
        let c = Box::new(ViewConditional::new(
            Signal::new(true),
            p("on"),
            Some(p("off")),
        ))
        .mount();
        let html = rh(&*c);
        assert!(html.contains("<p>on</p>"));
        assert!(!html.contains("<p>off</p>"));
    }

    #[test]
    fn parity_mount_mode_false() {
        let c = Box::new(ViewConditional::new(
            Signal::new(false),
            p("on"),
            Some(p("off")),
        ))
        .mount();
        let html = rh(&*c);
        assert!(!html.contains("<p>on</p>"));
        assert!(html.contains("<p>off</p>"));
    }

    #[test]
    fn reactive_hidden_attr_updates_on_signal_change() {
        use crate::renderer::{BrickRenderer as _, HtmlRenderer};
        let sig = Signal::new(true);
        let c = Box::new(ViewConditional::new(sig.clone(), p("hi"), None));
        let parent = HtmlRenderer::element("div");
        c.render_into(&parent);
        assert!(
            !parent.inner_html_string().contains(" hidden"),
            "should be visible when true"
        );
        sig.set(false);
        assert!(
            parent.inner_html_string().contains(" hidden"),
            "hidden attr added on false"
        );
        sig.set(true);
        assert!(
            !parent.inner_html_string().contains(" hidden"),
            "hidden removed on true again"
        );
    }
}
