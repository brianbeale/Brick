pub mod blueprint;
pub use blueprint::encode_blueprint;

use crate::renderer::BrickNode;
use crate::state_mgmt::{BrickAction, Signal};
use crate::view_components::Brick;
use std::sync::atomic::{AtomicU32, Ordering};

// ── Signal ID counter ─────────────────────────────────────────────────────────

static PORTABLE_SIG_COUNTER: AtomicU32 = AtomicU32::new(1);

pub(crate) fn next_signal_id() -> u32 {
    PORTABLE_SIG_COUNTER.fetch_add(1, Ordering::SeqCst)
}

// ── Drain queue (Android only) ────────────────────────────────────────────────

/// Pending signal updates for the Kotlin drain loop.
/// Each entry is `(signal_id, utf-8 value bytes)`.
/// Kotlin calls `brick_drain_queue()` once per Compose frame via `withFrameNanos`.
/// Thread-local because `Signal<T>` is `!Send` — all JNI calls land on the Android main thread.
#[cfg(brick_android)]
thread_local! {
    pub(crate) static SIGNAL_QUEUE: std::cell::RefCell<std::collections::VecDeque<(u32, Vec<u8>)>> =
        std::cell::RefCell::new(std::collections::VecDeque::new());
}

/// Drain all pending updates out of the queue in one shot.
#[cfg(brick_android)]
pub(crate) fn drain_signal_queue() -> Vec<(u32, Vec<u8>)> {
    SIGNAL_QUEUE.with(|q| q.borrow_mut().drain(..).collect())
}

/// Push a full-re-render sentinel (`signal_id = 0`, tag `0xFF`) to the queue.
/// Kotlin interprets this as "call `brickXxxBlueprintBytes()` again and swap the tree."
/// No-op on non-Android targets.
pub fn push_rerender_signal() {
    #[cfg(brick_android)]
    SIGNAL_QUEUE.with(|q| q.borrow_mut().push_back((0, vec![0xFF])));
}

/// Reset the signal ID counter to 1. Called by `clearAndroidState` before loading a
/// new demo so every demo's signal IDs start from a known baseline.
#[cfg(brick_android)]
pub fn reset_signal_counter() {
    PORTABLE_SIG_COUNTER.store(1, std::sync::atomic::Ordering::SeqCst);
}

// ── ButtonVariant / TextVariant ───────────────────────────────────────────────

/// Semantic button style. Maps to Material 3 composables on Android,
/// CSS class variants on web, and UIKit button styles on iOS.
///
/// | Variant   | Material 3           | Web CSS class     |
/// |-----------|----------------------|-------------------|
/// | Primary   | `Button` (filled)    | `brick-primary`   |
/// | Secondary | `FilledTonalButton`  | `brick-tonal`     |
/// | Outlined  | `OutlinedButton`     | `brick-secondary` |
/// | Ghost     | `TextButton`         | `brick-ghost`     |
/// | Danger         | `Button` (crimson filled)   | `brick-danger`    |
/// | DangerOutlined | `OutlinedButton` (crimson)  | `brick-danger`    |
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary   = 0,
    Secondary = 1,
    Outlined  = 2,
    Ghost     = 3,
    Danger         = 4,
    DangerOutlined = 5,
}

/// Semantic text style. Maps to Material 3 typography on Android.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextVariant {
    #[default]
    Body    = 0,
    Title   = 1,
    Caption = 2,
    Label   = 3,
}

// ── Navigator ─────────────────────────────────────────────────────────────────

/// A back-stack navigator for multi-screen portable apps.
///
/// Use as a model field annotated with `#[default(InitialScreen)]`:
/// ```rust,ignore
/// #[model]
/// struct App {
///     #[default(Screen::Home)]
///     nav: Navigator<Screen>,
/// }
/// ```
///
/// `push` and `pop` call `push_rerender_signal()` automatically, triggering a
/// full blueprint re-fetch on Android. The portable block pattern-matches on
/// `self.nav.current()` to return the correct screen's subtree.
pub struct Navigator<S: Clone> {
    stack: std::cell::RefCell<Vec<S>>,
}

impl<S: Clone> Navigator<S> {
    pub fn new(initial: S) -> Self {
        Self { stack: std::cell::RefCell::new(vec![initial]) }
    }

    pub fn push(&mut self, screen: S) {
        self.stack.borrow_mut().push(screen);
        push_rerender_signal();
    }

    pub fn pop(&mut self) -> bool {
        if self.stack.borrow().len() > 1 {
            self.stack.borrow_mut().pop();
            push_rerender_signal();
            true
        } else {
            false
        }
    }

