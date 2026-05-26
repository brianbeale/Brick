use crate::state_mgmt::Signal;
use crate::view_components::*;

pub struct NameEntry {
    pub first: String,
    pub last: String,
    pub visible: Signal<bool>,
    pub sel_class: Signal<String>,
}

impl Clone for NameEntry {
    fn clone(&self) -> Self {
        NameEntry {
            first: self.first.clone(),
            last: self.last.clone(),
            visible: self.visible.clone(),
            sel_class: self.sel_class.clone(),
        }
    }
}

impl NameEntry {
    pub fn new(first: &str, last: &str) -> Self {
        NameEntry {
            first: first.to_string(),
            last: last.to_string(),
            visible: Signal::new(true),
            sel_class: Signal::new(String::new()),
        }
    }
}

#[model]
pub struct Crud {
    #[default(String::new())]
    pub filter: String,
    pub names: List<NameEntry>,
    #[default(String::new())]
    pub first: String,
    #[default(String::new())]
    pub last: String,
    #[default(0usize)]
    pub selected_ptr: usize,
}

#[controller]
impl Crud {
    #[on(input, String)]
    pub fn update_filter(&mut self, v: String) {
        set!(self.filter => v.clone());
        let prefix = v.to_lowercase();
        let names = self.names.borrow();
        for entry in names.items() {
            let last_lc = entry.last.to_lowercase();
            entry
                .visible
                .set(prefix.is_empty() || last_lc.starts_with(&prefix));
        }
    }

    #[on(watch, selected_ptr)]
    pub fn on_select(&mut self) {
        let ptr = self.selected_ptr.read();
        let names = self.names.borrow();
        for entry in names.items() {
            if Rc::as_ptr(entry) as usize == ptr {
                entry.sel_class.set("selected".to_string());
                self.first.set(entry.first.clone());
                self.last.set(entry.last.clone());
            } else {
                entry.sel_class.set(String::new());
            }
        }
    }

    pub fn create_entry(&mut self) {
        let first = self.first.read();
        let last = self.last.read();
        if last.trim().is_empty() {
            return;
        }
        let entry = NameEntry::new(&first, &last);
        let filter = self.filter.read().to_lowercase();
        if !filter.is_empty() && !last.to_lowercase().starts_with(&filter) {
            entry.visible.set(false);
        }
        self.names.borrow_mut().push(entry);
    }

    pub fn update_entry(&mut self) {
        let ptr = self.selected_ptr.read();
        if ptr == 0 {
            return;
        }
        let first = self.first.read();
        let last = self.last.read();
        if last.trim().is_empty() {
            return;
        }
        let to_remove: Option<Rc<NameEntry>> = {
            let ns = self.names.borrow();
            ns.items()
                .iter()
                .find(|e| Rc::as_ptr(*e) as usize == ptr)
                .cloned()
        };
        if let Some(old) = to_remove {
            self.names.borrow_mut().remove(&old);
            let new_entry = NameEntry::new(&first, &last);
            let filter = self.filter.read().to_lowercase();
            if !filter.is_empty() && !last.to_lowercase().starts_with(&filter) {
                new_entry.visible.set(false);
            }
            self.names.borrow_mut().push(new_entry);
            self.selected_ptr.set(0);
        }
    }

    pub fn delete_entry(&mut self) {
        let ptr = self.selected_ptr.read();
        if ptr == 0 {
            return;
        }
        let to_remove: Option<Rc<NameEntry>> = {
            let ns = self.names.borrow();
            ns.items()
                .iter()
                .find(|e| Rc::as_ptr(*e) as usize == ptr)
                .cloned()
        };
        if let Some(entry) = to_remove {
            self.names.borrow_mut().remove(&entry);
            self.selected_ptr.set(0);
        }
    }
}

