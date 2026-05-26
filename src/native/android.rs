/// JNI entry points for the Android renderer.
///
/// Kotlin loads the Brick `.so` via `System.loadLibrary("brick")` and calls
/// these functions directly. All functions follow the JNI naming convention:
/// `Java_<package>_<class>_<method>` → called via JNI `native` keyword in Kotlin.
///
/// # Signal drain protocol
///
/// Wire format per entry: `[signal_id: u32 LE][WIRE_TAG: u8][value_bytes...]`
/// `signal_id = 0` is a reserved re-render sentinel (`WIRE_TAG = 0xFF`, no value bytes).
/// Kotlin calls `setupSignalBuffer()` once then `drainQueue()` each Compose frame.
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jint, jobject};
use jni::JNIEnv;

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

// ── Shared signal buffer ──────────────────────────────────────────────────────

/// Size of the shared signal update buffer in bytes.
pub const SIGNAL_BUFFER_SIZE: usize = 4096;

thread_local! {
    static SIGNAL_BUF: RefCell<Vec<u8>> = RefCell::new(vec![0u8; SIGNAL_BUFFER_SIZE]);
}

// ── JNI: setupSignalBuffer ────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_android_BrickJni_setupSignalBuffer<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jobject {
    let ptr = SIGNAL_BUF.with(|buf| buf.borrow().as_ptr() as *mut u8);
    // SAFETY: ptr points into a thread_local Vec that lives for the thread's
    // lifetime and is never reallocated (length is fixed at init).
    let bb = match unsafe { env.new_direct_byte_buffer(ptr, SIGNAL_BUFFER_SIZE) } {
        Ok(b) => b,
        Err(_) => return std::ptr::null_mut(),
    };
    JObject::from(bb).into_raw()
}

// ── JNI: drainQueue ───────────────────────────────────────────────────────────

/// Tick due intervals, then write all pending signal updates into the shared
/// `ByteBuffer` and return the number of bytes written (0 = no updates).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_android_BrickJni_drainQueue<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    tick_intervals();

    let entries = crate::portable::drain_signal_queue();
    if entries.is_empty() {
        return 0;
    }

    SIGNAL_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        let mut offset = 0usize;

        for (signal_id, bytes) in &entries {
            let needed = 4 + bytes.len();
            if offset + needed > SIGNAL_BUFFER_SIZE {
                break;
            }
            buf[offset..offset + 4].copy_from_slice(&signal_id.to_le_bytes());
            offset += 4;
            buf[offset..offset + bytes.len()].copy_from_slice(bytes);
            offset += bytes.len();
        }

        offset as jint
    })
}

// ── JNI: dispatch (no-arg click actions) ─────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_android_BrickJni_dispatch<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    method_name: JString<'local>,
) {
    let name: String = match env.get_string(&method_name) {
        Ok(s) => s.into(),
        Err(_) => return,
    };
    ACTION_TABLE.with(|table| {
        if let Some(f) = table.borrow().get(name.as_str()) {
            f();
        }
    });
}

// ── JNI: dispatchString (String-extractor actions) ───────────────────────────

/// Call a registered String-extractor controller method by name, forwarding
/// the text value from the native text field.
///
/// ```kotlin
/// BrickJni.dispatchString("set_name", textFieldValue)
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_android_BrickJni_dispatchString<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    method_name: JString<'local>,
    value: JString<'local>,
) {
    let name: String = match env.get_string(&method_name) {
        Ok(s) => s.into(),
        Err(_) => return,
    };
    let val: String = match env.get_string(&value) {
        Ok(s) => s.into(),
        Err(_) => return,
    };
    STRING_ACTION_TABLE.with(|table| {
        if let Some(f) = table.borrow().get(name.as_str()) {
            f(val);
        }
    });
}

// ── JNI: dispatchInt (index-based actions) ────────────────────────────────────

/// Call a registered int-extractor controller method by name, forwarding
/// the index value (e.g. list item position) from the native layer.
///
/// ```kotlin
/// BrickJni.dispatchInt("remove_item", itemIndex)
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_android_BrickJni_dispatchInt<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    method_name: JString<'local>,
    value: jni::sys::jint,
) {
    let name: String = match env.get_string(&method_name) {
        Ok(s) => s.into(),
        Err(_) => return,
    };
    INT_ACTION_TABLE.with(|table| {
        if let Some(f) = table.borrow().get(name.as_str()) {
            f(value);
        }
    });
}