    pub fn current(&self) -> std::cell::Ref<'_, S> {
        std::cell::Ref::map(self.stack.borrow(), |v| v.last().unwrap())
    }

    pub fn can_go_back(&self) -> bool {
        self.stack.borrow().len() > 1
    }

    pub fn depth(&self) -> usize {
        self.stack.borrow().len()
    }
}

impl<S: Clone> Clone for Navigator<S> {
    fn clone(&self) -> Self {
        Self { stack: std::cell::RefCell::new(self.stack.borrow().clone()) }
    }
}

// ── IntentType ────────────────────────────────────────────────────────────────

/// Platform Intent to launch. Used with `intent_button()` to produce
/// `BlueprintNode::ExternalIntent`. Each platform maps these to its native
/// mechanism: Android Intents, iOS UIActivityViewController / UIApplication.open,
/// web Share API / window.open.
#[derive(Clone, Debug)]
pub enum IntentType {
    /// Share plain text via the system share sheet.
    ShareText { text: String },
    /// Open a URL in the default browser.
    OpenUrl { url: String },
    /// Launch the camera; `result_action` receives the image path via `dispatchString`.
    LaunchCamera { result_action: String },
    /// Open the image picker; `result_action` receives the image path via `dispatchString`.
    PickImage { result_action: String },
    /// Open the app's system settings page.
    OpenSettings,
    /// Pre-populate an email composer.
    ComposeEmail { to: String, subject: String },
}

// ── SignalEncoding ────────────────────────────────────────────────────────────

/// Wire-format encoding for scalar signal values sent through the shared
/// `ByteBuffer` to native renderers (Android, iOS).
///
/// Every type that can be used as a reactive signal in a portable view must
/// implement this trait. The `Display` supertrait provides the initial-value
/// string embedded in the `BrickViewBlueprint` at mount time.
///
/// Wire format per entry in the signal buffer:
/// ```text
/// [signal_id: u32 LE][WIRE_TAG: u8][value_bytes ...]
/// ```
pub trait SignalEncoding: std::fmt::Display {
    /// One-byte discriminant identifying the type on the native side.
    const WIRE_TAG: u8;
    /// Append the encoded value bytes (no tag prefix) to `buf`.
    fn write_bytes(&self, buf: &mut Vec<u8>);
}

impl SignalEncoding for i8 {
    const WIRE_TAG: u8 = 0x01;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.push(*self as u8); }
}
impl SignalEncoding for u8 {
    const WIRE_TAG: u8 = 0x02;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.push(*self); }
}
impl SignalEncoding for i16 {
    const WIRE_TAG: u8 = 0x03;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for u16 {
    const WIRE_TAG: u8 = 0x04;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for i32 {
    const WIRE_TAG: u8 = 0x05;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for u32 {
    const WIRE_TAG: u8 = 0x06;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for i64 {
    const WIRE_TAG: u8 = 0x07;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for u64 {
    const WIRE_TAG: u8 = 0x08;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for isize {
    const WIRE_TAG: u8 = 0x09;
    // Always 8 bytes: on 32-bit targets isize is sign-extended to i64.
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&(*self as i64).to_le_bytes()); }
}
impl SignalEncoding for usize {
    const WIRE_TAG: u8 = 0x0A;
    // Always 8 bytes: on 32-bit targets usize is zero-extended to u64.
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&(*self as u64).to_le_bytes()); }
}
impl SignalEncoding for f32 {
    const WIRE_TAG: u8 = 0x0B;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for f64 {
    const WIRE_TAG: u8 = 0x0C;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
}
impl SignalEncoding for bool {
    const WIRE_TAG: u8 = 0x0D;
    fn write_bytes(&self, buf: &mut Vec<u8>) { buf.push(*self as u8); }
}
impl SignalEncoding for String {
    const WIRE_TAG: u8 = 0x0E;
    fn write_bytes(&self, buf: &mut Vec<u8>) {
        let bytes = self.as_bytes();
        // Saturate at u16::MAX; a 65 KB string in a signal is pathological.
        let len = (bytes.len().min(u16::MAX as usize)) as u16;
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&bytes[..len as usize]);
    }
}

// ── Signal subscription ───────────────────────────────────────────────────────

/// Register `signal` so every future value change pushes a typed wire entry
/// `(signal_id, [WIRE_TAG][value_bytes...])` to the drain queue.
/// No-op on non-Android targets.
pub fn register_signal_subscription<T: SignalEncoding + Clone + 'static>(
    signal: &Signal<T>,
    signal_id: u32,
) {
    #[cfg(brick_android)]
    {
        use crate::state_mgmt::observers::Effect;
        let id = signal_id;
        signal.add_observer(
            &format!("brick-portable-{}", id),
            Box::new(Effect::new(move |v: &T| {
                let mut bytes = vec![T::WIRE_TAG];
                v.write_bytes(&mut bytes);
                SIGNAL_QUEUE.with(|q| q.borrow_mut().push_back((id, bytes)));
            })),
        );
    }
    #[cfg(not(brick_android))]
    {
        let _ = (signal, signal_id);
    }
}

// ── PortableContent ───────────────────────────────────────────────────────────

pub enum PortableContent {
    Static(String),
    Reactive { signal_id: u32, initial: String },
}

impl PortableContent {
    pub fn text(&self) -> &str {
        match self {
            PortableContent::Static(s) => s.as_str(),
            PortableContent::Reactive { initial, .. } => initial.as_str(),
        }
    }
}

// ── IntoPortableContent ───────────────────────────────────────────────────────

pub trait IntoPortableContent {
    fn into_portable_content(self) -> PortableContent;
}

impl IntoPortableContent for &str {
    fn into_portable_content(self) -> PortableContent {
        PortableContent::Static(self.to_string())
    }
}

impl IntoPortableContent for String {
    fn into_portable_content(self) -> PortableContent {
        PortableContent::Static(self)
    }
}

impl<T: SignalEncoding + Clone + 'static> IntoPortableContent for &Signal<T> {
    fn into_portable_content(self) -> PortableContent {
        let id = next_signal_id();
        let initial = format!("{}", self.read());
        register_signal_subscription(self, id);
        PortableContent::Reactive { signal_id: id, initial }
    }
}

