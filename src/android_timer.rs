use crate::view_components::*;

#[model]
struct AndroidTimer {
    #[default(0.0f64)]
    elapsed: f64,
    #[default(15.0f64)]
    duration: f64,
}

#[controller]
impl AndroidTimer {
    #[on(interval, 100)]
    fn tick(&mut self) {
        let elapsed = self.elapsed.read();
        let duration = self.duration.read();
        // Round to one decimal to avoid f64 drift ("0.30000000000000004").
        let next = ((elapsed + 0.1) * 10.0).round() / 10.0;
        self.elapsed.set(next.min(duration));
    }

    fn reset(&mut self) {
        self.elapsed.set(0.0);
    }
}

#[view(AndroidTimer)]
fn timer_view() {
    portable! {
        crate::column![
            crate::portable::p(&self.elapsed),
            crate::portable::p(&self.duration),
            crate::portable::button("Reset").trigger(&AndroidTimer::RESET),
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
    static TIMER_MODEL: RefCell<Option<Rc<RefCell<AndroidTimer>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickTimerBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    TIMER_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidTimer = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidTimer::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
        AndroidTimer::register_android_intervals(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = TIMER_MODEL.with(|slot| {
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
