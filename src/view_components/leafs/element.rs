#![allow(dead_code)]
use crate::state_mgmt::BrickAction;
use crate::theme::Space;
use std::fmt::Display;
use std::rc::Rc;

pub struct ViewLeafText {
    pub(crate) tag: &'static str,
    pub(crate) class_name: String,
    pub(crate) text_content: String,
    pub(crate) action_name: Option<String>,
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) is_void: bool,
    pub(crate) bind_class: Option<String>,
    pub(crate) bind_initial: Option<String>,
    pub(crate) bind_handler: Option<Rc<dyn Fn(crate::renderer::BrickEvent)>>,
    pub(crate) click_class: Option<String>,
    pub(crate) click_handler: Option<Rc<dyn Fn(crate::renderer::BrickEvent)>>,
    pub(crate) toggle_class: Option<String>,
    pub(crate) toggle_handler: Option<Rc<dyn Fn(crate::renderer::BrickEvent)>>,
    pub(crate) brick_classes: Vec<&'static str>,
    pub(crate) inline_style: Option<String>,
    pub(crate) style_signal_class: Option<String>,
    pub(crate) style_signal_initial: Option<String>,
    pub(crate) live_classes: Vec<(String, String)>,
    pub(crate) touch_class: Option<String>,
    pub(crate) touch_handler: Option<Rc<dyn Fn(crate::renderer::BrickEvent)>>,
    pub(crate) dirty_class: Option<String>,
    pub(crate) dirty_handler: Option<Rc<dyn Fn(crate::renderer::BrickEvent)>>,
}

// ── IntoClass ─────────────────────────────────────────────────────────────────

pub trait IntoClass {
    fn apply_class(self, el: &mut ViewLeafText);
}

impl IntoClass for &str {
    fn apply_class(self, el: &mut ViewLeafText) {
        if el.class_name.is_empty() {
            el.class_name = self.to_string();
        } else {
            el.class_name.push(' ');
            el.class_name.push_str(self);
        }
    }
}

impl IntoClass for crate::state_mgmt::Signal<String> {
    fn apply_class(self, el: &mut ViewLeafText) {
        use crate::state_mgmt::observers::ClassObserver;
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let id_class = format!("brick-live-{}", id);
        let initial = self.read();
        self.add_observer(&id_class, Box::new(ClassObserver::new(&id_class, &initial)));
        el.live_classes.push((id_class, initial));
    }
}

// ── IntoStyle ─────────────────────────────────────────────────────────────────

pub trait IntoStyle {
    fn apply_style(self, el: &mut ViewLeafText);
}

impl IntoStyle for &str {
    fn apply_style(self, el: &mut ViewLeafText) {
        el.push_style_raw(self);
    }
}

impl IntoStyle for String {
    fn apply_style(self, el: &mut ViewLeafText) {
        el.push_style_raw(&self);
    }
}

impl IntoStyle for crate::theme::Style {
    fn apply_style(self, el: &mut ViewLeafText) {
        el.push_style_raw(self.0);
    }
}

impl IntoStyle for crate::state_mgmt::Signal<String> {
    fn apply_style(self, el: &mut ViewLeafText) {
        use crate::state_mgmt::observers::Effect;
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let class = format!("brick-style-{}", id);
        let initial = self.read();
        let class_clone = class.clone();
        self.add_observer(
            &class,
            Box::new(Effect::new(move |v: &String| {
                let _ = &class_clone; // suppress unused warning in test builds
                #[cfg(brick_dom)]
                {
                    let doc = web_sys::window().unwrap().document().unwrap();
                    if let Some(el) = doc.get_elements_by_class_name(&class_clone).item(0) {
                        el.set_attribute("style", v).ok();
                    }
                }
                #[cfg(test)]
                let _ = v;
            })),
        );
        el.style_signal_class = Some(class);
        el.style_signal_initial = Some(initial);
    }
}