// ── BlueprintNode ─────────────────────────────────────────────────────────────

/// A platform-agnostic description of a portable view tree.
/// Built by `PortableView::blueprint_node()`; consumed by `src/native/` on Android
/// and by Swift-side renderers on iOS (future).
pub enum BlueprintNode {
    Text { content: PortableContent, variant: TextVariant },
    Button {
        label: String,
        action: Option<String>,
        /// Method name for index-based dispatch; paired with `int_payload`.
        int_action: Option<String>,
        /// Index value sent with `dispatchInt` when `int_action` is set.
        int_payload: Option<i32>,
        variant: ButtonVariant,
    },
    Input {
        placeholder: String,
        value_signal_id: Option<u32>,
        change_action: Option<String>,
    },
    Column(Vec<BlueprintNode>),
    Row(Vec<BlueprintNode>),
    Scroll(Vec<BlueprintNode>),
    /// Cross-platform boolean toggle. Android → `Switch`; web → `<input type="checkbox">`.
    Toggle {
        label: String,
        checked_signal_id: Option<u32>,
        change_action: Option<String>,
    },
    /// Launch a platform-native external intent (share, URL, camera, etc.).
    ExternalIntent { label: String, intent_type: IntentType },
    /// Request a runtime permission; on Android renders as a button that opens the system dialog.
    PermissionRequest { label: String, permission: String, result_action: String },
}

// ── PortableView ──────────────────────────────────────────────────────────────

/// Implemented by any type that can produce a platform-agnostic view tree.
/// Leaf/container portable elements AND model structs (via `#[view]` with a
/// `portable! {}` block) implement this trait.
pub trait PortableView {
    fn blueprint_node(&self) -> BlueprintNode;
}

/// Combined marker for portable *elements* — nodes that support both web rendering
/// via `Brick::render_into` and native blueprint generation via `PortableView::blueprint_node`.
/// Model structs implement only `PortableView`; leaf/container elements implement both.
pub trait PortableElement: PortableView + Brick {}
impl<T: PortableView + Brick + 'static> PortableElement for T {}

/// Erase the concrete portable element type to `Box<dyn PortableElement>`.
/// Used when building `column()` / `row()` / `scroll()` child lists.
pub fn into_portable<T: PortableElement + 'static>(b: Box<T>) -> Box<dyn PortableElement> {
    b
}

// ── PortableText — p() ────────────────────────────────────────────────────────

pub struct PortableText {
    pub content: PortableContent,
    pub variant: TextVariant,
}

impl PortableText {
    pub fn title(mut self: Box<Self>) -> Box<Self> { self.variant = TextVariant::Title; self }
    pub fn caption(mut self: Box<Self>) -> Box<Self> { self.variant = TextVariant::Caption; self }
    pub fn label(mut self: Box<Self>) -> Box<Self> { self.variant = TextVariant::Label; self }
}

/// Cross-platform text paragraph. Accepts `&str`, `String`, or `&Signal<T>`.
/// On web → `<p>`. On Android → `Text` composable. On iOS → `Text` view (future).
pub fn p<C: IntoPortableContent>(content: C) -> Box<PortableText> {
    Box::new(PortableText { content: content.into_portable_content(), variant: TextVariant::Body })
}

