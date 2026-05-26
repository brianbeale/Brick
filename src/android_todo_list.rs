use crate::view_components::*;

#[model]
struct AndroidTodoList {
    #[default(String::new())]
    input: String,
    #[default(Vec::new())]
    todos: Vec<String>,
}

#[controller]
impl AndroidTodoList {
    #[on(input, String)]
    fn set_input(&mut self, v: String) {
        self.input.set(v);
    }

    fn add_todo(&mut self) {
        let text = self.input.read();
        if !text.trim().is_empty() {
            let mut todos = self.todos.read();
            todos.push(text.trim().to_string());
            self.todos.set(todos);
            self.input.set(String::new());
            crate::portable::push_rerender_signal();
        }
    }

    #[on(click, i32)]
    fn remove_item(&mut self, idx: i32) {
        let mut todos = self.todos.read();
        let i = idx as usize;
        if i < todos.len() {
            todos.remove(i);
            self.todos.set(todos);
            crate::portable::push_rerender_signal();
        }
    }
}

#[view(AndroidTodoList)]
fn todo_view() {
    portable! {
        {
            let mut children: Vec<Box<dyn crate::portable::PortableElement>> = vec![
                crate::portable::into_portable(crate::portable::row(vec![
                    crate::portable::into_portable(
                        crate::portable::input()
                            .bind(&self.input)
                            .on_change(&AndroidTodoList::SET_INPUT),
                    ),
                    crate::portable::into_portable(
                        crate::portable::button("Add").trigger(&AndroidTodoList::ADD_TODO),
                    ),
                ])),
            ];
            for (idx, text) in self.todos.read().iter().enumerate() {
                let i = idx as i32;
                children.push(crate::portable::into_portable(crate::portable::row(vec![
                    crate::portable::into_portable(crate::portable::p(text.as_str())),
                    crate::portable::into_portable(
                        crate::portable::button("✕").trigger_int(&AndroidTodoList::REMOVE_ITEM, i),
                    ),
                ])));
            }
            crate::portable::column(children)
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
    static TODO_MODEL: RefCell<Option<Rc<RefCell<AndroidTodoList>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickTodoListBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    TODO_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidTodoList = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidTodoList::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = TODO_MODEL.with(|slot| {
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
