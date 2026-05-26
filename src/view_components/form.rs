use super::Brick;
use std::sync::atomic::{AtomicUsize, Ordering};

static FORM_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct BrickForm {
    class: String,
    action_name: Option<&'static str>,
    children: Vec<Box<dyn Brick>>,
}

impl BrickForm {
    pub fn new(action_name: Option<&'static str>, children: Vec<Box<dyn Brick>>) -> Box<Self> {
        let id = FORM_COUNTER.fetch_add(1, Ordering::SeqCst);
        Box::new(BrickForm {
            class: format!("brick-form-{}", id),
            action_name,
            children,
        })
    }
}

impl Brick for BrickForm {
    fn render_into(&self, parent: &crate::renderer::BrickNode) {
        use crate::renderer::{BrickRenderer as _, Renderer};
        let node = Renderer::element("form");
        Renderer::set_attr(&node, "class", &self.class);
        for child in &self.children {
            child.render_into(&node);
        }
        Renderer::append(parent, &node);
    }

    fn attach_listeners(&self) {
        #[cfg(brick_dom)]
        if let Some(action_name) = self.action_name {
            use wasm_bindgen::{JsCast, closure::Closure};
            let class = self.class.clone();
            let action = action_name;
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                let selector = format!(".{}", class);
                if let Ok(Some(form_el)) = doc.query_selector(&selector) {
                    let action_str = action.to_string();
                    let class_str = class.clone();
                    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                        e.prevent_default();
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            let sel =
                                format!(".{} [data-brick-action='{}']", class_str, action_str);
                            if let Ok(Some(btn)) = doc.query_selector(&sel) {
                                if let Ok(html_btn) = btn.dyn_into::<web_sys::HtmlElement>() {
                                    html_btn.click();
                                }
                            }
                        }
                    })
                        as Box<dyn FnMut(web_sys::Event)>);
                    form_el
                        .add_event_listener_with_callback(
                            "submit",
                            closure.as_ref().unchecked_ref(),
                        )
                        .unwrap();
                    closure.forget();
                }
            }
        }
    }

    fn detach(&self) {
        for child in &self.children {
            child.detach();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BrickForm;
    use crate::view_components::{Brick, leafs::p};

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn renders_as_form_element() {
        let f = BrickForm::new(None, vec![p("hello")]);
        let html = rh(&f);
        assert!(html.starts_with("<form"), "should render <form> tag");
        assert!(html.contains("brick-form-"), "should have identity class");
        assert!(html.contains("<p>hello</p>"), "children rendered");
        assert!(html.ends_with("</form>"), "should close form tag");
    }

    #[test]
    fn two_forms_get_unique_classes() {
        let f1 = BrickForm::new(None, vec![]);
        let f2 = BrickForm::new(None, vec![]);
        assert_ne!(rh(&f1), rh(&f2), "each form gets a unique identity class");
    }
}