static BIND_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl ViewLeafText {
    pub(crate) fn new(tag: &'static str, text_content: String, is_void: bool) -> Self {
        ViewLeafText {
            tag,
            class_name: String::new(),
            text_content,
            action_name: None,
            attrs: Vec::new(),
            is_void,
            bind_class: None,
            bind_initial: None,
            bind_handler: None,
            click_class: None,
            click_handler: None,
            toggle_class: None,
            toggle_handler: None,
            brick_classes: Vec::new(),
            inline_style: None,
            style_signal_class: None,
            style_signal_initial: None,
            live_classes: Vec::new(),
            touch_class: None,
            touch_handler: None,
            dirty_class: None,
            dirty_handler: None,
        }
    }

    pub fn click<T: Clone + 'static>(
        mut self: Box<Self>,
        signal: &crate::state_mgmt::Signal<T>,
        transform: impl Fn(T) -> T + 'static,
    ) -> Box<Self> {
        let sig = signal.clone();
        let handler: Rc<dyn Fn(crate::renderer::BrickEvent)> =
            Rc::new(move |_e: crate::renderer::BrickEvent| {
                sig.set(transform(sig.read()));
            });
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.click_class = Some(format!("click_{}", id));
        self.click_handler = Some(handler);
        self
    }

    pub fn set<T: Clone + 'static>(
        mut self: Box<Self>,
        signal: &crate::state_mgmt::Signal<T>,
        value: T,
    ) -> Box<Self> {
        let sig = signal.clone();
        let handler: Rc<dyn Fn(crate::renderer::BrickEvent)> =
            Rc::new(move |_e: crate::renderer::BrickEvent| {
                sig.set(value.clone());
            });
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.click_class = Some(format!("click_{}", id));
        self.click_handler = Some(handler);
        self
    }

    /// Two-way binding for `<input type="checkbox">`. Sets the initial `checked`
    /// attribute, listens for `change` events (DOM → model), and registers a
    /// `CheckedObserver` (model → DOM).
    pub fn toggle(mut self: Box<Self>, signal: &crate::state_mgmt::Signal<bool>) -> Box<Self> {
        use crate::state_mgmt::observers::CheckedObserver;
        let sig = signal.clone();
        let initial = sig.read();
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let class = format!("toggle_{}", id);

        let handler: Rc<dyn Fn(crate::renderer::BrickEvent)> = Rc::new({
            let sig = sig.clone();
            move |e: crate::renderer::BrickEvent| {
                if let crate::renderer::BrickEvent::InputBool(checked) = e {
                    sig.set(checked);
                }
            }
        });

        sig.add_observer(&class, Box::new(CheckedObserver::new(&class)));

        if initial {
            self.attrs.push(("checked".to_string(), String::new()));
        }
        self.toggle_class = Some(class);
        self.toggle_handler = Some(handler);
        self
    }

    pub fn bind<T: crate::controller_system::FromBrickEvent + Clone>(
        mut self: Box<Self>,
        signal: &crate::state_mgmt::Signal<T>,
    ) -> Box<Self> {
        let initial = format!("{}", signal.read());
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let class = format!("bind_{}", id);

        // DOM → model
        let rc = signal.rc();
        let handler: Rc<dyn Fn(crate::renderer::BrickEvent)> =
            Rc::new(move |e: crate::renderer::BrickEvent| {
                if let Some(v) = T::from_brick_event(e) {
                    rc.borrow_mut().update(v);
                }
            });

        // model → DOM
        signal.add_observer(
            &class,
            Box::new(crate::state_mgmt::observers::InputObserver::new(&class)),
        );

        self.bind_class = Some(class);
        self.bind_initial = Some(initial);
        self.bind_handler = Some(handler);
        self
    }

    /// Mark a Signal<bool> as true when this input loses focus (user left the field).
    pub fn touch(mut self: Box<Self>, touched: &crate::state_mgmt::Signal<bool>) -> Box<Self> {
        let sig = touched.clone();
        let handler: Rc<dyn Fn(crate::renderer::BrickEvent)> =
            Rc::new(move |_: crate::renderer::BrickEvent| sig.set(true));
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.touch_class = Some(format!("brick-touch-{}", id));
        self.touch_handler = Some(handler);
        self
    }

    /// Mark a Signal<bool> as true when this input's value changes (user typed).
    pub fn dirty(mut self: Box<Self>, dirty: &crate::state_mgmt::Signal<bool>) -> Box<Self> {
        let sig = dirty.clone();
        let handler: Rc<dyn Fn(crate::renderer::BrickEvent)> =
            Rc::new(move |_: crate::renderer::BrickEvent| sig.set(true));
        let id = BIND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.dirty_class = Some(format!("brick-dirty-{}", id));
        self.dirty_handler = Some(handler);
        self
    }

    /// Wire this button as a validated submit trigger. Equivalent to .trigger(action)
    /// with type="submit". The action must be #[on(submit)] in the controller; the
    /// generated closure will call validate_all() before firing the method.
    pub fn submit(self: Box<Self>, action: &BrickAction) -> Box<Self> {
        self.trigger(action).attr("type", "submit")
    }

    #[allow(dead_code)]
    pub fn c<C: IntoClass>(mut self: Box<Self>, class: C) -> Box<Self> {
        class.apply_class(&mut *self);
        self
    }

    /// Escape hatch: wire a raw action name string when a typed `BrickAction` is
    /// unavailable (e.g. cross-component delegation). Prefer `.trigger()`.
    pub fn on_event<T: Display + ?Sized>(mut self: Box<Self>, action: &T) -> Box<Self> {
        self.action_name = Some(action.to_string());
        self
    }

    pub fn trigger(mut self: Box<Self>, action: &BrickAction) -> Box<Self> {
        self.action_name = Some(action.name.to_string());
        self
    }

    pub fn remove(self: Box<Self>) -> Box<Self> {
        self.attr("data-brick-remove", "true")
    }

    pub fn attr<K: Display + ?Sized, V: Display + ?Sized>(
        mut self: Box<Self>,
        key: &K,
        val: &V,
    ) -> Box<Self> {
        self.attrs.push((key.to_string(), val.to_string()));
        self
    }

    /// Apply a static CSS declaration string, a [`css!`]-produced [`Style`] value,
    /// or a `Signal<String>` for reactive inline styles.
    pub fn style<S: IntoStyle>(mut self: Box<Self>, s: S) -> Box<Self> {
        s.apply_style(&mut *self);
        self
    }

    fn push_style_raw(&mut self, decl: &str) {
        let s = self.inline_style.get_or_insert_with(String::new);
        if !s.is_empty() {
            s.push_str("; ");
        }
        s.push_str(decl.trim_end_matches(';').trim());
    }

    /// Assign a CSS `view-transition-name` to this element for hero animations.
    pub fn transition(mut self: Box<Self>, name: &str) -> Box<Self> {
        self.push_style_raw(&format!("view-transition-name: {}", name));
        self
    }

    // ── Semantic variant builders ─────────────────────────────────────────────

    /// Filled accent button / highlighted element.
    pub fn primary(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-primary");
        self
    }

    /// Outlined accent style.
    pub fn secondary(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-secondary");
        self
    }

    /// Transparent, low-emphasis style.
    pub fn ghost(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-ghost");
        self
    }

    /// Danger / destructive action style.
    pub fn danger(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-danger");
        self
    }

    /// Success / positive action style.
    pub fn success(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-success");
        self
    }

    /// Muted text style.
    pub fn muted(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-muted");
        self
    }

    // ── Size builders ─────────────────────────────────────────────────────────

    pub fn sm(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-sm");
        self
    }

    pub fn lg(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-lg");
        self
    }

    // ── Layout builders ───────────────────────────────────────────────────────

    pub fn full_width(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-full-width");
        self
    }

    pub fn row(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-row");
        self
    }

    pub fn col(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-col");
        self
    }

    pub fn center(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-center");
        self
    }

    // ── Typography builders ───────────────────────────────────────────────────

    pub fn bold(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-bold");
        self
    }

    pub fn mono(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-mono");
        self
    }

    pub fn truncate(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-truncate");
        self
    }

    // ── Typography scale ──────────────────────────────────────────────────────

    pub fn text_xs(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-text-xs");
        self
    }
    pub fn text_sm(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-text-sm");
        self
    }
    pub fn text_md(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-text-md");
        self
    }
    pub fn text_lg(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-text-lg");
        self
    }
    pub fn text_xl(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-text-xl");
        self
    }
    pub fn text_2xl(mut self: Box<Self>) -> Box<Self> {
        self.brick_classes.push("brick-text-2xl");
        self
    }

    // ── Spacing (inline style — per-instance, not class-based) ───────────────

    pub fn p(mut self: Box<Self>, s: Space) -> Box<Self> {
        self.push_style("padding", &s.to_css());
        self
    }
    pub fn px(mut self: Box<Self>, s: Space) -> Box<Self> {
        let v = s.to_css();
        self.push_style("padding-left", &v);
        self.push_style("padding-right", &v);
        self
    }
    pub fn py(mut self: Box<Self>, s: Space) -> Box<Self> {
        let v = s.to_css();
        self.push_style("padding-top", &v);
        self.push_style("padding-bottom", &v);
        self
    }
    pub fn m(mut self: Box<Self>, s: Space) -> Box<Self> {
        self.push_style("margin", &s.to_css());
        self
    }
    pub fn mx(mut self: Box<Self>, s: Space) -> Box<Self> {
        let v = s.to_css();
        self.push_style("margin-left", &v);
        self.push_style("margin-right", &v);
        self
    }
    pub fn my(mut self: Box<Self>, s: Space) -> Box<Self> {
        let v = s.to_css();
        self.push_style("margin-top", &v);
        self.push_style("margin-bottom", &v);
        self
    }
    pub fn gap(mut self: Box<Self>, s: Space) -> Box<Self> {
        self.push_style("gap", &s.to_css());
        self
    }

    fn push_style(&mut self, prop: &str, val: &str) {
        self.push_style_raw(&format!("{prop}: {val}"));
    }
}