impl Brick for PortableText {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("p");
        let class = match self.variant {
            TextVariant::Body    => "",
            TextVariant::Title   => "brick-title",
            TextVariant::Caption => "brick-caption",
            TextVariant::Label   => "brick-label",
        };
        if !class.is_empty() {
            Renderer::set_attr(&node, "class", class);
        }
        Renderer::append(&node, &Renderer::text(self.content.text()));
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortableText {
    fn blueprint_node(&self) -> BlueprintNode {
        let content = match &self.content {
            PortableContent::Static(s) => PortableContent::Static(s.clone()),
            PortableContent::Reactive { signal_id, initial } => {
                PortableContent::Reactive { signal_id: *signal_id, initial: initial.clone() }
            }
        };
        BlueprintNode::Text { content, variant: self.variant }
    }
}

// ── PortableButton — button() ─────────────────────────────────────────────────

pub struct PortableButton {
    pub label: String,
    pub action: Option<String>,
    pub int_action: Option<String>,
    pub int_payload: Option<i32>,
    pub variant: ButtonVariant,
}

impl PortableButton {
    /// Bind a controller action to this button's tap/click event.
    pub fn trigger(mut self: Box<Self>, action: &BrickAction) -> Box<Self> {
        self.action = Some(action.name.to_string());
        self
    }

    /// Bind an index-based controller action. Renders `dispatchInt(action, index)` on click.
    /// Used for per-item actions in lists (delete item, select row, etc.).
    pub fn trigger_int(mut self: Box<Self>, action: &BrickAction, index: i32) -> Box<Self> {
        self.int_action = Some(action.name.to_string());
        self.int_payload = Some(index);
        self
    }

    pub fn primary(mut self: Box<Self>) -> Box<Self> { self.variant = ButtonVariant::Primary; self }
    pub fn secondary(mut self: Box<Self>) -> Box<Self> { self.variant = ButtonVariant::Secondary; self }
    pub fn outlined(mut self: Box<Self>) -> Box<Self> { self.variant = ButtonVariant::Outlined; self }
    pub fn ghost(mut self: Box<Self>) -> Box<Self> { self.variant = ButtonVariant::Ghost; self }
    pub fn danger(mut self: Box<Self>) -> Box<Self> { self.variant = ButtonVariant::Danger; self }
    pub fn danger_outlined(mut self: Box<Self>) -> Box<Self> { self.variant = ButtonVariant::DangerOutlined; self }
}

/// Cross-platform button. On web → `<button>`. On Android → `Button` composable.
pub fn button(label: &str) -> Box<PortableButton> {
    Box::new(PortableButton {
        label: label.to_string(),
        action: None,
        int_action: None,
        int_payload: None,
        variant: ButtonVariant::Primary,
    })
}

impl Brick for PortableButton {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("button");
        let class = match self.variant {
            ButtonVariant::Primary   => "brick-primary",
            ButtonVariant::Secondary => "brick-tonal",
            ButtonVariant::Outlined  => "brick-secondary",
            ButtonVariant::Ghost     => "brick-ghost",
            ButtonVariant::Danger |
            ButtonVariant::DangerOutlined => "brick-danger",
        };
        Renderer::set_attr(&node, "class", class);
        if let Some(ref name) = self.action {
            Renderer::set_attr(&node, "data-brick-action", name);
        }
        Renderer::append(&node, &Renderer::text(&self.label));
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortableButton {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::Button {
            label: self.label.clone(),
            action: self.action.clone(),
            int_action: self.int_action.clone(),
            int_payload: self.int_payload,
            variant: self.variant,
        }
    }
}

// ── PortableInput — input() ───────────────────────────────────────────────────

pub struct PortableInput {
    pub placeholder: String,
    pub value_signal_id: Option<u32>,
    pub change_action: Option<String>,
}

impl PortableInput {
    pub fn placeholder(mut self: Box<Self>, text: &str) -> Box<Self> {
        self.placeholder = text.to_string();
        self
    }

    /// Two-way bind: registers `signal` to the drain queue and links its ID
    /// so the native renderer can pre-fill the input with the current value.
    pub fn bind<T: SignalEncoding + Clone + 'static>(
        mut self: Box<Self>,
        signal: &Signal<T>,
    ) -> Box<Self> {
        let id = next_signal_id();
        register_signal_subscription(signal, id);
        self.value_signal_id = Some(id);
        self
    }

    pub fn on_change(mut self: Box<Self>, action: &BrickAction) -> Box<Self> {
        self.change_action = Some(action.name.to_string());
        self
    }
}

/// Cross-platform text input. On web → `<input>`. On Android → `TextField` composable.
pub fn input() -> Box<PortableInput> {
    Box::new(PortableInput {
        placeholder: String::new(),
        value_signal_id: None,
        change_action: None,
    })
}

