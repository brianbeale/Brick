pub mod html;
pub use html::{HtmlNode, HtmlRenderer};

#[cfg(brick_dom)]
pub mod dom;
#[cfg(brick_dom)]
pub use dom::{DomNode, DomRenderer};

/// A UI event carrying any data the renderer extracts from a native event.
#[derive(Clone, Debug)]
pub enum BrickEvent {
    /// A click with no coordinate data (HTML renderer, server-side, or non-mouse triggers).
    Click,
    /// A click with client-relative coordinates — produced by DomRenderer for mouse events.
    ClickAt(f64, f64),
    InputString(String),
    InputBool(bool),
    InputNumber(f64),
}

/// Core renderer abstraction. Implement this for a target platform to make all
/// Brick components compile and run on that platform with zero runtime overhead.
///
/// ZST implementors are recommended — all methods are static dispatch.
pub trait BrickRenderer: 'static {
    type Node: Clone + 'static;

    fn element(tag: &str) -> Self::Node;
    fn text(content: &str) -> Self::Node;
    /// A wrapper-free group of sibling nodes. Serialises as concatenated HTML;
    /// in the DOM, appending this node moves its children into the parent.
    fn fragment(children: Vec<Self::Node>) -> Self::Node;

    fn set_attr(node: &Self::Node, key: &str, value: &str);
    fn remove_attr(node: &Self::Node, key: &str);
    fn set_text(node: &Self::Node, content: &str);

    fn append(parent: &Self::Node, child: &Self::Node);
    fn remove_child(parent: &Self::Node, child: &Self::Node);
    fn insert_before(parent: &Self::Node, new_node: &Self::Node, ref_node: &Self::Node);

    /// Return the number of direct child nodes.
    fn child_count(node: &Self::Node) -> usize;
    /// Return the Nth child node, or `None` if out of range.
    fn nth_child(node: &Self::Node, n: usize) -> Option<Self::Node>;
    /// Add a CSS class to an element node. No-op on non-element nodes.
    fn add_class(node: &Self::Node, class: &str);

    /// Attach a native event listener. The renderer extracts relevant data and
    /// calls `handler` with the appropriate `BrickEvent` variant.
    /// No-op on renderers that do not run in an interactive environment.
    fn on_event(node: &Self::Node, event: &str, handler: Box<dyn Fn(BrickEvent)>);

    /// Remove the node from its parent.
    fn remove_node(node: &Self::Node);

    /// Append the node to the platform's root container (document body for DOM,
    /// a no-op for pure rendering environments).
    fn mount_root(node: Self::Node);
}

// ── cfg-based Renderer alias ─────────────────────────────────────────────────
//
// The `brick` build tool sets one of these cfg flags via RUSTFLAGS or
// .cargo/config.toml [build] rustflags. `cargo test` sets nothing, so the
// default falls through to HtmlRenderer — the pure-Rust tree renderer that
// powers all unit tests and future SSR.

#[cfg(not(brick_dom))]
pub type Renderer = HtmlRenderer;

#[cfg(brick_dom)]
pub type Renderer = DomRenderer;

/// The node type produced by the active renderer. `HtmlNode` in test/SSR builds,
/// `DomNode` in browser builds.
pub type BrickNode = <Renderer as BrickRenderer>::Node;