#[view(Crud)]
fn render() -> Box<ViewComposite> {
    style! {
        .crud-wrap { display: flex; flex-direction: column; gap: 0.75rem; max-width: 360px; }
        .crud-filter { display: flex; align-items: center; gap: 0.5rem; }
        .crud-filter label { white-space: nowrap; font-size: 0.85rem; color: var(--brick-muted); }
        .crud-filter input { flex: 1; margin-bottom: 0; }
        .crud-list { border: 1px solid var(--brick-border); border-radius: 0.375rem; min-height: 120px; max-height: 200px; overflow-y: auto; }
        .name-entry { padding: 0.4rem 0.75rem; cursor: pointer; border-bottom: 1px solid var(--brick-border); }
        .name-entry:hover { background: color-mix(in srgb, var(--brick-accent) 8%, transparent); }
        .selected .name-entry { background: var(--brick-accent); color: #fff; }
        .crud-fields { display: flex; gap: 0.5rem; }
        .crud-fields input { flex: 1; margin-bottom: 0; }
        .crud-actions { display: flex; gap: 0.5rem; }
        .crud-actions button { flex: 1; }
    }
    children! {
        h2("CRUD"),
        div {
            class("crud-wrap"),
            div {
                class("crud-filter"),
                label("Filter (surname):"),
                input().attr("type", "text").trigger(&my.update_filter),
            },
            div {
                class("crud-list"),
                {
                    let selected_ptr = my.selected_ptr.clone();
                    list!(my.names, |entry| {
                        let ptr = Rc::as_ptr(entry) as usize;
                        let name_text = format!("{}, {}", entry.last, entry.first);
                        Box::new(ViewConditional::new(
                            entry.visible.clone(),
                            div {
                                class(entry.sel_class),
                                p(&name_text)
                                    .c("name-entry")
                                    .set(&selected_ptr, ptr),
                            },
                            None,
                        ))
                    })
                },
            },
            div {
                class("crud-fields"),
                input().attr("type", "text").attr("placeholder", "First name").bind(&my.first),
                input().attr("type", "text").attr("placeholder", "Surname").bind(&my.last),
            },
            div {
                class("crud-actions"),
                button("Create").primary().trigger(&my.create_entry),
                button("Update").secondary().trigger(&my.update_entry),
                button("Delete").danger().trigger(&my.delete_entry),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    #[test]
    fn renders_title() {
        let crud = Crud { ..cascade() };
        assert!(crate::view_components::render_to_html(&*crud.into_component()).contains("CRUD"));
    }

    #[test]
    fn starts_empty() {
        let crud = Crud { ..cascade() };
        assert_eq!(crud.names.borrow().items().len(), 0);
    }

    #[test]
    fn create_adds_entry() {
        let mut crud = Crud { ..cascade() };
        crud.first.set("John".to_string());
        crud.last.set("Doe".to_string());
        crud.create_entry();
        assert_eq!(crud.names.borrow().items().len(), 1);
    }

    #[test]
    fn create_skips_empty_last() {
        let mut crud = Crud { ..cascade() };
        crud.first.set("John".to_string());
        crud.create_entry();
        assert_eq!(crud.names.borrow().items().len(), 0);
    }

    #[test]
    fn delete_removes_entry() {
        let mut crud = Crud { ..cascade() };
        crud.first.set("Jane".to_string());
        crud.last.set("Smith".to_string());
        crud.create_entry();
        assert_eq!(crud.names.borrow().items().len(), 1);
        let ptr = {
            let ns = crud.names.borrow();
            Rc::as_ptr(ns.items().first().unwrap()) as usize
        };
        crud.selected_ptr.set(ptr);
        crud.delete_entry();
        assert_eq!(crud.names.borrow().items().len(), 0);
    }

    #[test]
    fn update_replaces_entry() {
        let mut crud = Crud { ..cascade() };
        crud.first.set("John".to_string());
        crud.last.set("Doe".to_string());
        crud.create_entry();
        let ptr = {
            let ns = crud.names.borrow();
            Rc::as_ptr(ns.items().first().unwrap()) as usize
        };
        crud.selected_ptr.set(ptr);
        crud.first.set("Jane".to_string());
        crud.last.set("Smith".to_string());
        crud.update_entry();
        let ns = crud.names.borrow();
        assert_eq!(ns.items().len(), 1);
        assert_eq!(ns.items().first().unwrap().first, "Jane");
        assert_eq!(ns.items().first().unwrap().last, "Smith");
    }

    #[test]
    fn filter_hides_non_matching() {
        let mut crud = Crud { ..cascade() };
        crud.first.set("John".to_string());
        crud.last.set("Doe".to_string());
        crud.create_entry();
        crud.first.set("Jane".to_string());
        crud.last.set("Smith".to_string());
        crud.create_entry();
        crud.update_filter("sm".to_string());
        let ns = crud.names.borrow();
        let visible: Vec<bool> = ns.items().iter().map(|e| e.visible.read()).collect();
        assert_eq!(visible, vec![false, true]);
    }
}
