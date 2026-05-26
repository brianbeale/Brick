use super::Brick;
use crate::state_mgmt::List;
#[cfg(brick_dom)]
use crate::state_mgmt::{ListChange, ListObserver};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ViewList<T: 'static> {
    subject: Rc<RefCell<List<T>>>,
    render_item: Rc<dyn Fn(&Rc<T>, usize) -> Box<dyn Brick>>,
    list_class: String,
    // Rendered during html() and reused by attach_listeners() so each item's
    // component is constructed exactly once — avoiding duplicate observer IDs.
    initial_components: RefCell<Vec<Box<dyn Brick>>>,
}

fn item_class<T>(item: &Rc<T>) -> String {
    format!("bli{:x}", Rc::as_ptr(item) as usize)
}

impl<T: 'static> ViewList<T> {
    pub fn new<F>(subject: Rc<RefCell<List<T>>>, render_item: F) -> Self
    where
        F: Fn(&Rc<T>, usize) -> Box<dyn Brick> + 'static,
    {
        let list_class = format!(
            "brick-list-{}",
            crate::INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );
        ViewList {
            subject,
            render_item: Rc::new(render_item),
            list_class,
            initial_components: RefCell::new(Vec::new()),
        }
    }
}

impl<T: 'static> Brick for ViewList<T> {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
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

            let obs = DomListObserver {
                list_class: self.list_class.clone(),
                render_item: Rc::clone(&self.render_item),
                subject: Rc::clone(&self.subject),
            };
            self.subject.borrow_mut().add_observer(Box::new(obs));
        }
    }
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

#[cfg(brick_dom)]
struct DomListObserver<T: 'static> {
    list_class: String,
    render_item: Rc<dyn Fn(&Rc<T>, usize) -> Box<dyn Brick>>,
    subject: Rc<RefCell<List<T>>>,
}

#[cfg(brick_dom)]
impl<T: 'static> ListObserver<T> for DomListObserver<T> {
    fn notify_list(&mut self, change: &ListChange<T>) {
        use wasm_bindgen::JsCast;
        let doc = web_sys::window().unwrap().document().unwrap();

        match change {
            ListChange::Push(item) => {
                let ic = item_class(item);
                let idx = self.subject.borrow().items().len().saturating_sub(1);
                let item_component = (self.render_item)(item, idx);
                let item_html = crate::view_components::render_to_html(&*item_component);
                let container_html = format!(r#"<div class="{}">{}</div>"#, ic, item_html);

                let selector = format!(".{}", self.list_class);
                if let Ok(Some(list_el)) = doc.query_selector(&selector) {
                    list_el
                        .insert_adjacent_html("beforeend", &container_html)
                        .unwrap();
                }
                attach_remove_buttons(&ic, &self.subject, item);
            }
            ListChange::Remove(target) => {
                let ic = item_class(target);
                let selector = format!(".{}", ic);
                if let Ok(Some(el)) = doc.query_selector(&selector) {
                    el.dyn_into::<web_sys::HtmlElement>()
                        .ok()
                        .map(|e| e.remove());
                }
            }
            ListChange::Clear => {
                let selector = format!(".{}", self.list_class);
                if let Ok(Some(list_el)) = doc.query_selector(&selector) {
                    list_el
                        .dyn_into::<web_sys::HtmlElement>()
                        .ok()
                        .map(|e| e.set_inner_html(""));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::{
        Brick,
        leafs::{button, p},
    };

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    fn make_list() -> ViewList<String> {
        let subject = Rc::new(RefCell::new(List::new()));
        ViewList::new(
            subject,
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
        )
    }

    #[test]
    fn empty_list_renders_container() {
        let list = make_list();
        let html = rh(&list);
        assert!(html.starts_with(r#"<div class="brick-list-"#));
        assert!(html.ends_with("</div>"));
    }

    #[test]
    fn list_renders_pushed_items() {
        let subject = Rc::new(RefCell::new(List::<String>::new()));
        subject.borrow_mut().push("hello".to_string());
        subject.borrow_mut().push("world".to_string());

        let list = ViewList::new(
            Rc::clone(&subject),
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
        );

        let html = rh(&list);
        assert!(html.contains("<p>hello</p>"));
        assert!(html.contains("<p>world</p>"));
    }

    #[test]
    fn item_containers_have_pointer_class() {
        let subject = Rc::new(RefCell::new(List::<String>::new()));
        let rc = subject.borrow_mut().push("item".to_string());

        let list = ViewList::new(
            Rc::clone(&subject),
            |item: &Rc<String>, _idx: usize| -> Box<dyn Brick> { p(item.as_str()) },
        );

        let html = rh(&list);
        let expected_class = format!("bli{:x}", Rc::as_ptr(&rc) as usize);
        assert!(
            html.contains(&expected_class),
            "expected class {} in {}",
            expected_class,
            html
        );
    }

    #[test]
    fn remove_button_renders_attribute() {
        let html = rh(&*button("Delete").remove());
        assert!(html.contains(r#"data-brick-remove="true""#));
    }
}
