use crate::view_components::*;

#[derive(Clone)]
struct TodoItem {
    text: String,
    completed: bool,
}

/// Filter state: 0=All, 1=Active, 2=Completed.
#[model]
struct AndroidTodoMvc {
    #[default(String::new())]
    input: String,
    #[default(Vec::new())]
    todos: Vec<TodoItem>,
    #[default(0i32)]
    filter: i32,
}

#[controller]
impl AndroidTodoMvc {
    #[on(input, String)]
    fn set_input(&mut self, v: String) {
        self.input.set(v);
    }

    fn add_todo(&mut self) {
        let text = self.input.read();
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            let mut todos = self.todos.read();
            todos.push(TodoItem { text: trimmed, completed: false });
            self.todos.set(todos);
            self.input.set(String::new());
            crate::portable::push_rerender_signal();
        }
    }

    #[on(click, i32)]
    fn toggle_item_at(&mut self, idx: i32) {
        let mut todos = self.todos.read();
        if let Some(item) = todos.get_mut(idx as usize) {
            item.completed = !item.completed;
        }
        self.todos.set(todos);
        crate::portable::push_rerender_signal();
    }

    #[on(click, i32)]
    fn delete_item(&mut self, idx: i32) {
        let mut todos = self.todos.read();
        let i = idx as usize;
        if i < todos.len() {
            todos.remove(i);
        }
        self.todos.set(todos);
        crate::portable::push_rerender_signal();
    }

    fn filter_all(&mut self) { self.filter.set(0); crate::portable::push_rerender_signal(); }
    fn filter_active(&mut self) { self.filter.set(1); crate::portable::push_rerender_signal(); }
    fn filter_completed(&mut self) { self.filter.set(2); crate::portable::push_rerender_signal(); }

    fn clear_completed(&mut self) {
        let todos = self.todos.read();
        self.todos.set(todos.into_iter().filter(|t| !t.completed).collect());
        crate::portable::push_rerender_signal();
    }
}

#[view(AndroidTodoMvc)]
fn todo_mvc_view() {
    portable! {
        {
            let filter = self.filter.read();
            let todos = self.todos.read();
            let visible: Vec<(usize, &TodoItem)> = todos.iter().enumerate()
                .filter(|(_, t)| match filter {
                    1 => !t.completed,
                    2 => t.completed,
                    _ => true,
                })
                .collect();

            let remaining = todos.iter().filter(|t| !t.completed).count();

            let mut children: Vec<Box<dyn crate::portable::PortableElement>> = vec![
                crate::portable::into_portable(crate::portable::p("TodoMVC")),
                // Input row
                crate::portable::into_portable(crate::portable::row(vec![
                    crate::portable::into_portable(
                        crate::portable::input()
                            .placeholder("What needs to be done?")
                            .bind(&self.input)
                            .on_change(&AndroidTodoMvc::SET_INPUT),
                    ),
                    crate::portable::into_portable(
                        crate::portable::button("Add").trigger(&AndroidTodoMvc::ADD_TODO)
                    ),
                ])),
            ];

            // Todo items
            for (orig_idx, item) in &visible {
                let idx = *orig_idx as i32;
                let label = if item.completed {
                    format!("✓ {}", item.text)
                } else {
                    item.text.clone()
                };
                children.push(crate::portable::into_portable(crate::portable::row(vec![
                    crate::portable::into_portable(crate::portable::p(label.as_str())),
                    crate::portable::into_portable(
                        crate::portable::button(if item.completed { "Undo" } else { "Done" })
                            .trigger_int(&AndroidTodoMvc::TOGGLE_ITEM_AT, idx)
                    ),
                    crate::portable::into_portable(
                        crate::portable::button("✕").trigger_int(&AndroidTodoMvc::DELETE_ITEM, idx)
                    ),
                ])));
            }

            // Status + filter bar
            children.push(crate::portable::into_portable(
                crate::portable::p(format!("{} item(s) left", remaining).as_str())
            ));
            children.push(crate::portable::into_portable(crate::portable::row(vec![
                crate::portable::into_portable(
                    crate::portable::button("All").trigger(&AndroidTodoMvc::FILTER_ALL)
                ),
                crate::portable::into_portable(
                    crate::portable::button("Active").trigger(&AndroidTodoMvc::FILTER_ACTIVE)
                ),
                crate::portable::into_portable(
                    crate::portable::button("Completed").trigger(&AndroidTodoMvc::FILTER_COMPLETED)
                ),
                crate::portable::into_portable(
                    crate::portable::button("Clear done").trigger(&AndroidTodoMvc::CLEAR_COMPLETED)
                ),
            ])));

            crate::portable::scroll(children)
        }
    }
    children! {}
}

// ── JNI entry point ───────────────────────────────────────────────────────────

use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static TODO_MVC_MODEL: RefCell<Option<Rc<RefCell<AndroidTodoMvc>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickTodoMvcBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    TODO_MVC_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidTodoMvc = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidTodoMvc::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = TODO_MVC_MODEL.with(|slot| {
        let borrow = slot.borrow();
        let model_rc = borrow.as_ref().unwrap();
        let model = model_rc.borrow();
        use crate::portable::PortableView as _;
        crate::portable::encode_blueprint(&*model)
    });

    let arr = match env.new_byte_array(bytes.len() as i32) {
        Ok(a) => a,
        Err(_) => return std::ptr::null_mut(),
    };
    let signed: Vec<i8> = bytes.iter().map(|&b| b as i8).collect();
    if env.set_byte_array_region(&arr, 0, &signed).is_err() {
        return std::ptr::null_mut();
    }
    arr.into_raw()
}
