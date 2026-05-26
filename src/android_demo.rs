use crate::view_components::*;

#[model]
struct AndroidCounter {
    #[default(0)]
    count: i32,
}

#[controller]
impl AndroidCounter {
    fn increment(&mut self) {
        self.count.update(|n| *n += 1);
    }
    fn decrement(&mut self) {
        self.count.update(|n| *n -= 1);
    }
}

#[view(AndroidCounter)]
fn counter_view() {
    portable! {
        crate::column![
            crate::portable::p(&self.count),
            crate::portable::button("+").trigger(&AndroidCounter::INCREMENT),
            crate::portable::button("\u{2212}").trigger(&AndroidCounter::DECREMENT),
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

// Thread-local: Signal<i32> is !Send. All JNI calls land on the Android main thread.
thread_local! {
    static MODEL: RefCell<Option<Rc<RefCell<AndroidCounter>>>> = const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickDemoBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let counter: AndroidCounter = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(counter)));
        }
        AndroidCounter::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = MODEL.with(|slot| {
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