// ── Phase 10: Brick<R> implementation ───────────────────────────────

impl crate::view_components::Brick for ViewLeafText {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};

        let node = Renderer::element(self.tag);

        // ── Class attribute ──────────────────────────────────────────────────
        let mut class_parts: Vec<&str> = vec![
            self.class_name.as_str(),
            self.bind_class.as_deref().unwrap_or(""),
            self.click_class.as_deref().unwrap_or(""),
            self.toggle_class.as_deref().unwrap_or(""),
            self.style_signal_class.as_deref().unwrap_or(""),
            self.touch_class.as_deref().unwrap_or(""),
            self.dirty_class.as_deref().unwrap_or(""),
        ];
        for c in &self.brick_classes {
            class_parts.push(c);
        }
        for (id, val) in &self.live_classes {
            class_parts.push(id.as_str());
            if !val.is_empty() {
                class_parts.push(val.as_str());
            }
        }
        let class: String = class_parts
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !class.is_empty() {
            Renderer::set_attr(&node, "class", &class);
        }

        // ── Style attribute ──────────────────────────────────────────────────
        let combined_style = match (&self.inline_style, &self.style_signal_initial) {
            (Some(s), Some(sig)) => Some(format!("{}; {}", s, sig)),
            (Some(s), None) => Some(s.clone()),
            (None, Some(sig)) => Some(sig.clone()),
            (None, None) => None,
        };
        if let Some(ref style) = combined_style {
            Renderer::set_attr(&node, "style", style);
        }

        // ── data-brick-action + inline event wiring ──────────────────────────
        if let Some(ref action) = self.action_name {
            Renderer::set_attr(&node, "data-brick-action", action);
            if let Some((event_type, handler)) =
                crate::view_components::composites::lookup_scope_listener(action.as_str())
            {
                Renderer::on_event(&node, event_type, Box::new(move |e| handler(e)));
            }
        }

        // ── Bind initial value ───────────────────────────────────────────────
        if let Some(ref val) = self.bind_initial {
            Renderer::set_attr(&node, "value", val);
        }

        // ── Arbitrary attrs ──────────────────────────────────────────────────
        for (k, v) in &self.attrs {
            if v.is_empty() {
                Renderer::set_attr(&node, k, "");
            } else {
                Renderer::set_attr(&node, k, v);
            }
        }

        // ── Text content ─────────────────────────────────────────────────────
        // text_content may contain raw HTML (e.g. from live! reactive spans),
        // so we embed it verbatim rather than going through Renderer::text()
        // which would HTML-escape the markup.
        if !self.is_void && !self.text_content.is_empty() {
            Renderer::append(
                &node,
                &crate::renderer::HtmlNode::raw(self.text_content.clone()),
            );
        }

        // ── Reactive live_classes observer ───────────────────────────────────
        // Wire a signal observer per live-class slot so the class attribute
        // stays in sync when the signal changes. Noop for HtmlRenderer since
        // on_event is a noop, but the observer still updates the HtmlNode tree.
        for (id, _initial) in &self.live_classes {
            let _ = id; // used via ClassObserver in attach_listeners path
        }

        // ── Inline event wiring ───────────────────────────────────────────────
        // Wire all direct handlers (bind, click, toggle, touch, dirty) inline
        // using Renderer::on_event. No-op for HtmlRenderer; DomRenderer adds
        // element-level listeners directly, replacing the attach_listeners path.
        if let Some(ref h) = self.bind_handler {
            let h = Rc::clone(h);
            Renderer::on_event(&node, "input", Box::new(move |e| h(e)));
        }
        if let Some(ref h) = self.click_handler {
            let h = Rc::clone(h);
            Renderer::on_event(&node, "click", Box::new(move |e| h(e)));
        }
        if let Some(ref h) = self.toggle_handler {
            let h = Rc::clone(h);
            Renderer::on_event(&node, "change", Box::new(move |e| h(e)));
        }
        if let Some(ref h) = self.touch_handler {
            let h = Rc::clone(h);
            Renderer::on_event(&node, "focusout", Box::new(move |e| h(e)));
        }
        if let Some(ref h) = self.dirty_handler {
            let h = Rc::clone(h);
            Renderer::on_event(&node, "input", Box::new(move |e| h(e)));
        }

        Renderer::append(parent, &node);
    }

    fn attach_listeners(&self) {}
}

