use super::Brick;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
static RDIV_COUNTER: AtomicUsize = AtomicUsize::new(0);
static MAPPED_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SVG_MAPPED_COUNTER: AtomicUsize = AtomicUsize::new(0);

// ─── ViewMapped ───────────────────────────────────────────────────────────────

/// A component that re-renders a subtree whenever a `Signal<T>` changes.
/// Used by `subroute!` to swap page content on route changes.
pub struct ViewMapped<T: Clone + 'static> {
    id: String,
    signal: crate::state_mgmt::Signal<T>,
    render_fn: Rc<dyn Fn(T) -> Box<dyn Brick>>,
    current: Rc<std::cell::RefCell<Box<dyn Brick>>>,
    use_view_transition: bool,
}

impl<T: Clone + 'static> ViewMapped<T> {
    pub fn new(
        signal: crate::state_mgmt::Signal<T>,
        f: impl Fn(T) -> Box<dyn Brick> + 'static,
    ) -> Box<Self> {
        let id = format!(
            "brick-mapped-{}",
            MAPPED_COUNTER.fetch_add(1, Ordering::SeqCst)
        );
        let render_fn: Rc<dyn Fn(T) -> Box<dyn Brick>> = Rc::new(f);
        let initial = (render_fn)(signal.read());
        let current = Rc::new(std::cell::RefCell::new(initial));
        Box::new(ViewMapped {
            id,
            signal,
            render_fn,
            current,
            use_view_transition: false,
        })
    }
}

impl<T: Clone + 'static> ViewMapped<T> {
    pub fn transition(mut self: Box<Self>) -> Box<Self> {
        self.use_view_transition = true;
        self
    }
}

// ─── SvgViewMapped ────────────────────────────────────────────────────────────

/// Like `ViewMapped` but wraps the subtree in `<g data-brick-svg-mapped="N">`
/// instead of a `<div>`, so it is valid inside an SVG document.
pub struct SvgViewMapped<T: Clone + 'static> {
    id: String,
    signal: crate::state_mgmt::Signal<T>,
    render_fn: Rc<dyn Fn(T) -> Box<dyn Brick>>,
    current: Rc<std::cell::RefCell<Box<dyn Brick>>>,
}

impl<T: Clone + 'static> SvgViewMapped<T> {
    pub fn new(
        signal: crate::state_mgmt::Signal<T>,
        f: impl Fn(T) -> Box<dyn Brick> + 'static,
    ) -> Box<Self> {
        let id = format!(
            "brick-svg-mapped-{}",
            SVG_MAPPED_COUNTER.fetch_add(1, Ordering::SeqCst)
        );
        let render_fn: Rc<dyn Fn(T) -> Box<dyn Brick>> = Rc::new(f);
        let initial = (render_fn)(signal.read());
        let current = Rc::new(std::cell::RefCell::new(initial));
        Box::new(SvgViewMapped {
            id,
            signal,
            render_fn,
            current,
        })
    }
}

impl<T: Clone + 'static> Brick for ViewMapped<T> {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, HtmlNode, Renderer};
        let wrapper = Renderer::element("div");
        Renderer::set_attr(&wrapper, "data-brick-mapped", &self.id);
        let inner_html = crate::view_components::render_to_html(&**self.current.borrow());
        if !inner_html.is_empty() {
            Renderer::append(&wrapper, &HtmlNode::raw(inner_html));
        }
        Renderer::append(parent, &wrapper);
    }

    fn attach_listeners(&self) {
        #[cfg(brick_dom)]
        {
            use crate::state_mgmt::observers::Effect;
            let render_fn = Rc::clone(&self.render_fn);
            let id = self.id.clone();
            let current = Rc::clone(&self.current);
            let use_view_transition = self.use_view_transition;
            self.signal.add_observer(
                &id.clone(),
                Box::new(Effect::new(move |val: &T| {
                    let new_content = (render_fn)(val.clone());
                    current.borrow().detach();
                    let doc = web_sys::window().unwrap().document().unwrap();
                    let sel = format!("[data-brick-mapped='{}']", id);
                    if use_view_transition {
                        let new_html = crate::view_components::render_to_html(&*new_content);
                        *current.borrow_mut() = new_content;
                        let current_c = Rc::clone(&current);
                        if let Ok(Some(el)) = doc.query_selector(&sel) {
                            crate::view_components::start_view_transition(move || {
                                el.set_inner_html(&new_html);
                            });
                        }
                    } else {
                        if let Ok(Some(el)) = doc.query_selector(&sel) {
                            el.set_inner_html(&crate::view_components::render_to_html(
                                &*new_content,
                            ));
                        }
                        *current.borrow_mut() = new_content;
                    }
                })),
            );
        }
    }

    fn detach(&self) {
        self.current.borrow().detach();
        self.signal.remove_observer(&self.id);
    }
}