impl Brick for PortableInput {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("input");
        if !self.placeholder.is_empty() {
            Renderer::set_attr(&node, "placeholder", &self.placeholder);
        }
        if let Some(ref name) = self.change_action {
            Renderer::set_attr(&node, "data-brick-action", name);
        }
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortableInput {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::Input {
            placeholder: self.placeholder.clone(),
            value_signal_id: self.value_signal_id,
            change_action: self.change_action.clone(),
        }
    }
}

// ── PortableColumn — column() / column![] ─────────────────────────────────────

pub struct PortableColumn {
    pub children: Vec<Box<dyn PortableElement>>,
}

/// Cross-platform vertical stack. On web → `<div style="display:flex;flex-direction:column">`.
/// On Android → `Column` composable.
pub fn column(children: Vec<Box<dyn PortableElement>>) -> Box<PortableColumn> {
    Box::new(PortableColumn { children })
}

/// Ergonomic macro for `column()`. Each child is automatically type-erased.
///
/// ```rust,ignore
/// column![p("hello"), button("Click").trigger(&model.submit)]
/// ```
#[macro_export]
macro_rules! column {
    [$($child:expr),* $(,)?] => {
        $crate::portable::column(vec![$($crate::portable::into_portable($child)),*])
    };
}

impl Brick for PortableColumn {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("div");
        Renderer::set_attr(&node, "style", "display:flex;flex-direction:column");
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortableColumn {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::Column(self.children.iter().map(|c| c.blueprint_node()).collect())
    }
}

// ── PortableRow — row() ───────────────────────────────────────────────────────

pub struct PortableRow {
    pub children: Vec<Box<dyn PortableElement>>,
}

/// Cross-platform horizontal stack. On web → `<div style="display:flex;flex-direction:row">`.
/// On Android → `Row` composable.
pub fn row(children: Vec<Box<dyn PortableElement>>) -> Box<PortableRow> {
    Box::new(PortableRow { children })
}

impl Brick for PortableRow {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("div");
        Renderer::set_attr(&node, "style", "display:flex;flex-direction:row");
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortableRow {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::Row(self.children.iter().map(|c| c.blueprint_node()).collect())
    }
}

// ── PortableScroll — scroll() ─────────────────────────────────────────────────

pub struct PortableScroll {
    pub children: Vec<Box<dyn PortableElement>>,
}

/// Cross-platform scroll container. On web → `<div style="overflow:auto">`.
/// On Android → `LazyColumn` / `ScrollView` composable.
pub fn scroll(children: Vec<Box<dyn PortableElement>>) -> Box<PortableScroll> {
    Box::new(PortableScroll { children })
}

impl Brick for PortableScroll {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("div");
        Renderer::set_attr(&node, "style", "overflow:auto");
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortableScroll {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::Scroll(self.children.iter().map(|c| c.blueprint_node()).collect())
    }
}

// ── PortableToggle — toggle() ─────────────────────────────────────────────────

pub struct PortableToggle {
    pub label: String,
    pub checked_signal_id: Option<u32>,
    pub change_action: Option<String>,
}

impl PortableToggle {
    /// Two-way bind: registers `signal` to the drain queue and links its ID
    /// so the native renderer can pre-fill the toggle with the current state.
    pub fn bind(mut self: Box<Self>, signal: &Signal<bool>) -> Box<Self> {
        let id = next_signal_id();
        register_signal_subscription(signal, id);
        self.checked_signal_id = Some(id);
        self
    }

    pub fn on_change(mut self: Box<Self>, action: &BrickAction) -> Box<Self> {
        self.change_action = Some(action.name.to_string());
        self
    }
}

/// Cross-platform boolean toggle. On web → `<input type="checkbox">`. On Android → `Switch`.
pub fn toggle(label: &str) -> Box<PortableToggle> {
    Box::new(PortableToggle { label: label.to_string(), checked_signal_id: None, change_action: None })
}

impl Brick for PortableToggle {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let lbl = Renderer::element("label");
        let input = Renderer::element("input");
        Renderer::set_attr(&input, "type", "checkbox");
        if let Some(ref name) = self.change_action {
            Renderer::set_attr(&input, "data-brick-action", name);
        }
        Renderer::append(&lbl, &input);
        Renderer::append(&lbl, &Renderer::text(&self.label));
        Renderer::append(parent, &lbl);
    }
}

impl PortableView for PortableToggle {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::Toggle {
            label: self.label.clone(),
            checked_signal_id: self.checked_signal_id,
            change_action: self.change_action.clone(),
        }
    }
}

// ── PortableIntentButton — intent_button() ────────────────────────────────────