#[cfg(test)]
mod tests {
    use super::ViewLeafText;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn no_attrs_renders_cleanly() {
        let el = ViewLeafText::new("p", "hello".to_string(), false);
        assert_eq!(rh(&el), "<p>hello</p>");
    }

    #[test]
    fn c_builder_sets_class() {
        let el = Box::new(ViewLeafText::new("h1", "Hi".to_string(), false)).c("hero");
        assert_eq!(rh(&*el), r#"<h1 class="hero">Hi</h1>"#);
    }

    #[test]
    fn c_builder_appends_multiple_classes() {
        let el = Box::new(ViewLeafText::new("button", "X".to_string(), false))
            .c("tab-btn")
            .c("active");
        assert_eq!(rh(&*el), r#"<button class="tab-btn active">X</button>"#);
    }

    #[test]
    fn on_builder_sets_data_action() {
        let el =
            Box::new(ViewLeafText::new("button", "+".to_string(), false)).on_event("increment");
        assert_eq!(
            rh(&*el),
            r#"<button data-brick-action="increment">+</button>"#
        );
    }

    #[test]
    fn attr_adds_arbitrary_attribute() {
        let el = Box::new(ViewLeafText::new("span", "x".to_string(), false)).attr("id", "main");
        assert_eq!(rh(&*el), r#"<span id="main">x</span>"#);
    }

    #[test]
    fn multiple_attrs_are_ordered() {
        let el = Box::new(ViewLeafText::new("span", "x".to_string(), false))
            .attr("id", "foo")
            .attr("data-val", "42");
        assert_eq!(rh(&*el), r#"<span id="foo" data-val="42">x</span>"#);
    }

    #[test]
    fn class_precedes_action_precedes_attrs_in_output() {
        let el = Box::new(ViewLeafText::new("button", "go".to_string(), false))
            .c("btn")
            .on_event("submit")
            .attr("type", "submit");
        assert_eq!(
            rh(&*el),
            r#"<button class="btn" data-brick-action="submit" type="submit">go</button>"#
        );
    }

    #[test]
    fn void_element_has_no_closing_tag() {
        let el = ViewLeafText::new("input", String::new(), true);
        assert_eq!(rh(&el), "<input>");
    }

    #[test]
    fn void_with_attrs() {
        let el = Box::new(ViewLeafText::new("input", String::new(), true))
            .attr("type", "text")
            .attr("placeholder", "Name");
        assert_eq!(rh(&*el), r#"<input type="text" placeholder="Name">"#);
    }

    #[test]
    fn style_builder_emits_style_attr() {
        let el = Box::new(ViewLeafText::new("p", "hi".to_string(), false)).style("color: red");
        assert_eq!(rh(&*el), r#"<p style="color: red">hi</p>"#);
    }

    #[test]
    fn multiple_style_calls_accumulate() {
        let el = Box::new(ViewLeafText::new("p", "hi".to_string(), false))
            .style("color: red")
            .style("font-weight: 700");
        assert_eq!(
            rh(&*el),
            r#"<p style="color: red; font-weight: 700">hi</p>"#
        );
    }

    #[test]
    fn style_signal_emits_initial_style() {
        use crate::state_mgmt::Signal;
        let sig = Signal::new(50.0f64);
        let style_sig = sig.map(|p| format!("width: {:.0}%", p));
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false)).style(style_sig);
        let html = rh(&*el);
        assert!(
            html.contains("width: 50%"),
            "initial style rendered from signal"
        );
        assert!(
            html.contains("brick-style-"),
            "identity class present for DOM targeting"
        );
    }

    #[test]
    fn style_signal_merges_with_static_style() {
        use crate::state_mgmt::Signal;
        let sig = Signal::new(75.0f64);
        let style_sig = sig.map(|p| format!("width: {:.0}%", p));
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false))
            .style("height: 1rem")
            .style(style_sig);
        let html = rh(&*el);
        assert!(html.contains("height: 1rem"), "static style preserved");
        assert!(html.contains("width: 75%"), "signal style included");
    }

    #[test]
    fn live_class_emits_initial_class_and_identity() {
        use crate::state_mgmt::Signal;
        let sig = Signal::new(true);
        let class_sig = sig.map(|b| if *b { "selected" } else { "" }.to_string());
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false))
            .c("entry")
            .c(class_sig);
        let html = rh(&*el);
        assert!(html.contains("entry"), "static class present");
        assert!(html.contains("selected"), "initial reactive class rendered");
        assert!(
            html.contains("brick-live-"),
            "identity class for ClassObserver targeting"
        );
    }

