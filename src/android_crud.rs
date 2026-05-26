use crate::view_components::*;

#[derive(Clone)]
struct CrudEntry {
    first: String,
    last: String,
}

impl CrudEntry {
    fn full(&self) -> String {
        format!("{} {}", self.first, self.last)
    }
}

#[model]
struct AndroidCrud {
    #[default(Vec::new())]
    entries: Vec<CrudEntry>,
    #[default(-1i32)]
    selected_idx: i32,
    #[default(String::new())]
    filter_prefix: String,
    #[default(String::new())]
    edit_first: String,
    #[default(String::new())]
    edit_last: String,
}

#[controller]
impl AndroidCrud {
    #[on(click, i32)]
    fn select_item(&mut self, idx: i32) {
        self.selected_idx.set(idx);
        let entries = self.entries.read();
        if let Some(e) = entries.get(idx as usize) {
            self.edit_first.set(e.first.clone());
            self.edit_last.set(e.last.clone());
        }
        crate::portable::push_rerender_signal();
    }

    #[on(input, String)]
    fn set_filter(&mut self, v: String) {
        self.filter_prefix.set(v);
        self.selected_idx.set(-1);
        crate::portable::push_rerender_signal();
    }

    #[on(input, String)]
    fn set_edit_first(&mut self, v: String) {
        self.edit_first.set(v);
    }

    #[on(input, String)]
    fn set_edit_last(&mut self, v: String) {
        self.edit_last.set(v);
    }

    fn create_entry(&mut self) {
        let first = self.edit_first.read();
        let last = self.edit_last.read();
        if !first.trim().is_empty() || !last.trim().is_empty() {
            let mut entries = self.entries.read();
            entries.push(CrudEntry { first: first.trim().to_string(), last: last.trim().to_string() });
            self.entries.set(entries);
            self.edit_first.set(String::new());
            self.edit_last.set(String::new());
            crate::portable::push_rerender_signal();
        }
    }

    fn update_entry(&mut self) {
        let idx = self.selected_idx.read();
        if idx < 0 { return; }
        let mut entries = self.entries.read();
        if let Some(e) = entries.get_mut(idx as usize) {
            e.first = self.edit_first.read().trim().to_string();
            e.last = self.edit_last.read().trim().to_string();
        }
        self.entries.set(entries);
        crate::portable::push_rerender_signal();
    }

    fn delete_entry(&mut self) {
        let idx = self.selected_idx.read();
        if idx < 0 { return; }
        let mut entries = self.entries.read();
        let i = idx as usize;
        if i < entries.len() {
            entries.remove(i);
        }
        self.entries.set(entries);
        self.selected_idx.set(-1);
        crate::portable::push_rerender_signal();
    }
}

#[view(AndroidCrud)]
fn crud_view() {
    portable! {
        {
            let prefix = self.filter_prefix.read().to_lowercase();
            let entries = self.entries.read();
            let selected = self.selected_idx.read();

            let filtered: Vec<(usize, &CrudEntry)> = entries.iter().enumerate()
                .filter(|(_, e)| prefix.is_empty() || e.full().to_lowercase().starts_with(&*prefix))
                .collect();

            let mut children: Vec<Box<dyn crate::portable::PortableElement>> = vec![
                crate::portable::into_portable(crate::portable::p("CRUD")),
                crate::portable::into_portable(
                    crate::portable::input()
                        .placeholder("Filter by name prefix")
                        .bind(&self.filter_prefix)
                        .on_change(&AndroidCrud::SET_FILTER),
                ),
            ];

            // Entry list
            for (orig_idx, entry) in &filtered {
                let idx = *orig_idx as i32;
                let is_selected = selected == idx;
                let label = if is_selected {
                    format!("▶ {}", entry.full())
                } else {
                    entry.full()
                };
                children.push(crate::portable::into_portable(
                    crate::portable::button(label.as_str())
                        .trigger_int(&AndroidCrud::SELECT_ITEM, idx)
                ));
            }

            // Edit fields
            children.push(crate::portable::into_portable(
                crate::portable::input()
                    .placeholder("First name")
                    .bind(&self.edit_first)
                    .on_change(&AndroidCrud::SET_EDIT_FIRST),
            ));
            children.push(crate::portable::into_portable(
                crate::portable::input()
                    .placeholder("Last name")
                    .bind(&self.edit_last)
                    .on_change(&AndroidCrud::SET_EDIT_LAST),
            ));

            // Action buttons
            children.push(crate::portable::into_portable(crate::portable::row(vec![
                crate::portable::into_portable(
                    crate::portable::button("Create").trigger(&AndroidCrud::CREATE_ENTRY)
                ),
                crate::portable::into_portable(
                    crate::portable::button("Update").trigger(&AndroidCrud::UPDATE_ENTRY)
                ),
                crate::portable::into_portable(
                    crate::portable::button("Delete").trigger(&AndroidCrud::DELETE_ENTRY)
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
    static CRUD_MODEL: RefCell<Option<Rc<RefCell<AndroidCrud>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickCrudBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    CRUD_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidCrud = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidCrud::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = CRUD_MODEL.with(|slot| {
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
