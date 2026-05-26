use std::cell::RefCell;
use std::rc::Rc;

use super::{BrickEvent, BrickRenderer};

// ── HtmlNode ─────────────────────────────────────────────────────────────────

enum HtmlNodeInner {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<HtmlNode>,
    },
    Text(String),
    /// Pre-rendered HTML injected without escaping. Used as a migration bridge
    /// so that `ViewComposite` can delegate children to the legacy `html()` path
    /// until all child types have native `Brick<R>` implementations.
    RawHtml(String),
    /// Multiple sibling nodes with no wrapper element. Serialises as the
    /// concatenation of each child's HTML.
    Fragment(Vec<HtmlNode>),
}

/// A lightweight tree node used by `HtmlRenderer`. Cheap to clone — all clones
/// share the same `Rc<RefCell<…>>` and see mutations through it, making reactive
/// observer closures straightforward.
#[derive(Clone)]
pub struct HtmlNode(Rc<RefCell<HtmlNodeInner>>);

impl HtmlNode {
    /// Insert pre-rendered HTML as a child without escaping. Migration bridge
    /// only — prefer building proper `HtmlNode` trees wherever possible.
    pub fn raw(html: impl Into<String>) -> HtmlNode {
        HtmlNode(Rc::new(RefCell::new(HtmlNodeInner::RawHtml(html.into()))))
    }

    /// A fragment of sibling nodes with no wrapper element.
    pub fn fragment(children: Vec<HtmlNode>) -> HtmlNode {
        HtmlNode(Rc::new(RefCell::new(HtmlNodeInner::Fragment(children))))
    }

    /// Serialize only the children of this element node to an HTML string.
    /// Equivalent to `innerHTML`. For non-element nodes falls back to `to_html_string()`.
    pub fn inner_html_string(&self) -> String {
        match &*self.0.borrow() {
            HtmlNodeInner::Element { children, .. } => {
                children.iter().map(HtmlNode::to_html_string).collect()
            }
            HtmlNodeInner::Fragment(children) => {
                children.iter().map(HtmlNode::to_html_string).collect()
            }
            HtmlNodeInner::Text(t) => html_escape(t),
            HtmlNodeInner::RawHtml(s) => s.clone(),
        }
    }

