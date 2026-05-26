use crate::view_components::*;

#[model]
struct AndroidThermometer {
    #[default(20.0f64)]
    celsius: f64,
    #[default(String::from("20"))]
    celsius_input: String,
    #[from(celsius, |c: f64| (c * 9.0 / 5.0 + 32.0) * 10.0_f64.powi(1) / 10.0_f64.powi(1))]
    fahrenheit: f64,
}

#[controller]
impl AndroidThermometer {
    #[on(input, String)]
    fn set_celsius(&mut self, v: String) {
        self.celsius_input.set(v.clone());
        if let Ok(c) = v.trim().parse::<f64>() {
            self.celsius.set(c);
        }
    }
}

#[view(AndroidThermometer)]
fn thermo_view() {
    portable! {
        crate::column![
            crate::portable::p("Temperature Converter"),
            crate::portable::row(vec![
                crate::portable::into_portable(
                    crate::portable::input()
                        .placeholder("Celsius")
                        .bind(&self.celsius_input)
                        .on_change(&AndroidThermometer::SET_CELSIUS),
                ),
                crate::portable::into_portable(crate::portable::p("°C")),
            ]),
            crate::portable::row(vec![
                crate::portable::into_portable(crate::portable::p(&self.fahrenheit)),
                crate::portable::into_portable(crate::portable::p("°F")),
            ]),
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
    static THERMO_MODEL: RefCell<Option<Rc<RefCell<AndroidThermometer>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickThermometerBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    THERMO_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let mut model: AndroidThermometer = cascade();
            model.wire_cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidThermometer::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = THERMO_MODEL.with(|slot| {
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
