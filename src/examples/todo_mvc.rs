use crate::state_mgmt::Effect;
use crate::view_components::*;

pub struct TodoItem {
    pub text: String,
    pub completed: Signal<bool>,
    pub item_class: Signal<String>,
}

impl TodoItem {
    pub fn new(text: String) -> Self {
        let completed = Signal::new(false);
        let item_class = Signal::new("todo-active".to_string());
        let ic = item_class.clone();
        completed.add_observer(
            "item-class",
            Box::new(Effect::new(move |b: &bool| {
                ic.set(if *b { "todo-completed" } else { "todo-active" }.to_string());
            })),
        );
        TodoItem {
            text,
            completed,
            item_class,
        }
    }
}

impl Clone for TodoItem {
    fn clone(&self) -> Self {
        TodoItem {
            text: self.text.clone(),
            completed: self.completed.clone(),
            item_class: self.item_class.clone(),
        }
    }
}

#[model]
pub struct TodoMvc {
    #[default(String::new())]
    pub input: String,
    pub todos: List<TodoItem>,
    #[default("filter-all".to_string())]
    pub filter: String,
    #[default(0usize)]
    pub active_count: usize,
}

#[controller]
impl TodoMvc {
    pub fn add_todo(&mut self) {
        let text = self.input.read();
        if text.trim().is_empty() {
            return;
        }
        let item = TodoItem::new(text);

        // Delta-track count: avoid re-reading todos list inside observer (borrow conflict).
        let count = self.active_count.clone();
        let mut prev = false; // new item always starts as active (false = not completed)
        item.completed.add_observer(
            "count-delta",
            Box::new(Effect::new(move |b: &bool| {
                let now = *b;
                if now && !prev {
                    count.update(|v| *v = v.saturating_sub(1));
                } else if !now && prev {
                    count.update(|v| *v += 1);
                }
                prev = now;
            })),
        );

        self.todos.borrow_mut().push(item);
        self.active_count.update(|v| *v += 1);
        set!(self.input => String::new());
    }

    pub fn toggle_all(&mut self) {
        let all_done = self.active_count.read() == 0;
        let todos = self.todos.borrow();
        for item in todos.items() {
            item.completed.set(!all_done);
        }
    }

    pub fn clear_completed(&mut self) {
        let to_remove: Vec<Rc<TodoItem>> = self
            .todos
            .borrow()
            .items()
            .iter()
            .filter(|i| i.completed.read())
            .cloned()
            .collect();
        for item in &to_remove {
            self.todos.borrow_mut().remove(item);
        }
    }
}

#[view(TodoMvc)]
fn render() -> Box<ViewComposite> {
    style! {
        h1 { text-align: center; font-weight: 100; font-size: 3rem; margin-bottom: 0.25rem; color: rgba(175,47,47,.15); }
        .add-row { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; }
        .add-row input { flex: 1; margin-bottom: 0; }
        .add-row button { white-space: nowrap; }
        .todo-item { display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 0.25rem; border-bottom: 1px solid var(--brick-border); }
        .todo-item input[type=checkbox] { flex-shrink: 0; accent-color: var(--brick-accent); width: 1.1rem; height: 1.1rem; }
        .todo-item span { flex: 1; }
        .todo-completed .todo-item span { text-decoration: line-through; opacity: 0.5; }
        .footer { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 0.5rem; margin-top: 0.75rem; font-size: 0.85rem; }
        .filter-active .todo-completed { display: none; }
        .filter-completed .todo-active { display: none; }
    }
    children! {
        h1("todos"),
        div {
            class("add-row"),
            input().attr("type", "text").attr("placeholder", "What needs to be done?").bind(&my.input),
            button("Add").primary().trigger(&my.add_todo),
        },
        div {
            class(my.filter),
            list!(my.todos, |todo|
                div {
                    class(todo.item_class),
                    div {
                        class("todo-item"),
                        input().attr("type", "checkbox").toggle(&todo.completed),
                        span(&todo.text),
                        button("×").danger().sm().remove(),
                    }
                }
            ),
        },
        div {
            class("footer"),
            p(live!("{my.active_count} items left")).muted(),
            row! {
                button("All").set(&my.filter, "filter-all".to_string()),
                button("Active").set(&my.filter, "filter-active".to_string()),
                button("Completed").set(&my.filter, "filter-completed".to_string()),
            },
            button("Clear completed").ghost().trigger(&my.clear_completed),
        },
    }
}
