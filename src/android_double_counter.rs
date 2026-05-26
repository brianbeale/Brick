use crate::view_components::*;

#[model]
struct AndroidDoubleCounter {
    #[default(0)]
    left: i32,
    #[default(0)]
    right: i32,
}

#[controller]
impl AndroidDoubleCounter {
    fn left_up(&mut self) { self.left.update(|n| *n += 1); }
    fn left_down(&mut self) { self.left.update(|n| *n -= 1); }
    fn right_up(&mut self) { self.right.update(|n| *n += 1); }
    fn right_down(&mut self) { self.right.update(|n| *n -= 1); }
    fn reset_both(&mut self) {
        self.left.set(0);
        self.right.set(0);
    }
}

#[view(AndroidDoubleCounter)]
fn double_counter_view() {
    portable! {
        crate::column![
            crate::portable::p("Double Counter"),
            crate::portable::row(vec![
                // Left counter
                crate::portable::into_portable(crate::portable::column(vec![
                    crate::portable::into_portable(crate::portable::p("Left")),
                    crate::portable::into_portable(crate::portable::p(&self.left)),
                    crate::portable::into_portable(
                        crate::portable::button("+").trigger(&AndroidDoubleCounter::LEFT_UP)
                    ),
                    crate::portable::into_portable(
                        crate::portable::button("\u{2212}").trigger(&AndroidDoubleCounter::LEFT_DOWN)
                    ),
                ])),
                // Right counter
                crate::portable::into_portable(crate::portable::column(vec![
                    crate::portable::into_portable(crate::portable::p("Right")),
                    crate::portable::into_portable(crate::portable::p(&self.right)),
                    crate::portable::into_portable(
                        crate::portable::button("+").trigger(&AndroidDoubleCounter::RIGHT_UP)
                    ),
                    crate::portable::into_portable(
                        crate::portable::button("\u{2212}").trigger(&AndroidDoubleCounter::RIGHT_DOWN)
                    ),
                ])),
            ]),
            crate::portable::button("Reset Both").trigger(&AndroidDoubleCounter::RESET_BOTH),
        ]
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
    static DOUBLE_COUNTER_MODEL: RefCell<Option<Rc<RefCell<AndroidDoubleCounter>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickDoubleCounterBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    DOUBLE_COUNTER_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidDoubleCounter = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidDoubleCounter::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = DOUBLE_COUNTER_MODEL.with(|slot| {
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