// ── JNI: dispatchBool (bool-extractor actions) ────────────────────────────────

/// Call a registered bool-extractor controller method by name.
/// Used by toggle/checkbox change handlers and permission result callbacks.
///
/// ```kotlin
/// BrickJni.dispatchBool("on_dark_mode_changed", isChecked)
/// BrickJni.dispatchBool("on_camera_granted", isGranted)
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_android_BrickJni_dispatchBool<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    method_name: JString<'local>,
    value: jni::sys::jboolean,
) {
    let name: String = match env.get_string(&method_name) {
        Ok(s) => s.into(),
        Err(_) => return,
    };
    BOOL_ACTION_TABLE.with(|table| {
        if let Some(f) = table.borrow().get(name.as_str()) {
            f(value != 0);
        }
    });
}

// ── JNI: clearAndroidState ────────────────────────────────────────────────────

/// Reset all thread-local state so a different demo can be loaded cleanly.
/// Called by Kotlin before switching demos.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_brick_android_BrickJni_clearAndroidState<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    ACTION_TABLE.with(|t| t.borrow_mut().clear());
    STRING_ACTION_TABLE.with(|t| t.borrow_mut().clear());
    INT_ACTION_TABLE.with(|t| t.borrow_mut().clear());
    BOOL_ACTION_TABLE.with(|t| t.borrow_mut().clear());
    INTERVAL_TABLE.with(|t| t.borrow_mut().clear());
    crate::portable::SIGNAL_QUEUE.with(|q| q.borrow_mut().clear());
    crate::portable::reset_signal_counter();
}

// ── Action registration ───────────────────────────────────────────────────────

thread_local! {
    static ACTION_TABLE: RefCell<HashMap<&'static str, Box<dyn Fn()>>> =
        RefCell::new(HashMap::new());
    static STRING_ACTION_TABLE: RefCell<HashMap<&'static str, Box<dyn Fn(String)>>> =
        RefCell::new(HashMap::new());
    static INT_ACTION_TABLE: RefCell<HashMap<&'static str, Box<dyn Fn(i32)>>> =
        RefCell::new(HashMap::new());
    static BOOL_ACTION_TABLE: RefCell<HashMap<&'static str, Box<dyn Fn(bool)>>> =
        RefCell::new(HashMap::new());
}

/// Register a no-arg controller method so `dispatch` can call it by name.
pub fn register_action(name: &'static str, f: impl Fn() + 'static) {
    ACTION_TABLE.with(|table| {
        table.borrow_mut().insert(name, Box::new(f));
    });
}

/// Register a String-value controller method so `dispatchString` can call it.
pub fn register_string_action(name: &'static str, f: impl Fn(String) + 'static) {
    STRING_ACTION_TABLE.with(|table| {
        table.borrow_mut().insert(name, Box::new(f));
    });
}

/// Register an i32-value controller method so `dispatchInt` can call it.
pub fn register_int_action(name: &'static str, f: impl Fn(i32) + 'static) {
    INT_ACTION_TABLE.with(|table| {
        table.borrow_mut().insert(name, Box::new(f));
    });
}

/// Register a bool-value controller method so `dispatchBool` can call it.
pub fn register_bool_action(name: &'static str, f: impl Fn(bool) + 'static) {
    BOOL_ACTION_TABLE.with(|table| {
        table.borrow_mut().insert(name, Box::new(f));
    });
}

// ── Interval registration ─────────────────────────────────────────────────────

struct IntervalEntry {
    interval_ms: u64,
    last_fired: Instant,
    callback: Box<dyn Fn()>,
}

thread_local! {
    static INTERVAL_TABLE: RefCell<Vec<IntervalEntry>> = RefCell::new(Vec::new());
}

/// Register a recurring callback to be ticked inside `drainQueue`.
pub fn register_interval(interval_ms: u64, f: impl Fn() + 'static) {
    INTERVAL_TABLE.with(|table| {
        table.borrow_mut().push(IntervalEntry {
            interval_ms,
            last_fired: Instant::now(),
            callback: Box::new(f),
        });
    });
}

fn tick_intervals() {
    INTERVAL_TABLE.with(|table| {
        let now = Instant::now();
        for entry in table.borrow_mut().iter_mut() {
            if entry.last_fired.elapsed().as_millis() as u64 >= entry.interval_ms {
                (entry.callback)();
                entry.last_fired = now;
            }
        }
    });
}