pub struct PortableIntentButton {
    pub label: String,
    pub intent_type: IntentType,
}

/// Cross-platform external-intent button.
/// On Android → button that launches the corresponding Android Intent.
/// On iOS → button that launches the equivalent UIKit/SwiftUI mechanism.
/// On web → button using the Web Share API, window.open, input[capture], etc.
pub fn intent_button(label: &str, intent_type: IntentType) -> Box<PortableIntentButton> {
    Box::new(PortableIntentButton { label: label.to_string(), intent_type })
}

impl Brick for PortableIntentButton {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = match &self.intent_type {
            IntentType::OpenUrl { url } => {
                let a = Renderer::element("a");
                Renderer::set_attr(&a, "href", url);
                Renderer::set_attr(&a, "target", "_blank");
                Renderer::set_attr(&a, "rel", "noopener");
                a
            }
            IntentType::LaunchCamera { .. } | IntentType::PickImage { .. } => {
                let label_el = Renderer::element("label");
                let input = Renderer::element("input");
                Renderer::set_attr(&input, "type", "file");
                Renderer::set_attr(&input, "accept", "image/*");
                if matches!(&self.intent_type, IntentType::LaunchCamera { .. }) {
                    Renderer::set_attr(&input, "capture", "environment");
                }
                Renderer::append(&label_el, &input);
                label_el
            }
            _ => Renderer::element("button"),
        };
        Renderer::append(&node, &Renderer::text(&self.label));
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortableIntentButton {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::ExternalIntent { label: self.label.clone(), intent_type: self.intent_type.clone() }
    }
}

// ── PortablePermissionRequest ─────────────────────────────────────────────────

pub struct PortablePermissionRequest {
    pub label: String,
    pub permission: String,
    pub result_action: String,
}

/// Cross-platform permission request button.
/// On Android → button that opens the system permission dialog via
/// `ActivityResultContracts.RequestPermission`; result dispatched via
/// `BrickJni.dispatchBool(result_action, isGranted)`.
/// On web → button using the Permissions API.
pub fn permission_request(label: &str, permission: &str, result_action: &str) -> Box<PortablePermissionRequest> {
    Box::new(PortablePermissionRequest {
        label: label.to_string(),
        permission: permission.to_string(),
        result_action: result_action.to_string(),
    })
}

impl Brick for PortablePermissionRequest {
    fn render_into(&self, parent: &BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("button");
        Renderer::set_attr(&node, "data-brick-permission", &self.permission);
        Renderer::set_attr(&node, "data-brick-action", &self.result_action);
        Renderer::append(&node, &Renderer::text(&self.label));
        Renderer::append(parent, &node);
    }
}