impl<T: Clone + 'static> Brick for SvgViewMapped<T> {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, HtmlNode, Renderer};
        let wrapper = Renderer::element("g");
        Renderer::set_attr(&wrapper, "data-brick-svg-mapped", &self.id);
        let inner_html = crate::view_components::render_to_html(&**self.current.borrow());
        if !inner_html.is_empty() {
            Renderer::append(&wrapper, &HtmlNode::raw(inner_html));
        }
        Renderer::append(parent, &wrapper);
    }

    fn attach_listeners(&self) {
        #[cfg(brick_dom)]
        {
            use crate::state_mgmt::observers::Effect;
            let render_fn = Rc::clone(&self.render_fn);
            let id = self.id.clone();
            let current = Rc::clone(&self.current);
            self.signal.add_observer(
                &id.clone(),
                Box::new(Effect::new(move |val: &T| {
                    let new_content = (render_fn)(val.clone());
                    current.borrow().detach();
                    let doc = web_sys::window().unwrap().document().unwrap();
                    let sel = format!("[data-brick-svg-mapped='{}']", id);
                    if let Ok(Some(el)) = doc.query_selector(&sel) {
                        el.set_inner_html(&crate::view_components::render_to_html(&*new_content));
                    }
                    *current.borrow_mut() = new_content;
                })),
            );
        }
    }

    fn detach(&self) {
        self.current.borrow().detach();
        self.signal.remove_observer(&self.id);
    }
}

impl<T: Clone + 'static> super::IntoComponent for SvgViewMapped<T> {
    fn into_component(self) -> Box<dyn Brick> {
        Box::new(self)
    }
}

/// A div whose CSS class is driven by a `Signal<String>`. Constructed by the `div {}`
/// DSL when `class(signal)` receives a non-literal expression. The element gets a
/// stable identity class (`brick-rdiv-N`) plus the current signal value as a second
/// class; a `ClassObserver` swaps the second class whenever the signal changes.
pub struct ReactiveDiv {
    id_class: String,
    initial_class: String,
    signal: crate::state_mgmt::Signal<String>,
    children: Vec<Box<dyn Brick>>,
}

impl ReactiveDiv {
    pub fn new(
        signal: &crate::state_mgmt::Signal<String>,
        children: Vec<Box<dyn Brick>>,
    ) -> Box<Self> {
        let id = RDIV_COUNTER.fetch_add(1, Ordering::SeqCst);
        let id_class = format!("brick-rdiv-{}", id);
        let initial_class = signal.read();
        signal.add_observer(
            &id_class,
            Box::new(crate::state_mgmt::observers::ClassObserver::new(
                &id_class,
                &initial_class,
            )),
        );
        Box::new(ReactiveDiv {
            id_class,
            initial_class,
            signal: signal.clone(),
            children,
        })
    }
}

impl Brick for ReactiveDiv {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("div");
        let class = if self.initial_class.is_empty() {
            self.id_class.clone()
        } else {
            format!("{} {}", self.id_class, self.initial_class)
        };
        Renderer::set_attr(&node, "class", &class);
        for child in &self.children {
            child.render_into(&node);
        }
        let node_clone = node.clone();
        let id_class = self.id_class.clone();
        use crate::state_mgmt::observers::Effect;
        self.signal.add_observer(
            &self.id_class,
            Box::new(Effect::new(move |new_class: &String| {
                let combined = if new_class.is_empty() {
                    id_class.clone()
                } else {
                    format!("{} {}", id_class, new_class)
                };
                Renderer::set_attr(&node_clone, "class", &combined);
            })),
        );
        Renderer::append(parent, &node);
    }

    fn detach(&self) {
        self.signal.remove_observer(&self.id_class);
        for child in &self.children {
            child.detach();
        }
    }
}

