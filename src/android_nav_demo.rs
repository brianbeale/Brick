use crate::view_components::*;
use crate::portable::Navigator;

#[derive(Clone, PartialEq, Debug)]
enum NavScreen {
    Home,
    Detail { item_idx: usize },
    Settings,
}

#[model]
struct AndroidNavDemo {
    #[default(NavScreen::Home)]
    nav: Navigator<NavScreen>,
    #[default(String::from("Alice"))]
    item_a: String,
    #[default(String::from("Bob"))]
    item_b: String,
    #[default(String::from("Charlie"))]
    item_c: String,
    #[default(false)]
    dark_mode: bool,
}

#[controller]
impl AndroidNavDemo {
    fn go_detail_0(&mut self) { self.nav.push(NavScreen::Detail { item_idx: 0 }); }
    fn go_detail_1(&mut self) { self.nav.push(NavScreen::Detail { item_idx: 1 }); }
    fn go_detail_2(&mut self) { self.nav.push(NavScreen::Detail { item_idx: 2 }); }
    fn go_settings(&mut self) { self.nav.push(NavScreen::Settings); }
    fn go_back(&mut self) { self.nav.pop(); }

    #[on(change, bool)]
    fn toggle_dark_mode(&mut self, v: bool) {
        self.dark_mode.set(v);
    }
}

#[view(AndroidNavDemo)]
fn nav_demo_view() {
    portable! {
        {
            let screen = self.nav.current().clone();
            match screen {
                NavScreen::Home => {
                    let items = [
                        self.item_a.read(),
                        self.item_b.read(),
                        self.item_c.read(),
                    ];
                    crate::column![
                        crate::portable::p("Navigator Demo — Home"),
                        crate::portable::button(&*items[0]).trigger(&AndroidNavDemo::GO_DETAIL_0),
                        crate::portable::button(&*items[1]).trigger(&AndroidNavDemo::GO_DETAIL_1),
                        crate::portable::button(&*items[2]).trigger(&AndroidNavDemo::GO_DETAIL_2),
                        crate::portable::button("Settings").trigger(&AndroidNavDemo::GO_SETTINGS),
                    ]
                }
                NavScreen::Detail { item_idx } => {
                    let name = match item_idx {
                        0 => self.item_a.read(),
                        1 => self.item_b.read(),
                        _ => self.item_c.read(),
                    };
                    crate::column![
                        crate::portable::p(format!("Detail: {}", name).as_str()),
                        crate::portable::p(format!("Item #{}", item_idx + 1).as_str()),
                        crate::portable::button("← Back").trigger(&AndroidNavDemo::GO_BACK),
                    ]
                }
                NavScreen::Settings => {
                    crate::column![
                        crate::portable::p("Settings"),
                        crate::portable::toggle("Dark mode")
                            .bind(&self.dark_mode)
                            .on_change(&AndroidNavDemo::TOGGLE_DARK_MODE),
                        crate::portable::button("← Back").trigger(&AndroidNavDemo::GO_BACK),
                    ]
                }
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
    static NAV_MODEL: RefCell<Option<Rc<RefCell<AndroidNavDemo>>>> =
        const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_demo_MainActivity_brickNavDemoBlueprintBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jbyteArray {
    NAV_MODEL.with(|slot| {
        if slot.borrow().is_none() {
            use crate::state_mgmt::cascade;
            let model: AndroidNavDemo = cascade();
            *slot.borrow_mut() = Some(Rc::new(RefCell::new(model)));
        }
        AndroidNavDemo::register_android_actions(Rc::clone(slot.borrow().as_ref().unwrap()));
    });

    let bytes = NAV_MODEL.with(|slot| {
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
