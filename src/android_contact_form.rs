use crate::view_components::*;

#[model]
struct AndroidContactForm {
    #[validate(required, min_length(2, "Name must be at least 2 characters"))]
    name: String,
    #[validate(required, email)]
    email: String,
    #[validate(required, min_length(10, "Message must be at least 10 characters"))]
    message: String,
    #[default(false)]
    submitted: bool,
}

#[controller]
impl AndroidContactForm {
    #[on(input, String)]
    fn set_name(&mut self, v: String) {
        self.name.set(v);
        self.validate_name();
    }

    #[on(input, String)]
    fn set_email(&mut self, v: String) {
        self.email.set(v);
        self.validate_email();
    }

    #[on(input, String)]
    fn set_message(&mut self, v: String) {
        self.message.set(v);
        self.validate_message();
    }

    fn send(&mut self) {
        if self.validate_all() {
            self.submitted.set(true);
            crate::portable::push_rerender_signal();
        }
    }
}

#[view(AndroidContactForm)]
fn contact_form_view() {
    portable! {
        {
            if self.submitted.read() {
                crate::portable::column(vec![
                    crate::portable::into_portable(
                        crate::portable::p("Thanks! We'll be in touch.")
                    ),
                ])
            } else {
                crate::portable::column(vec![
                    crate::portable::into_portable(crate::portable::p("Name")),
                    crate::portable::into_portable(
                        crate::portable::input()
                            .bind(&self.name)
                            .on_change(&AndroidContactForm::SET_NAME),
                    ),
                    crate::portable::into_portable(crate::portable::p(&self.name_error)),
                    crate::portable::into_portable(crate::portable::p("Email")),
                    crate::portable::into_portable(
                        crate::portable::input()
                            .bind(&self.email)
                            .on_change(&AndroidContactForm::SET_EMAIL),
                    ),
                    crate::portable::into_portable(crate::portable::p(&self.email_error)),
                    crate::portable::into_portable(crate::portable::p("Message")),
                    crate::portable::into_portable(
                        crate::portable::input()
                            .bind(&self.message)
                            .on_change(&AndroidContactForm::SET_MESSAGE),
                    ),
                    crate::portable::into_portable(crate::portable::p(&self.message_error)),
                    crate::portable::into_portable(
                        crate::portable::button("Send").trigger(&AndroidContactForm::SEND),
                    ),
                ])
            }
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
    static FORM_MODEL: RefCell<Option<Rc<RefCell<AndroidContactForm>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickContactFormBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    FORM_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidContactForm = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidContactForm::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = FORM_MODEL.with(|slot| {
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