#[macro_export]
macro_rules! make_composite {
    ( $class_name:expr, $event_listeners:expr, $lifecycle:expr, $intervals:expr, $sharing_model:expr, $raf_callbacks:expr, $kids:expr ) => {{
        let children: Vec<Box<dyn Brick>> = $kids;
        let is_fragment_root = children.len() == 1 && children[0].is_fragment();
        let mut lifecycle = $lifecycle;
        lifecycle
            .on_mount
            .extend($crate::view_components::drain_css_on_mount());
        Box::new(ViewComposite {
            tag: "div",
            class_name: $class_name,
            children,
            is_fragment_root,
            event_listeners: $event_listeners,
            lifecycle,
            intervals: $intervals,
            interval_handles: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            raf_callbacks: $raf_callbacks,
            raf_handles: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            model: $sharing_model,
        }) as Box<dyn Brick>
    }};
}

macro_rules! children {
    ( $( $child:expr ),* $(,)* ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        vec![ $( $child.into_component() ),* ]
    }};
}

/// A structural container — renders children inside `<div class="$class">`.
/// Constructed by the `div {}` DSL when `class("literal")` is used.
pub struct BrickContainer {
    pub class: &'static str,
    pub children: Vec<Box<dyn Brick>>,
}

impl Brick for BrickContainer {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("div");
        if !self.class.is_empty() {
            Renderer::set_attr(&node, "class", self.class);
        }
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }

    fn detach(&self) {
        for child in &self.children {
            child.detach();
        }
    }
}

/// A non-reactive flex-row wrapper. Renders children in a `<div class="brick-row">` so they
/// sit side-by-side with the `--brick-space-sm` gap. Event listeners delegate upward normally.
pub struct BrickRow {
    pub children: Vec<Box<dyn Brick>>,
}

impl Brick for BrickRow {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("div");
        Renderer::set_attr(&node, "class", "brick-row");
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }

    fn detach(&self) {
        for child in &self.children {
            child.detach();
        }
    }
}

impl BrickRow {
    pub fn boxed(children: Vec<Box<dyn Brick>>) -> Box<dyn Brick> {
        Box::new(BrickRow { children })
    }
}

/// A wrapper-free group of sibling nodes. Renders children inline with no
/// enclosing element — useful for `<dt>`/`<dd>` pairs, `<tr>`/`<td>` rows,
/// and any situation where an extra `<div>` would violate HTML structure.
pub struct BrickFragment {
    pub children: Vec<Box<dyn Brick>>,
}

impl Brick for BrickFragment {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        for child in &self.children {
            child.render_into(parent);
        }
    }

    fn detach(&self) {
        for child in &self.children {
            child.detach();
        }
    }

    fn is_fragment(&self) -> bool {
        true
    }
}

impl super::IntoComponent for BrickFragment {
    fn into_component(self) -> Box<dyn Brick> {
        Box::new(self)
    }
}

impl BrickFragment {
    pub fn boxed(children: Vec<Box<dyn Brick>>) -> Box<dyn Brick> {
        Box::new(BrickFragment { children })
    }
}

/// Group sibling nodes with no wrapper element.
///
/// ```rust,ignore
/// fn definition_pair(term: &str, def: &str) -> impl Brick {
///     fragment! { dt(term), dd(def) }
/// }
/// ```
#[allow(unused_macros)]
macro_rules! fragment {
    ( $( $child:expr ),* $(,)* ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        $crate::view_components::BrickFragment::boxed(vec![$( $child.into_component() ),*])
    }};
}

/// Wrap children in a flex row with `--brick-space-sm` gap.
///
/// ```rust,ignore
/// row! {
///     button("A").primary(),
///     button("B").secondary(),
/// }
/// ```
macro_rules! row {
    ( $( $child:expr ),* $(,)* ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        BrickRow::boxed(vec![$( $child.into_component() ),*])
    }};
}

