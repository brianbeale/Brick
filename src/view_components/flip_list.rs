use super::Brick;
use crate::state_mgmt::List;
#[cfg(brick_dom)]
use crate::state_mgmt::{ListChange, ListObserver};
use crate::theme::Ms;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FlipList<T: 'static> {
    subject: Rc<RefCell<List<T>>>,
    render_item: Rc<dyn Fn(&Rc<T>, usize) -> Box<dyn Brick>>,
    list_class: String,
    duration: Ms,
    initial_components: RefCell<Vec<Box<dyn Brick>>>,
}

fn item_class<T>(item: &Rc<T>) -> String {
    format!("bli{:x}", Rc::as_ptr(item) as usize)
}

impl<T: 'static> FlipList<T> {
    pub fn new<F>(subject: Rc<RefCell<List<T>>>, render_item: F) -> Self
    where
        F: Fn(&Rc<T>, usize) -> Box<dyn Brick> + 'static,
    {
        Self::with_duration(subject, render_item, Ms(300.0))
    }

    pub fn with_duration<F>(subject: Rc<RefCell<List<T>>>, render_item: F, duration: Ms) -> Self
    where
        F: Fn(&Rc<T>, usize) -> Box<dyn Brick> + 'static,
    {
        let list_class = format!(
            "brick-flip-{}",
            crate::INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );
        FlipList {
            subject,
            render_item: Rc::new(render_item),
            list_class,
            duration,
            initial_components: RefCell::new(Vec::new()),
        }
    }
}

impl<T: 'static> Brick for FlipList<T> {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        crate::view_components::inject_global_style(&format!(
            "[data-flip] {{ transition: transform {}ms ease; }}",
            self.duration.0 as u32,
        ));
        let borrowed = self.subject.borrow();
        let mut cache = self.initial_components.borrow_mut();
        cache.clear();
        let list_node = Renderer::element("div");
        Renderer::set_attr(&list_node, "class", &self.list_class);
        for (idx, item) in borrowed.items().iter().enumerate() {
            let ic = item_class(item);
            let comp = (self.render_item)(item, idx);
            let item_node = Renderer::element("div");
            Renderer::set_attr(&item_node, "class", &ic);
            Renderer::set_attr(&item_node, "data-flip", "true");
            comp.render_into(&item_node);
            cache.push(comp);
            Renderer::append(&list_node, &item_node);
        }
        Renderer::append(parent, &list_node);
    }

    fn attach_listeners(&self) {
        #[cfg(brick_dom)]
        {
            let borrowed = self.subject.borrow();
            let cache = self.initial_components.borrow();
            for (item, comp) in borrowed.items().iter().zip(cache.iter()) {
                let ic = item_class(item);
                let _ = comp;
                attach_remove_buttons(&ic, &self.subject, item);
            }
            drop(cache);
            drop(borrowed);

            let obs = DomFlipObserver {
                list_class: self.list_class.clone(),
                render_item: Rc::clone(&self.render_item),
                subject: Rc::clone(&self.subject),
            };
            self.subject.borrow_mut().add_observer(Box::new(obs));
        }
    }
}

// ── DOM helpers (non-test only) ───────────────────────────────────────────────

#[cfg(brick_dom)]
fn snapshot_rects(
    doc: &web_sys::Document,
    container_sel: &str,
) -> std::collections::HashMap<String, web_sys::DomRect> {
    use wasm_bindgen::JsCast;
    let mut map = std::collections::HashMap::new();
    let selector = format!("{} > [data-flip]", container_sel);
    if let Ok(nodes) = doc.query_selector_all(&selector) {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i) {
                if let Ok(el) = node.dyn_into::<web_sys::Element>() {
                    let class = el.class_name();
                    let rect = el.get_bounding_client_rect();
                    map.insert(class, rect);
                }
            }
        }
    }
    map
}

#[cfg(brick_dom)]
fn attach_remove_buttons<T: 'static>(
    container_class: &str,
    subject: &Rc<RefCell<List<T>>>,
    item: &Rc<T>,
) {
    use wasm_bindgen::{JsCast, closure::Closure};
    let doc = web_sys::window().unwrap().document().unwrap();
    let selector = format!(".{} [data-brick-remove]", container_class);
    let nodes = match doc.query_selector_all(&selector) {
        Ok(n) => n,
        Err(_) => return,
    };
    for i in 0..nodes.length() {
        if let Some(node) = nodes.item(i) {
            if let Ok(el) = node.dyn_into::<web_sys::Element>() {
                let s = Rc::clone(subject);
                let target = Rc::clone(item);
                let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    s.borrow_mut().remove(&target);
                }) as Box<dyn FnMut(web_sys::Event)>);
                el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }
        }
    }
}

// ── DomFlipObserver ───────────────────────────────────────────────────────────