impl PortableView for PortablePermissionRequest {
    fn blueprint_node(&self) -> BlueprintNode {
        BlueprintNode::PermissionRequest {
            label: self.label.clone(),
            permission: self.permission.clone(),
            result_action: self.result_action.clone(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::Signal;
    use crate::view_components::render_to_html;

    fn rh(c: &dyn Brick) -> String {
        render_to_html(c)
    }

    // ── Web rendering ─────────────────────────────────────────────────────────

    #[test]
    fn p_static_renders_p_tag() {
        assert_eq!(rh(&*p("hello")), "<p>hello</p>");
    }

    #[test]
    fn p_reactive_renders_initial_value() {
        let sig = Signal::new(42i32);
        assert_eq!(rh(&*p(&sig)), "<p>42</p>");
    }

    #[test]
    fn button_renders_label() {
        assert_eq!(rh(&*button("Click")), r#"<button class="brick-primary">Click</button>"#);
    }

    #[test]
    fn button_with_action_sets_data_attr() {
        let action = BrickAction { name: "increment", event: "click" };
        let html = rh(&*button("Click").trigger(&action));
        assert!(html.contains(r#"data-brick-action="increment""#));
        assert!(html.contains("Click"));
    }

    #[test]
    fn input_renders_void() {
        assert_eq!(rh(&*input()), "<input>");
    }

    #[test]
    fn input_with_placeholder() {
        let html = rh(&*input().placeholder("Enter text"));
        assert!(html.contains(r#"placeholder="Enter text""#));
    }

    #[test]
    fn column_renders_children_in_flex_column() {
        let html = rh(&*column(vec![into_portable(p("a")), into_portable(p("b"))]));
        assert!(html.contains("flex-direction:column"));
        assert!(html.contains("<p>a</p>"));
        assert!(html.contains("<p>b</p>"));
    }

    #[test]
    fn row_renders_children_in_flex_row() {
        let html = rh(&*row(vec![into_portable(button("X"))]));
        assert!(html.contains("flex-direction:row"));
        assert!(html.contains(r#"class="brick-primary""#));
        assert!(html.contains("X"));
    }

    #[test]
    fn scroll_renders_overflow_auto() {
        let html = rh(&*scroll(vec![into_portable(p("content"))]));
        assert!(html.contains("overflow:auto"));
        assert!(html.contains("<p>content</p>"));
    }

    #[test]
    fn column_macro_works() {
        let html = rh(&*column![p("x"), p("y")]);
        assert!(html.contains("<p>x</p>"));
        assert!(html.contains("<p>y</p>"));
    }

    // ── Blueprint generation ──────────────────────────────────────────────────

    #[test]
    fn blueprint_text_static() {
        match p("hi").blueprint_node() {
            BlueprintNode::Text { content: PortableContent::Static(s), .. } => assert_eq!(s, "hi"),
            _ => panic!("expected static text node"),
        }
    }

    #[test]
    fn blueprint_text_reactive_captures_initial() {
        let sig = Signal::new(7i32);
        match p(&sig).blueprint_node() {
            BlueprintNode::Text { content: PortableContent::Reactive { initial, .. }, .. } => {
                assert_eq!(initial, "7")
            }
            _ => panic!("expected reactive text node"),
        }
    }

    #[test]
    fn blueprint_button_no_action() {
        match button("Go").blueprint_node() {
            BlueprintNode::Button { label, action: None, int_action: None, .. } => assert_eq!(label, "Go"),
            _ => panic!("expected button without action"),
        }
    }

    #[test]
    fn blueprint_button_with_action() {
        let action = BrickAction { name: "submit", event: "click" };
        match button("Go").trigger(&action).blueprint_node() {
            BlueprintNode::Button { action: Some(a), .. } => assert_eq!(a, "submit"),
            _ => panic!("expected button with action"),
        }
    }

    #[test]
    fn blueprint_button_trigger_int() {
        let action = BrickAction { name: "remove", event: "click" };
        match button("✕").trigger_int(&action, 2).blueprint_node() {
            BlueprintNode::Button { int_action: Some(a), int_payload: Some(i), .. } => {
                assert_eq!(a, "remove");
                assert_eq!(i, 2);
            }
            _ => panic!("expected button with int dispatch"),
        }
    }

    #[test]
    fn blueprint_toggle_binds_signal() {
        let sig = Signal::new(true);
        match toggle("Dark mode").bind(&sig).blueprint_node() {
            BlueprintNode::Toggle { label, checked_signal_id: Some(_), .. } => {
                assert_eq!(label, "Dark mode");
            }
            _ => panic!("expected toggle with signal"),
        }
    }

    #[test]
    fn blueprint_column_collects_children() {
        match column(vec![into_portable(p("a")), into_portable(p("b"))]).blueprint_node() {
            BlueprintNode::Column(children) => assert_eq!(children.len(), 2),
            _ => panic!("expected column"),
        }
    }

    #[test]
    fn blueprint_scroll_collects_children() {
        match scroll(vec![into_portable(p("x"))]).blueprint_node() {
            BlueprintNode::Scroll(children) => assert_eq!(children.len(), 1),
            _ => panic!("expected scroll"),
        }
    }

    #[test]
    fn blueprint_reactive_assigns_unique_signal_ids() {
        let sig = Signal::new(0i32);
        let id1 = match p(&sig).blueprint_node() {
            BlueprintNode::Text { content: PortableContent::Reactive { signal_id, .. }, .. } => signal_id,
            _ => panic!(),
        };
        let id2 = match p(&sig).blueprint_node() {
            BlueprintNode::Text { content: PortableContent::Reactive { signal_id, .. }, .. } => signal_id,
            _ => panic!(),
        };
        assert_ne!(id1, id2, "each call registers a new subscription with a fresh ID");
    }

    #[test]
    fn blueprint_input_placeholder_and_action() {
        let action = BrickAction { name: "update", event: "input" };
        match input().placeholder("Search").on_change(&action).blueprint_node() {
            BlueprintNode::Input { placeholder, change_action: Some(a), .. } => {
                assert_eq!(placeholder, "Search");
                assert_eq!(a, "update");
            }
            _ => panic!("expected input with placeholder and action"),
        }
    }

    // ── Phase 13 Q1+Q2: macro-generated PortableView + action constants ───────

    mod macro_tests {
        use super::*;
        use crate::view_components::{controller, model, view, IntoComponent, ViewComposite};
        use crate::state_mgmt::cascade;

        #[model]
        struct PCounter {
            #[default(0)]
            val: i32,
        }

        #[controller]
        impl PCounter {
            fn up(&mut self) {
                self.val.update(|n| *n += 1);
            }
            fn down(&mut self) {
                self.val.update(|n| *n -= 1);
            }
        }

        #[view(PCounter)]
        fn pcounter_view() {
            portable! {
                crate::column![
                    crate::portable::p(&self.val),
                    crate::portable::button("+").trigger(&PCounter::UP),
                    crate::portable::button("\u{2212}").trigger(&PCounter::DOWN),
                ]
            }
            children! {}
        }

        #[test]
        fn controller_generates_action_constants() {
            assert_eq!(PCounter::UP.name, "up");
            assert_eq!(PCounter::UP.event, "click");
            assert_eq!(PCounter::DOWN.name, "down");
            assert_eq!(PCounter::DOWN.event, "click");
        }

        #[test]
        fn view_generates_portable_view_impl() {
            let counter = PCounter { ..cascade() };
            match counter.blueprint_node() {
                BlueprintNode::Column(children) => assert_eq!(children.len(), 3),
                _ => panic!("expected Column at root"),
            }
        }

        #[test]
        fn portable_view_text_reflects_initial_signal_value() {
            let counter = PCounter { ..cascade() };
            match counter.blueprint_node() {
                BlueprintNode::Column(children) => match &children[0] {
                    BlueprintNode::Text { content: c, .. } => assert_eq!(c.text(), "0"),
                    _ => panic!("expected Text as first child"),
                },
                _ => panic!("expected Column"),
            }
        }

        #[test]
        fn portable_view_buttons_carry_action_names() {
            let counter = PCounter { ..cascade() };
            match counter.blueprint_node() {
                BlueprintNode::Column(children) => {
                    match &children[1] {
                        BlueprintNode::Button { action: Some(a), .. } => assert_eq!(a, "up"),
                        _ => panic!("expected up button"),
                    }
                    match &children[2] {
                        BlueprintNode::Button { action: Some(a), .. } => assert_eq!(a, "down"),
                        _ => panic!("expected down button"),
                    }
                    // new fields default to None
                }
                _ => panic!("expected Column"),
            }
        }
    }

    // ── SignalEncoding wire format ─────────────────────────────────────────────

    #[test]
    fn signal_encoding_i32_tag_and_bytes() {
        let mut buf = Vec::new();
        42i32.write_bytes(&mut buf);
        assert_eq!(i32::WIRE_TAG, 0x05);
        assert_eq!(buf, 42i32.to_le_bytes().to_vec());
    }

    #[test]
    fn signal_encoding_bool_tag_and_bytes() {
        let mut buf = Vec::new();
        true.write_bytes(&mut buf);
        assert_eq!(bool::WIRE_TAG, 0x0D);
        assert_eq!(buf, vec![0x01]);
    }

    #[test]
    fn signal_encoding_string_len_prefix_and_bytes() {
        let s = "hi".to_string();
        let mut buf = Vec::new();
        s.write_bytes(&mut buf);
        assert_eq!(String::WIRE_TAG, 0x0E);
        // 2-byte LE length prefix then UTF-8 bytes
        assert_eq!(buf, vec![0x02, 0x00, b'h', b'i']);
    }

    #[test]
    fn signal_encoding_isize_always_8_bytes() {
        let mut buf = Vec::new();
        (1isize).write_bytes(&mut buf);
        assert_eq!(buf.len(), 8);
        assert_eq!(isize::WIRE_TAG, 0x09);
    }

    #[test]
    fn signal_encoding_usize_always_8_bytes() {
        let mut buf = Vec::new();
        (1usize).write_bytes(&mut buf);
        assert_eq!(buf.len(), 8);
        assert_eq!(usize::WIRE_TAG, 0x0A);
    }

    #[test]
    fn signal_encoding_f32_tag_and_bytes() {
        let mut buf = Vec::new();
        (1.5f32).write_bytes(&mut buf);
        assert_eq!(f32::WIRE_TAG, 0x0B);
        assert_eq!(buf, 1.5f32.to_le_bytes().to_vec());
    }

    // ── Drain queue (Android-only path) ───────────────────────────────────────

    #[cfg(brick_android)]
    #[test]
    fn signal_subscription_pushes_typed_bytes_to_drain_queue() {
        let sig = Signal::new(10i32);
        let id = next_signal_id();
        register_signal_subscription(&sig, id);
        sig.set(42);
        // Expected: [WIRE_TAG=0x05][42i32 LE = 0x2A, 0x00, 0x00, 0x00]
        let expected: Vec<u8> = {
            let mut v = vec![i32::WIRE_TAG];
            42i32.write_bytes(&mut v);
            v
        };
        SIGNAL_QUEUE.with(|q| {
            let queue = q.borrow();
            assert!(
                queue.iter().any(|(qid, bytes)| *qid == id && bytes == &expected),
                "drain queue should contain typed binary encoding of the updated value"
            );
        });
    }
}