#[allow(unused_macros)]
/// Wrap any single [`IntoComponent`] expression in a [`Slot`] for injection
/// into a `#[slot]`-annotated model field.
///
/// ```rust,ignore
/// Card { children: slot!(row! { button("Save"), button("Cancel") }), ..cascade() }
/// ```
macro_rules! slot {
    ($e:expr) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        $crate::view_components::Slot::from($e.into_component())
    }};
}

// Reactive list rendering: list!(my.todos, |item| ...) or list!(my.cart.items, |item| ...)
// Indexed form: list!(my.todos, |item, idx| ...)
#[allow(unused_macros)]
macro_rules! list {
    ( $path:expr, | $item:ident | $body:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        Box::new($crate::view_components::ViewList::new(
            std::rc::Rc::clone(&$path),
            move |$item, _| ($body).into_component(),
        ))
    }};
    ( $path:expr, | $item:ident, $idx:ident | $body:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        Box::new($crate::view_components::ViewList::new(
            std::rc::Rc::clone(&$path),
            move |$item, $idx| ($body).into_component(),
        ))
    }};
}

// FLIP-animated list: items that move when a sibling is removed animate smoothly.
// flip!(my.items, |item| ...) or flip!(my.items, Ms(200), |item| ...)
// Indexed forms: flip!(my.items, |item, idx| ...) or flip!(my.items, Ms(200), |item, idx| ...)
#[allow(unused_macros)]
macro_rules! flip {
    ( $path:expr, | $item:ident | $body:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        Box::new($crate::view_components::FlipList::new(
            std::rc::Rc::clone(&$path),
            move |$item, _| ($body).into_component(),
        ))
    }};
    ( $path:expr, | $item:ident, $idx:ident | $body:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        Box::new($crate::view_components::FlipList::new(
            std::rc::Rc::clone(&$path),
            move |$item, $idx| ($body).into_component(),
        ))
    }};
    ( $path:expr, $duration:expr, | $item:ident | $body:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        Box::new($crate::view_components::FlipList::with_duration(
            std::rc::Rc::clone(&$path),
            move |$item, _| ($body).into_component(),
            $duration,
        ))
    }};
    ( $path:expr, $duration:expr, | $item:ident, $idx:ident | $body:expr $(,)? ) => {{
        #[allow(unused_imports)]
        use $crate::view_components::IntoComponent;
        Box::new($crate::view_components::FlipList::with_duration(
            std::rc::Rc::clone(&$path),
            move |$item, $idx| ($body).into_component(),
            $duration,
        ))
    }};
}

// Borrow #[store] or #[global] model fields into local variables for ergonomic
// field access in a view body. Relies on `my` being in scope (injected by #[view]).
//   stores!(cart, auth)
//   expands to: let cart = my.cart.clone(); let auth = my.auth.clone();
#[allow(unused_macros)]
macro_rules! stores {
    ( $( $field:ident ),+ $(,)? ) => {
        $( let $field = my.$field.clone(); )+
    };
}

// Conditional rendering: when!(my.flag, true_branch) or
// when!(my.flag, true_branch, false_branch)
// Default mode is .hide(); call .mount() to swap innerHTML instead.
#[allow(unused_macros)]
macro_rules! when {
    ( $my:ident . $field:ident, $true_branch:expr $(,)? ) => {
        Box::new(crate::view_components::ViewConditional::new(
            $my.$field.clone(),
            $true_branch,
            None,
        ))
    };
    ( $my:ident . $field:ident, $true_branch:expr, $false_branch:expr $(,)? ) => {
        Box::new(crate::view_components::ViewConditional::new(
            $my.$field.clone(),
            $true_branch,
            Some($false_branch),
        ))
    };
}

