package com.brick.android

import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.nio.ByteBuffer

/**
 * Manages the per-signal Compose state slots and the drain-queue loop.
 *
 * ## Setup
 * ```kotlin
 * val signalBuf = BrickJni.setupSignalBuffer()
 *     .apply { order(java.nio.ByteOrder.LITTLE_ENDIAN) }
 * val bridge = BrickSignalBridge(lifecycleScope, signalBuf)
 * bridge.onRerender = { /* re-fetch blueprint and swap */ }
 * ```
 *
 * ## Wire format
 * Each entry: `[signal_id: u32 LE][WIRE_TAG: u8][value_bytes...]`
 * `signal_id = 0` is a re-render sentinel — [onRerender] is invoked and the
 * loop continues without consuming further bytes for that entry.
 */
class BrickSignalBridge(
    private val scope: CoroutineScope,
    private val signalBuf: ByteBuffer,
) {

    private val slots = mutableMapOf<UInt, MutableState<String>>()
    @Volatile private var isActive = true

    /**
     * Called on the Main thread when Rust pushes a re-render sentinel
     * (`signal_id = 0`). Set this before calling [startDrainLoop].
     */
    var onRerender: (() -> Unit)? = null

    /**
     * Called when the renderer encounters a `PermissionRequest` button tap.
     * The host Activity sets this to launch `ActivityResultContracts.RequestPermission`
     * and dispatch the result back via [BrickJni.dispatchBool].
     */
    var onPermissionRequest: ((permission: String, resultAction: String) -> Unit)? = null

    /** Register a signal slot with its initial value from the blueprint. */
    fun registerSignal(id: UInt, initialValue: String) {
        slots[id] = mutableStateOf(initialValue)
    }

    /** Stop the drain loop. Call when this bridge is being replaced (demo switch). */
    fun stop() { isActive = false }

    /** Drop all registered signal slots (call before re-registering for a fresh blueprint). */
    fun clearSlots() {
        slots.clear()
    }

    /** Read the current Compose state for a signal.
     *  Returns an empty-string state if the ID was never registered. */
    fun stateFor(id: UInt): MutableState<String> =
        slots.getOrPut(id) { mutableStateOf("") }

    /**
     * Start the per-frame drain loop.
     *
     * Uses [android.view.Choreographer] via a recursive post to align drains
     * with VSYNC, ensuring signal updates reach Compose within the same frame
     * they were produced.
     */
    fun startDrainLoop() {
        val choreographer = android.view.Choreographer.getInstance()

        fun scheduleFrame() {
            if (!isActive) return
            choreographer.postFrameCallback {
                if (isActive) drain()
                scheduleFrame()
            }
        }

        scope.launch(Dispatchers.Main) {
            scheduleFrame()
        }
    }

    /** Drain one batch of signal updates from the Rust queue. */
    private fun drain() {
        val byteCount = BrickJni.drainQueue()
        if (byteCount == 0) return

        signalBuf.position(0).limit(byteCount)
        while (signalBuf.hasRemaining()) {
            if (signalBuf.remaining() < 5) break
            val signalId = signalBuf.int.toUInt()
            val tag = signalBuf.get().toInt() and 0xFF

            // signal_id = 0 is the re-render sentinel; no value bytes follow.
            if (signalId == 0u) {
                onRerender?.invoke()
                continue
            }

            val value = readTypedValue(tag) ?: break
            slots[signalId]?.value = value
        }
        signalBuf.limit(signalBuf.capacity())
    }

    /**
     * Decode one typed value from [signalBuf] given its [tag] byte.
     * Returns `null` on unknown tag or buffer underflow (caller breaks the loop).
     */
    private fun readTypedValue(tag: Int): String? {
        return try {
            when (tag) {
                0x01 -> signalBuf.get().toString()                              // i8
                0x02 -> (signalBuf.get().toInt() and 0xFF).toString()           // u8
                0x03 -> signalBuf.short.toString()                              // i16
                0x04 -> (signalBuf.short.toInt() and 0xFFFF).toString()         // u16
                0x05 -> signalBuf.int.toString()                                // i32
                0x06 -> (signalBuf.int.toLong() and 0xFFFFFFFFL).toString()     // u32
                0x07 -> signalBuf.long.toString()                               // i64
                0x08 -> java.lang.Long.toUnsignedString(signalBuf.long)         // u64
                0x09 -> signalBuf.long.toString()                               // isize (8 bytes)
                0x0A -> java.lang.Long.toUnsignedString(signalBuf.long)         // usize (8 bytes)
                0x0B -> signalBuf.float.toString()                              // f32
                0x0C -> signalBuf.double.toString()                             // f64
                0x0D -> (signalBuf.get() != 0.toByte()).toString()              // bool
                0x0E -> {                                                        // String
                    val len = signalBuf.short.toInt() and 0xFFFF
                    val bytes = ByteArray(len)
                    signalBuf.get(bytes)
                    String(bytes, Charsets.UTF_8)
                }
                else -> null
            }
        } catch (_: Exception) {
            null
        }
    }
}
