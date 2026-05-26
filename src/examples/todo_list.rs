use crate::view_components::*;

pub struct TodoItem {
    pub text: String,
}

#[model]
pub struct TodoList {
    #[default(String::new())]
    pub input: String,
    pub todos: List<TodoItem>,
}

#[controller]
impl TodoList {
    pub fn add_todo(&mut self) {
        let text = self.input.read();
        if !text.trim().is_empty() {
            self.todos.borrow_mut().push(TodoItem { text });
            set!(self.input => String::new());
        }
    }
}

#[view(TodoList)]
fn render() -> Box<ViewComposite> {
    style! {
        .add-row { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; }
        .add-row input { flex: 1; margin-bottom: 0; }
        .add-row button { margin-top: 0; white-space: nowrap; }
        .todo-item { display: flex; justify-content: space-between; align-items: center; padding: 0.35rem 0.5rem; border-radius: 0.3rem; margin-top: 0.4rem; }
        .todo-item p { margin-top: 0; color: var(--brick-text); }
        .todo-item button { margin-top: 0; }
    }
    children! {
        div {
            class("add-row"),
            input().attr("type", "text").attr("placeholder", "New todo…").bind(&my.input),
            button("Add").primary().trigger(&my.add_todo),
        },
        list!(my.todos, |todo|
            div {
                class("todo-item"),
                p(&todo.text),
                button("✕").danger().sm().remove(),
            }
        ),
    }
}