/// Render a reactive subtree driven by a `Signal<T>`.  Re-renders whenever the
/// signal changes — the primary use-case is route dispatch.
///
/// ```rust,ignore
/// subroute!(app_route(), |route| match route {
///     AppRoute::Home  => HomePage { ..cascade() }.into_component(),
///     AppRoute::About => AboutPage { ..cascade() }.into_component(),
///     _               => p("Not found").into_component(),
/// })
/// ```
#[allow(unused_macros)]
macro_rules! subroute {
    ( $signal:expr, | $var:ident | $body:expr $(,)? ) => {
        Box::new($crate::view_components::ViewMapped::new(
            $signal,
            move |$var| $body,
        ))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::Signal;
    use crate::view_components::leafs::p;
    use crate::view_components::{Brick, IntoComponent};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    struct DetachSpy(Rc<RefCell<bool>>);
    impl Brick for DetachSpy {
        fn render_into(&self, _parent: &crate::renderer::BrickNode) {}
        fn detach(&self) {
            *self.0.borrow_mut() = true;
        }
    }

    // ── BrickContainer ──────────────────────────────────────────────────────────

    #[test]
    fn brick_container_renders_with_class() {
        let c = BrickContainer {
            class: "my-row",
            children: vec![p("hello")],
        };
        assert_eq!(rh(&c), r#"<div class="my-row"><p>hello</p></div>"#);
    }

    #[test]
    fn brick_container_renders_without_class() {
        let c = BrickContainer {
            class: "",
            children: vec![],
        };
        assert_eq!(rh(&c), "<div></div>");
    }

    #[test]
    fn brick_container_detach_calls_children() {
        let flag = Rc::new(RefCell::new(false));
        let spy = Box::new(DetachSpy(Rc::clone(&flag)));
        let c = BrickContainer {
            class: "wrap",
            children: vec![spy as Box<dyn Brick>],
        };
        <BrickContainer as Brick>::detach(&c);
        assert!(
            *flag.borrow(),
            "BrickContainer.detach() should propagate to children"
        );
    }

    // ── BrickRow ────────────────────────────────────────────────────────────────

    #[test]
    fn brick_row_renders_children_in_brick_row_div() {
        let r = BrickRow {
            children: vec![p("a"), p("b")],
        };
        let html = rh(&r);
        assert!(
            html.contains(r#"class="brick-row""#),
            "should have brick-row class"
        );
        assert!(html.contains("<p>a</p>"), "should contain first child");
        assert!(html.contains("<p>b</p>"), "should contain second child");
    }

    // ── ViewMapped ──────────────────────────────────────────────────────────────

    #[test]
    fn view_mapped_html_uses_current_component() {
        let m = ViewMapped::new(Signal::new("hello".to_string()), |s| -> Box<dyn Brick> {
            p(&s)
        });
        let html = rh(&*m);
        assert!(html.contains("hello"), "should render current signal value");
        assert!(
            html.contains("data-brick-mapped"),
            "should have data-brick-mapped attribute"
        );
    }

    // ── SvgViewMapped ───────────────────────────────────────────────────────────

    #[test]
    fn svg_view_mapped_uses_g_container() {
        use super::SvgViewMapped;
        use crate::view_components::svg::shapes::svg_circle;
        let m = SvgViewMapped::new(Signal::new(42.0_f64), |r| -> Box<dyn Brick> {
            Box::new(svg_circle(50.0, 50.0, r))
        });
        let html = rh(&*m);
        assert!(html.starts_with("<g "), "container must be <g");
        assert!(
            html.contains("data-brick-svg-mapped"),
            "must have data-brick-svg-mapped attr"
        );
        assert!(
            html.contains(r#"r="42""#),
            "should render current signal value"
        );
        assert!(html.ends_with("</g>"));
    }

    #[test]
    fn fragment_empty_renders_empty_string() {
        let f = BrickFragment { children: vec![] };
        assert_eq!(rh(&f), "");
    }

    #[test]
    fn fragment_renders_siblings_without_wrapper() {
        let f = BrickFragment {
            children: vec![p("a"), p("b")],
        };
        assert_eq!(rh(&f), "<p>a</p><p>b</p>");
    }

    #[test]
    fn fragment_macro_syntax() {
        let f = fragment! { p("x"), p("y"), p("z") };
        assert_eq!(rh(&*f), "<p>x</p><p>y</p><p>z</p>");
    }

    #[test]
    fn fragment_detach_recurses() {
        let flag = Rc::new(RefCell::new(false));
        let spy = DetachSpy(Rc::clone(&flag));
        let f = BrickFragment {
            children: vec![Box::new(spy)],
        };
        <BrickFragment as Brick>::detach(&f);
        assert!(*flag.borrow(), "detach should recurse into children");
    }
}