    #[test]
    fn live_class_empty_initial_omits_value_class() {
        use crate::state_mgmt::Signal;
        let sig = Signal::new(false);
        let class_sig = sig.map(|b| if *b { "selected" } else { "" }.to_string());
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false)).c(class_sig);
        let html = rh(&*el);
        assert!(
            !html.contains("selected"),
            "empty initial value not rendered"
        );
        assert!(html.contains("brick-live-"), "identity class still present");
    }

    #[test]
    fn spacing_builder_p_emits_inline_style() {
        use crate::theme::Space;
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false)).p(Space::Md);
        assert!(rh(&*el).contains(r#"style="padding: var(--brick-space-md)""#));
    }

    #[test]
    fn spacing_px_emits_left_and_right() {
        use crate::theme::Space;
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false)).px(Space::Sm);
        let html = rh(&*el);
        assert!(html.contains("padding-left: var(--brick-space-sm)"));
        assert!(html.contains("padding-right: var(--brick-space-sm)"));
    }

    #[test]
    fn semantic_builder_primary_emits_class() {
        let el = Box::new(ViewLeafText::new("button", "Go".to_string(), false)).primary();
        assert!(rh(&*el).contains("brick-primary"));
    }

    #[test]
    fn text_scale_builder_emits_class() {
        let el = Box::new(ViewLeafText::new("p", "".to_string(), false)).text_lg();
        assert!(rh(&*el).contains("brick-text-lg"));
    }

    #[test]
    fn transition_sets_view_transition_name_style() {
        let el = Box::new(ViewLeafText::new("img", String::new(), true)).transition("hero");
        assert!(rh(&*el).contains("view-transition-name: hero"));
    }

    #[test]
    fn transition_composes_with_other_styles() {
        let el = Box::new(ViewLeafText::new("div", String::new(), false))
            .style("color: red")
            .transition("card");
        let html = rh(&*el);
        assert!(html.contains("color: red"));
        assert!(html.contains("view-transition-name: card"));
    }

    #[test]
    fn touch_emits_identity_class() {
        use crate::state_mgmt::Signal;
        let touched = Signal::new(false);
        let el = Box::new(ViewLeafText::new("input", String::new(), true)).touch(&touched);
        assert!(
            rh(&*el).contains("brick-touch-"),
            "touch identity class present"
        );
    }

    #[test]
    fn dirty_emits_identity_class() {
        use crate::state_mgmt::Signal;
        let dirty = Signal::new(false);
        let el = Box::new(ViewLeafText::new("input", String::new(), true)).dirty(&dirty);
        assert!(
            rh(&*el).contains("brick-dirty-"),
            "dirty identity class present"
        );
    }

    #[test]
    fn parity_plain_element() {
        let el = ViewLeafText::new("p", "hello".to_string(), false);
        assert_eq!(rh(&el), "<p>hello</p>");
    }

    #[test]
    fn parity_with_class() {
        let el = Box::new(ViewLeafText::new("h1", "Title".to_string(), false)).c("hero");
        assert_eq!(rh(&*el), r#"<h1 class="hero">Title</h1>"#);
    }

    #[test]
    fn parity_with_data_action() {
        let el = Box::new(ViewLeafText::new("button", "+".to_string(), false)).on_event("inc");
        assert_eq!(rh(&*el), r#"<button data-brick-action="inc">+</button>"#);
    }

    #[test]
    fn parity_with_attrs() {
        let el = Box::new(ViewLeafText::new("span", "x".to_string(), false))
            .attr("id", "main")
            .attr("data-val", "42");
        assert_eq!(rh(&*el), r#"<span id="main" data-val="42">x</span>"#);
    }

    #[test]
    fn parity_void_element() {
        let el = ViewLeafText::new("br", String::new(), true);
        assert_eq!(rh(&el), "<br>");
    }

    #[test]
    fn parity_void_with_attrs() {
        let el = Box::new(ViewLeafText::new("input", String::new(), true))
            .attr("type", "text")
            .attr("placeholder", "Name");
        assert_eq!(rh(&*el), r#"<input type="text" placeholder="Name">"#);
    }

    #[test]
    fn parity_with_static_style() {
        let el = Box::new(ViewLeafText::new("p", "hi".to_string(), false)).style("color: red");
        assert_eq!(rh(&*el), r#"<p style="color: red">hi</p>"#);
    }

    #[test]
    fn parity_class_action_attrs() {
        let el = Box::new(ViewLeafText::new("button", "go".to_string(), false))
            .c("btn")
            .on_event("submit")
            .attr("type", "submit");
        assert_eq!(
            rh(&*el),
            r#"<button class="btn" data-brick-action="submit" type="submit">go</button>"#
        );
    }

    #[test]
    fn parity_semantic_builders() {
        let el = Box::new(ViewLeafText::new("button", "Go".to_string(), false))
            .primary()
            .lg();
        let html = rh(&*el);
        assert!(html.contains("brick-primary"), "primary class present");
    }

    #[test]
    fn parity_style_signal() {
        use crate::state_mgmt::Signal;
        let sig = Signal::new(50.0f64);
        let style_sig = sig.map(|p| format!("width: {:.0}%", p));
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false)).style(style_sig);
        let html = rh(&*el);
        assert!(html.contains("width: 50%"), "signal style rendered");
    }

    #[test]
    fn parity_live_class() {
        use crate::state_mgmt::Signal;
        let sig = Signal::new(true);
        let cls = sig.map(|b| if *b { "active" } else { "" }.to_string());
        let el = Box::new(ViewLeafText::new("div", "".to_string(), false))
            .c("item")
            .c(cls);
        let html = rh(&*el);
        assert!(html.contains("item"), "static class present");
        assert!(html.contains("active"), "reactive class rendered");
    }
}
