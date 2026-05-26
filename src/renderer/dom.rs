use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use super::{BrickEvent, BrickRenderer};

// ── DomNode ───────────────────────────────────────────────────────────────────

/// Thin newtype over `web_sys::Node`. `Clone` is cheap (JS reference copy).
/// The newtype gives us a place to add hydration IDs or other metadata later
/// without touching all the call sites.
#[derive(Clone)]
pub struct DomNode(pub(crate) web_sys::Node);

impl DomNode {
    /// Downcast to `web_sys::Element` for attribute operations.
    fn as_element(&self) -> Option<web_sys::Element> {
        self.0.dyn_ref::<web_sys::Element>().cloned()
    }

    fn as_html_element(&self) -> Option<web_sys::HtmlElement> {
        self.0.dyn_ref::<web_sys::HtmlElement>().cloned()
    }
}

// ── DomRenderer ───────────────────────────────────────────────────────────────

pub struct DomRenderer;

impl BrickRenderer for DomRenderer {
    type Node = DomNode;

    fn element(tag: &str) -> DomNode {
        let doc = web_sys::window().unwrap().document().unwrap();
        DomNode(doc.create_element(tag).unwrap().into())
    }

    fn text(content: &str) -> DomNode {
        let doc = web_sys::window().unwrap().document().unwrap();
        DomNode(doc.create_text_node(content).into())
    }

    fn fragment(children: Vec<DomNode>) -> DomNode {
        let doc = web_sys::window().unwrap().document().unwrap();
        let frag = doc.create_document_fragment();
        for child in children {
            frag.append_child(&child.0).ok();
        }
        DomNode(frag.into())
    }

    fn set_attr(node: &DomNode, key: &str, value: &str) {
        if let Some(el) = node.as_element() {
            el.set_attribute(key, value).ok();
        }
    }

    fn remove_attr(node: &DomNode, key: &str) {
        if let Some(el) = node.as_element() {
            el.remove_attribute(key).ok();
        }
    }

    fn set_text(node: &DomNode, content: &str) {
        node.0.set_text_content(Some(content));
    }

    fn append(parent: &DomNode, child: &DomNode) {
        parent.0.append_child(&child.0).ok();
    }

    fn remove_child(parent: &DomNode, child: &DomNode) {
        parent.0.remove_child(&child.0).ok();
    }

    fn insert_before(parent: &DomNode, new_node: &DomNode, ref_node: &DomNode) {
        parent.0.insert_before(&new_node.0, Some(&ref_node.0)).ok();
    }

    fn on_event(node: &DomNode, event: &str, handler: Box<dyn Fn(BrickEvent)>) {
        let event_name = event.to_string();
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let brick_event = match event_name.as_str() {
                "click" => {
                    use wasm_bindgen::JsCast as _;
                    if let Some(me) = e.dyn_ref::<web_sys::MouseEvent>() {
                        BrickEvent::ClickAt(me.client_x() as f64, me.client_y() as f64)
                    } else {
                        BrickEvent::Click
                    }
                }
                "input" | "change" => {
                    if let Some(input) = e
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                    {
                        if input.type_() == "checkbox" {
                            BrickEvent::InputBool(input.checked())
                        } else if input.type_() == "number" {
                            let n = input.value_as_number();
                            if n.is_nan() {
                                return; // drop NaN input events silently
                            }
                            BrickEvent::InputNumber(n)
                        } else {
                            BrickEvent::InputString(input.value())
                        }
                    } else if let Some(ta) = e
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                    {
                        BrickEvent::InputString(ta.value())
                    } else {
                        BrickEvent::Click
                    }
                }
                _ => BrickEvent::Click,
            };
            handler(brick_event);
        }) as Box<dyn FnMut(web_sys::Event)>);

        if let Some(el) = node.as_element() {
            el.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
                .ok();
        }
        closure.forget();
    }

    fn child_count(node: &DomNode) -> usize {
        node.0.child_nodes().length() as usize
    }

    fn nth_child(node: &DomNode, n: usize) -> Option<DomNode> {
        node.0.child_nodes().item(n as u32).map(DomNode)
    }

    fn add_class(node: &DomNode, class: &str) {
        if let Some(el) = node.as_element() {
            el.class_list().add_1(class).ok();
        }
    }

    fn remove_node(node: &DomNode) {
        if let Some(parent) = node.0.parent_node() {
            parent.remove_child(&node.0).ok();
        }
    }

    fn mount_root(node: DomNode) {
        let doc = web_sys::window().unwrap().document().unwrap();
        doc.body().unwrap().append_child(&node.0).ok();
    }
}