    /// Serialize the node tree to an HTML string. Attributes are emitted in
    /// insertion order; void elements are self-closing.
    pub fn to_html_string(&self) -> String {
        match &*self.0.borrow() {
            HtmlNodeInner::Text(t) => html_escape(t),
            HtmlNodeInner::RawHtml(s) => s.clone(),
            HtmlNodeInner::Fragment(children) => {
                children.iter().map(HtmlNode::to_html_string).collect()
            }
            HtmlNodeInner::Element {
                tag,
                attrs,
                children,
            } => {
                const VOID: &[&str] = &[
                    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                    "param", "source", "track", "wbr",
                ];
                let attr_str: String = attrs
                    .iter()
                    .map(|(k, v)| {
                        if v.is_empty() {
                            format!(" {k}")
                        } else {
                            format!(" {k}=\"{v}\"")
                        }
                    })
                    .collect();
                if VOID.contains(&tag.as_str()) {
                    format!("<{tag}{attr_str}>")
                } else {
                    let inner: String = children.iter().map(HtmlNode::to_html_string).collect();
                    format!("<{tag}{attr_str}>{inner}</{tag}>")
                }
            }
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ── HtmlRenderer ─────────────────────────────────────────────────────────────

pub struct HtmlRenderer;

impl BrickRenderer for HtmlRenderer {
    type Node = HtmlNode;

    fn element(tag: &str) -> HtmlNode {
        HtmlNode(Rc::new(RefCell::new(HtmlNodeInner::Element {
            tag: tag.to_string(),
            attrs: Vec::new(),
            children: Vec::new(),
        })))
    }

    fn text(content: &str) -> HtmlNode {
        HtmlNode(Rc::new(RefCell::new(HtmlNodeInner::Text(
            content.to_string(),
        ))))
    }

    fn fragment(children: Vec<HtmlNode>) -> HtmlNode {
        HtmlNode::fragment(children)
    }

    fn set_attr(node: &HtmlNode, key: &str, value: &str) {
        if let HtmlNodeInner::Element { attrs, .. } = &mut *node.0.borrow_mut() {
            if let Some(pair) = attrs.iter_mut().find(|(k, _)| k == key) {
                pair.1 = value.to_string();
            } else {
                attrs.push((key.to_string(), value.to_string()));
            }
        }
    }

    fn remove_attr(node: &HtmlNode, key: &str) {
        if let HtmlNodeInner::Element { attrs, .. } = &mut *node.0.borrow_mut() {
            attrs.retain(|(k, _)| k != key);
        }
    }

    fn set_text(node: &HtmlNode, content: &str) {
        match &mut *node.0.borrow_mut() {
            HtmlNodeInner::Text(t) => *t = content.to_string(),
            HtmlNodeInner::Element { children, .. } => {
                *children = vec![HtmlNode(Rc::new(RefCell::new(HtmlNodeInner::Text(
                    content.to_string(),
                ))))];
            }
            HtmlNodeInner::RawHtml(s) => *s = content.to_string(),
            HtmlNodeInner::Fragment(_) => {}
        }
    }

    fn append(parent: &HtmlNode, child: &HtmlNode) {
        if let HtmlNodeInner::Element { children, .. } = &mut *parent.0.borrow_mut() {
            children.push(child.clone());
        }
    }

    fn remove_child(parent: &HtmlNode, child: &HtmlNode) {
        if let HtmlNodeInner::Element { children, .. } = &mut *parent.0.borrow_mut() {
            let ptr = Rc::as_ptr(&child.0);
            children.retain(|c| Rc::as_ptr(&c.0) != ptr);
        }
    }

    fn insert_before(parent: &HtmlNode, new_node: &HtmlNode, ref_node: &HtmlNode) {
        if let HtmlNodeInner::Element { children, .. } = &mut *parent.0.borrow_mut() {
            let ptr = Rc::as_ptr(&ref_node.0);
            let pos = children
                .iter()
                .position(|c| Rc::as_ptr(&c.0) == ptr)
                .unwrap_or(children.len());
            children.insert(pos, new_node.clone());
        }
    }

    fn child_count(node: &HtmlNode) -> usize {
        match &*node.0.borrow() {
            HtmlNodeInner::Element { children, .. } => children.len(),
            HtmlNodeInner::Fragment(children) => children.len(),
            _ => 0,
        }
    }

    fn nth_child(node: &HtmlNode, n: usize) -> Option<HtmlNode> {
        match &*node.0.borrow() {
            HtmlNodeInner::Element { children, .. } => children.get(n).cloned(),
            HtmlNodeInner::Fragment(children) => children.get(n).cloned(),
            _ => None,
        }
    }

    fn add_class(node: &HtmlNode, class: &str) {
        if let HtmlNodeInner::Element { attrs, .. } = &mut *node.0.borrow_mut() {
            if let Some(pair) = attrs.iter_mut().find(|(k, _)| k == "class") {
                if pair.1.is_empty() {
                    pair.1 = class.to_string();
                } else {
                    pair.1.push(' ');
                    pair.1.push_str(class);
                }
            } else {
                attrs.push(("class".to_string(), class.to_string()));
            }
        }
    }

    fn on_event(_node: &HtmlNode, _event: &str, _handler: Box<dyn Fn(BrickEvent)>) {
        // Non-interactive environment — events are not dispatched.
    }

    fn remove_node(node: &HtmlNode) {
        // Without a parent pointer we cannot detach. The parent removes via
        // `remove_child`; this is a no-op here for renderer-agnostic call sites.
        let _ = node;
    }

    fn mount_root(_node: HtmlNode) {
        // Nothing to do — caller holds the root reference.
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn el(tag: &str) -> HtmlNode {
        HtmlRenderer::element(tag)
    }

    fn txt(s: &str) -> HtmlNode {
        HtmlRenderer::text(s)
    }

    #[test]
    fn text_node_serializes() {
        assert_eq!(txt("hello").to_html_string(), "hello");
    }

    #[test]
    fn text_escapes_special_chars() {
        assert_eq!(
            txt("<b>a & b</b>").to_html_string(),
            "&lt;b&gt;a &amp; b&lt;/b&gt;"
        );
    }

    #[test]
    fn empty_element_serializes() {
        assert_eq!(el("div").to_html_string(), "<div></div>");
    }

    #[test]
    fn element_with_text_child() {
        let div = el("div");
        HtmlRenderer::append(&div, &txt("hi"));
        assert_eq!(div.to_html_string(), "<div>hi</div>");
    }

    #[test]
    fn nested_elements() {
        let outer = el("div");
        let inner = el("p");
        HtmlRenderer::append(&inner, &txt("body"));
        HtmlRenderer::append(&outer, &inner);
        assert_eq!(outer.to_html_string(), "<div><p>body</p></div>");
    }

    #[test]
    fn set_attr_adds_and_updates() {
        let div = el("div");
        HtmlRenderer::set_attr(&div, "class", "foo");
        assert_eq!(div.to_html_string(), r#"<div class="foo"></div>"#);
        HtmlRenderer::set_attr(&div, "class", "bar");
        assert_eq!(div.to_html_string(), r#"<div class="bar"></div>"#);
    }

    #[test]
    fn remove_attr() {
        let div = el("div");
        HtmlRenderer::set_attr(&div, "id", "x");
        HtmlRenderer::remove_attr(&div, "id");
        assert_eq!(div.to_html_string(), "<div></div>");
    }

    #[test]
    fn boolean_attr_no_value() {
        let input = el("input");
        HtmlRenderer::set_attr(&input, "disabled", "");
        assert_eq!(input.to_html_string(), "<input disabled>");
    }

    #[test]
    fn void_element_self_closes() {
        let br = el("br");
        assert_eq!(br.to_html_string(), "<br>");
        let input = el("input");
        HtmlRenderer::set_attr(&input, "type", "text");
        assert_eq!(input.to_html_string(), r#"<input type="text">"#);
    }

    #[test]
    fn set_text_replaces_children() {
        let div = el("div");
        HtmlRenderer::append(&div, &el("p"));
        HtmlRenderer::set_text(&div, "replaced");
        assert_eq!(div.to_html_string(), "<div>replaced</div>");
    }

    #[test]
    fn set_text_on_text_node() {
        let t = txt("old");
        HtmlRenderer::set_text(&t, "new");
        assert_eq!(t.to_html_string(), "new");
    }

    #[test]
    fn remove_child() {
        let parent = el("div");
        let child1 = el("span");
        let child2 = el("p");
        HtmlRenderer::append(&parent, &child1);
        HtmlRenderer::append(&parent, &child2);
        HtmlRenderer::remove_child(&parent, &child1);
        assert_eq!(parent.to_html_string(), "<div><p></p></div>");
    }

    #[test]
    fn insert_before() {
        let parent = el("div");
        let first = el("a");
        let last = el("b");
        let middle = el("span");
        HtmlRenderer::append(&parent, &first);
        HtmlRenderer::append(&parent, &last);
        HtmlRenderer::insert_before(&parent, &middle, &last);
        assert_eq!(
            parent.to_html_string(),
            "<div><a></a><span></span><b></b></div>"
        );
    }

    #[test]
    fn clone_shares_state() {
        let a = el("div");
        let b = a.clone();
        HtmlRenderer::set_attr(&a, "id", "shared");
        assert_eq!(b.to_html_string(), r#"<div id="shared"></div>"#);
    }

    #[test]
    fn reactive_set_text_via_clone() {
        let node = txt("initial");
        let handle = node.clone();
        HtmlRenderer::set_text(&handle, "updated");
        assert_eq!(node.to_html_string(), "updated");
    }

    #[test]
    fn empty_fragment_serializes_to_empty_string() {
        assert_eq!(HtmlNode::fragment(vec![]).to_html_string(), "");
    }

    #[test]
    fn fragment_concatenates_children() {
        let f = HtmlNode::fragment(vec![el("span"), txt("hello"), el("br")]);
        assert_eq!(f.to_html_string(), "<span></span>hello<br>");
    }
}
