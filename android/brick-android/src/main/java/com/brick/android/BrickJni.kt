package com.brick.android

/**
 * JNI surface into the Brick Rust library.
 *
 * Load the library once at app startup:
 * ```kotlin
 * BrickJni.load()
 * ```
 *
 * The matching Rust entry points live in `src/native/android.rs`.
 */
object BrickJni {

    fun load() {
        System.loadLibrary("brick")
    }

    /**
     * Allocate the shared signal buffer on the Rust side and return it as a
     * direct [java.nio.ByteBuffer].
     *
     * Call this once after [load], then set [java.nio.ByteOrder.LITTLE_ENDIAN]
     * on the returned buffer and pass it to [BrickSignalBridge].
     */
    external fun setupSignalBuffer(): java.nio.ByteBuffer

    /**
     * Write all pending signal updates into the shared [ByteBuffer] and return
     * the number of bytes written (0 = no updates).
     *
     * Wire format per entry: `[signal_id: u32 LE][WIRE_TAG: u8][value_bytes...]`
     * `signal_id = 0` is a re-render sentinel (no value bytes follow).
     */
    external fun drainQueue(): Int

    /** Call a registered no-arg controller method by name. */
    external fun dispatch(methodName: String)

    /**
     * Call a registered String-extractor controller method by name, forwarding
     * the text value from the native text field.
     */
    external fun dispatchString(methodName: String, value: String)

    /**
     * Call a registered i32-extractor controller method by name.
     * Used for per-item index dispatch (e.g. delete item at position).
     */
    external fun dispatchInt(methodName: String, value: Int)

    /**
     * Call a registered bool-extractor controller method by name.
     * Used for toggle/checkbox change handlers and permission result callbacks.
     */
    external fun dispatchBool(methodName: String, value: Boolean)

    /**
     * Reset all Rust-side thread-local state (action table, interval table,
     * signal queue, signal counter). Call before switching demos.
     */
    external fun clearAndroidState()
}