#[cfg(brick_dom)]
struct DomFlipObserver<T: 'static> {
    list_class: String,
    render_item: Rc<dyn Fn(&Rc<T>, usize) -> Box<dyn Brick>>,
    subject: Rc<RefCell<List<T>>>,
}

#[cfg(brick_dom)]
impl<T: 'static> ListObserver<T> for DomFlipObserver<T> {
    fn notify_list(&mut self, change: &ListChange<T>) {
        use wasm_bindgen::JsCast;
        let doc = web_sys::window().unwrap().document().unwrap();

        match change {
            ListChange::Push(item) => {
                let ic = item_class(item);
                let idx = self.subject.borrow().items().len().saturating_sub(1);
                let item_component = (self.render_item)(item, idx);
                let item_html = crate::view_components::render_to_html(&*item_component);
                let container_html = format!(
                    r#"<div class="{}" data-flip="true">{}</div>"#,
                    ic, item_html
                );
                let selector = format!(".{}", self.list_class);
                if let Ok(Some(list_el)) = doc.query_selector(&selector) {
                    list_el
                        .insert_adjacent_html("beforeend", &container_html)
                        .unwrap();
                }
                attach_remove_buttons(&ic, &self.subject, item);
            }

            ListChange::Remove(target) => {
                let container_sel = format!(".{}", self.list_class);

                // First — snapshot all item positions before removal
                let first = snapshot_rects(&doc, &container_sel);

                // Remove the target element
                let ic = item_class(target);
                if let Ok(Some(el)) = doc.query_selector(&format!(".{}", ic)) {
                    if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                        html_el.remove();
                    }
                }

                // Last — snapshot surviving items (forces reflow)
                let last = snapshot_rects(&doc, &container_sel);

                // Invert then Play for each item that moved
                for (key, last_rect) in &last {
                    if let Some(first_rect) = first.get(key) {
                        let dy = first_rect.top() - last_rect.top();
                        let dx = first_rect.left() - last_rect.left();
                        if dy.abs() < 0.5 && dx.abs() < 0.5 {
                            continue;
                        }

                        if let Ok(Some(el)) = doc.query_selector(&format!(".{}", key)) {
                            if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                                let style = html_el.style();
                                // Invert: move element back to where it was, no transition
                                let _ = style.set_property("transition", "none");
                                let _ = style.set_property(
                                    "transform",
                                    &format!("translate({}px, {}px)", dx, dy),
                                );
                                // Force the browser to commit the inverted position
                                let _ = html_el.get_bounding_client_rect();
                                // Play: remove overrides; CSS transition animates to Last
                                let _ = style.remove_property("transition");
                                let _ = style.remove_property("transform");
                            }
                        }
                    }
                }
            }

            ListChange::Clear => {
                let selector = format!(".{}", self.list_class);
                if let Ok(Some(list_el)) = doc.query_selector(&selector) {
                    if let Ok(html_el) = list_el.dyn_into::<web_sys::HtmlElement>() {
                        html_el.set_inner_html("");
                    }
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::{Brick, leafs::p};

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    fn make_flip_list() -> FlipList<String> {
        let subject = Rc::new(RefCell::new(List::new()));
        FlipList::new(
            subject,
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
        )
    }

    #[test]
    fn empty_list_renders_container() {
        let list = make_flip_list();
        let html = rh(&list);
        assert!(html.starts_with(r#"<div class="brick-flip-"#));
        assert!(html.ends_with("</div>"));
    }

    #[test]
    fn list_renders_pushed_items() {
        let subject = Rc::new(RefCell::new(List::<String>::new()));
        subject.borrow_mut().push("hello".to_string());
        subject.borrow_mut().push("world".to_string());

        let list = FlipList::new(
            Rc::clone(&subject),
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
        );

        let html = rh(&list);
        assert!(html.contains("<p>hello</p>"));
        assert!(html.contains("<p>world</p>"));
    }

    #[test]
    fn item_wrappers_have_data_flip_attribute() {
        let subject = Rc::new(RefCell::new(List::<String>::new()));
        subject.borrow_mut().push("item".to_string());

        let list = FlipList::new(
            Rc::clone(&subject),
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
        );

        let html = rh(&list);
        assert!(html.contains(r#"data-flip="true""#));
    }

    #[test]
    fn item_wrappers_have_pointer_class() {
        let subject = Rc::new(RefCell::new(List::<String>::new()));
        let rc = subject.borrow_mut().push("item".to_string());

        let list = FlipList::new(
            Rc::clone(&subject),
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
        );

        let html = rh(&list);
        let expected = format!("bli{:x}", Rc::as_ptr(&rc) as usize);
        assert!(
            html.contains(&expected),
            "expected class {} in {}",
            expected,
            html
        );
    }

    #[test]
    fn with_duration_stores_custom_ms() {
        let subject = Rc::new(RefCell::new(List::<String>::new()));
        let list = FlipList::with_duration(
            subject,
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
            Ms(500.0),
        );
        assert_eq!(list.duration.0, 500.0);
    }
}
