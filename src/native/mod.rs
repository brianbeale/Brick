/// JNI bridge between Brick's Rust layer and the Kotlin/Android host.
///
/// Exported functions (called by Kotlin):
///   `brick_build_blueprint(view_ptr)`  — encode a PortableView tree → prost bytes → jbyteArray
///   `brick_drain_queue()`              — drain pending signal updates → prost bytes → jbyteArray
///   `brick_dispatch_action(name)`      — call a registered controller method by name
///
/// This module compiles only when `--cfg brick_android` is set (Android targets).
/// On all other targets it is an empty module so `pub mod native;` in lib.rs compiles cleanly.
#[cfg(brick_android)]
mod android;

#[cfg(brick_android)]
pub use android::*;
