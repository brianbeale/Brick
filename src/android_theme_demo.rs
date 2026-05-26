use crate::view_components::*;

#[model]
struct AndroidThemeDemo {
    sample_text: String,
    #[default(false)]
    toggle_on: bool,
}

#[controller]
impl AndroidThemeDemo {
    #[on(input, String)]
    fn set_sample_text(&mut self, v: String) {
        self.sample_text.set(v);
    }

    #[on(input, bool)]
    fn set_toggle(&mut self, v: bool) {
        self.toggle_on.set(v);
    }
}

#[view(AndroidThemeDemo)]
fn theme_view() {
    portable! {
        crate::portable::scroll(vec![
            // Typography
            crate::portable::into_portable(crate::portable::p("Typography").label()),
            crate::portable::into_portable(crate::portable::p("Title — bold, large, prominent").title()),
            crate::portable::into_portable(crate::portable::p("Body — default paragraph text")),
            crate::portable::into_portable(crate::portable::p("Caption — small, secondary info").caption()),
            crate::portable::into_portable(crate::portable::p("Label — tiny, uppercase, structural").label()),
            // Buttons
            crate::portable::into_portable(crate::portable::p("Buttons").label()),
            crate::portable::into_portable(crate::portable::row(vec![
                crate::portable::into_portable(crate::portable::button("Primary")),
                crate::portable::into_portable(crate::portable::button("Tonal").secondary()),
            ])),
            crate::portable::into_portable(crate::portable::row(vec![
                crate::portable::into_portable(crate::portable::button("Outlined").outlined()),
                crate::portable::into_portable(crate::portable::button("Ghost").ghost()),
            ])),
            crate::portable::into_portable(crate::portable::p("Danger variants").label()),
            crate::portable::into_portable(crate::portable::row(vec![
                crate::portable::into_portable(crate::portable::button("Delete").danger()),
                crate::portable::into_portable(crate::portable::button("Delete").danger_outlined()),
            ])),
            // Input
            crate::portable::into_portable(crate::portable::p("Input").label()),
            crate::portable::into_portable(
                crate::portable::input()
                    .placeholder("Type something…")
                    .bind(&self.sample_text)
                    .on_change(&AndroidThemeDemo::SET_SAMPLE_TEXT),
            ),
            crate::portable::into_portable(crate::portable::p(format!(
                "Value: {}",
                if self.sample_text.read().is_empty() { "(empty)".to_string() } else { self.sample_text.read().clone() }
            ))),
            // Toggle
            crate::portable::into_portable(crate::portable::p("Toggle").label()),
            crate::portable::into_portable(
                crate::portable::toggle(if self.toggle_on.read() { "Enabled" } else { "Disabled" })
                    .bind(&self.toggle_on)
                    .on_change(&AndroidThemeDemo::SET_TOGGLE),
            ),
        ])
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
    static THEME_MODEL: RefCell<Option<Rc<RefCell<AndroidThemeDemo>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickThemeDemoBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    THEME_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidThemeDemo = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidThemeDemo::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = THEME_MODEL.with(|slot| {
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
