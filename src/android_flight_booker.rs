use crate::view_components::*;

/// One-way vs. return flight selector.
#[model]
struct AndroidFlightBooker {
    #[default(false)]
    is_return: bool,
    #[default(String::from("2026-06-01"))]
    depart_date: String,
    #[default(String::from("2026-06-15"))]
    return_date: String,
    #[default(String::new())]
    confirmation: String,
}

#[controller]
impl AndroidFlightBooker {
    #[on(change, bool)]
    fn toggle_return(&mut self, v: bool) {
        self.is_return.set(v);
        crate::portable::push_rerender_signal();
    }

    #[on(input, String)]
    fn set_depart(&mut self, v: String) {
        self.depart_date.set(v);
    }

    #[on(input, String)]
    fn set_return_date(&mut self, v: String) {
        self.return_date.set(v);
    }

    fn book(&mut self) {
        let msg = if self.is_return.read() {
            format!("Return flight: {} → {}", self.depart_date.read(), self.return_date.read())
        } else {
            format!("One-way flight: {}", self.depart_date.read())
        };
        self.confirmation.set(msg);
        crate::portable::push_rerender_signal();
    }
}

#[view(AndroidFlightBooker)]
fn flight_booker_view() {
    portable! {
        {
            let mut children: Vec<Box<dyn crate::portable::PortableElement>> = vec![
                crate::portable::into_portable(crate::portable::p("Flight Booker")),
                crate::portable::into_portable(
                    crate::portable::toggle("Return flight")
                        .bind(&self.is_return)
                        .on_change(&AndroidFlightBooker::TOGGLE_RETURN),
                ),
                crate::portable::into_portable(
                    crate::portable::input()
                        .placeholder("Departure date (YYYY-MM-DD)")
                        .bind(&self.depart_date)
                        .on_change(&AndroidFlightBooker::SET_DEPART),
                ),
            ];

            if self.is_return.read() {
                children.push(crate::portable::into_portable(
                    crate::portable::input()
                        .placeholder("Return date (YYYY-MM-DD)")
                        .bind(&self.return_date)
                        .on_change(&AndroidFlightBooker::SET_RETURN_DATE),
                ));
            }

            let conf = self.confirmation.read();
            if !conf.is_empty() {
                children.push(crate::portable::into_portable(crate::portable::p(&*conf)));
            } else {
                children.push(crate::portable::into_portable(
                    crate::portable::button("Book Flight").trigger(&AndroidFlightBooker::BOOK),
                ));
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
    static FLIGHT_MODEL: RefCell<Option<Rc<RefCell<AndroidFlightBooker>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickFlightBookerBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    FLIGHT_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidFlightBooker = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidFlightBooker::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = FLIGHT_MODEL.with(|slot| {
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
