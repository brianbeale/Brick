package com.brick.demo

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.graphics.Color
import com.brick.android.BrickJni
import com.brick.android.BrickNode
import com.brick.android.BrickSignalBridge
import com.brick.proto.BrickViewBlueprint
import java.nio.ByteBuffer
import java.nio.ByteOrder

// ── Brick dark color scheme ───────────────────────────────────────────────────
// Mapped from BRICK_DARK_THEME tokens in src/theme/theme.rs.

private val BrickDarkColors = darkColorScheme(
    primary              = Color(0xFFD64236), // accent
    onPrimary            = Color(0xFFFFFFFF), // accent_fg
    primaryContainer     = Color(0xFF5C221C), // border (dark brick red)
    onPrimaryContainer   = Color(0xFFF0D8D4), // text
    secondary            = Color(0xFFA4746C), // text_muted
    onSecondary          = Color(0xFF1A0806), // bg
    secondaryContainer   = Color(0xFF5C221C), // border — tonal button surface
    onSecondaryContainer = Color(0xFFF0D8D4), // text — tonal button label
    tertiary             = Color(0xFFDA9E2E), // warning
    onTertiary           = Color(0xFF1A0806),
    tertiaryContainer    = Color(0xFF3D2800),
    onTertiaryContainer  = Color(0xFFF0E0B4),
    error                = Color(0xFFBB0033), // danger — medium crimson
    onError              = Color(0xFFFFFFFF),
    errorContainer       = Color(0xFF5C0A0A),
    onErrorContainer     = Color(0xFFFFCDD2),
    background           = Color(0xFF1A0806), // bg
    onBackground         = Color(0xFFF0D8D4), // text
    surface              = Color(0xFF2A0D0A), // surface
    onSurface            = Color(0xFFF0D8D4), // text
    surfaceVariant       = Color(0xFF3A130F), // surface_raised
    onSurfaceVariant     = Color(0xFFA4746C), // text_muted
    outline              = Color(0xFFA4746C), // text_muted — visible on dark bg
    outlineVariant       = Color(0xFF5C221C), // border
    inverseSurface       = Color(0xFFF0D8D4),
    inverseOnSurface     = Color(0xFF2A0D0A),
    inversePrimary       = Color(0xFF9E281A), // accent light
    surfaceTint          = Color(0xFFD64236), // accent
)

/**
 * Phase 14 demo host.
 *
 * Four demos — Counter, Timer, Contact Form, Todo List — selectable via a tab
 * bar at the top. Switching demos calls [BrickJni.clearAndroidState] so Rust's
 * action/interval tables start fresh, then fetches a new blueprint.
 *
 * Re-render tokens (`signal_id = 0`) from the drain queue trigger a fresh
 * blueprint fetch without clearing the model state, allowing structural view
 * changes (form submit → thank-you screen, todo list mutations) to propagate.
 */
class MainActivity : ComponentActivity() {

    private external fun brickDemoBlueprintBytes(): ByteArray
    private external fun brickTimerBlueprintBytes(): ByteArray
    private external fun brickContactFormBlueprintBytes(): ByteArray
    private external fun brickTodoListBlueprintBytes(): ByteArray
    private external fun brickThermometerBlueprintBytes(): ByteArray
    private external fun brickFlightBookerBlueprintBytes(): ByteArray
    private external fun brickDoubleCounterBlueprintBytes(): ByteArray
    private external fun brickTodoMvcBlueprintBytes(): ByteArray
    private external fun brickCrudBlueprintBytes(): ByteArray
    private external fun brickNavDemoBlueprintBytes(): ByteArray
    private external fun brickThemeDemoBlueprintBytes(): ByteArray

    // Permission launcher — wired to the active bridge via onPermissionRequest.
    private var pendingPermissionResultAction: String = ""
    private val permissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted ->
        if (pendingPermissionResultAction.isNotEmpty()) {
            BrickJni.dispatchBool(pendingPermissionResultAction, isGranted)
            pendingPermissionResultAction = ""
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        BrickJni.load()
        val signalBuf = BrickJni.setupSignalBuffer()
            .apply { order(ByteOrder.LITTLE_ENDIAN) }

        setContent {
            MaterialTheme(colorScheme = BrickDarkColors) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    DemoApp(signalBuf, ::fetchBlueprint, ::onPermissionRequest)
                }
            }
        }
    }

    private fun onPermissionRequest(permission: String, resultAction: String) {
        pendingPermissionResultAction = resultAction
        permissionLauncher.launch(permission)
    }

    private fun fetchBlueprint(demo: String): ByteArray? = when (demo) {
        "counter"    -> brickDemoBlueprintBytes()
        "timer"      -> brickTimerBlueprintBytes()
        "form"       -> brickContactFormBlueprintBytes()
        "todos"      -> brickTodoListBlueprintBytes()
        "thermo"     -> brickThermometerBlueprintBytes()
        "flight"     -> brickFlightBookerBlueprintBytes()
        "double"     -> brickDoubleCounterBlueprintBytes()
        "todomvc"    -> brickTodoMvcBlueprintBytes()
        "crud"       -> brickCrudBlueprintBytes()
        "nav"        -> brickNavDemoBlueprintBytes()
        "theme"      -> brickThemeDemoBlueprintBytes()
        else         -> null
    }
}

// ── Demo switcher composable ──────────────────────────────────────────────────

private val DEMOS = listOf(
    "counter" to "Counter", "timer" to "Timer", "form" to "Form", "todos" to "Todos",
    "thermo" to "Thermo", "flight" to "Flight", "double" to "2×Ctr",
    "todomvc" to "TodoMVC", "crud" to "CRUD", "nav" to "Nav", "theme" to "Theme",
    "styles" to "Styles",
)

@Composable
fun DemoApp(
    signalBuf: ByteBuffer,
    fetchBlueprint: (String) -> ByteArray?,
    onPermissionRequest: ((String, String) -> Unit)? = null,
) {
    val scope = rememberCoroutineScope()
    var currentDemo by remember { mutableStateOf("theme") }
    // Incremented by the re-render sentinel; triggers blueprint re-fetch.
    var rerenderTick by remember { mutableIntStateOf(0) }

    // One bridge per demo; recreated when the demo changes.
    val bridge = remember(currentDemo) {
        BrickJni.clearAndroidState()
        BrickSignalBridge(scope, signalBuf).also { b ->
            b.onRerender = { rerenderTick++ }
            b.onPermissionRequest = onPermissionRequest
            b.startDrainLoop()
        }
    }

    // Stop the drain loop when this bridge is replaced (i.e. when currentDemo changes).
    // Without this, old drain loops accumulate and compete for drainQueue() — consuming
    // signal updates before the active bridge can see them.
    DisposableEffect(bridge) {
        onDispose { bridge.stop() }
    }

    // Blueprint re-fetched on demo switch OR re-render tick.
    val blueprint = remember(currentDemo, rerenderTick) {
        val bytes = fetchBlueprint(currentDemo) ?: return@remember null
        val bp = BrickViewBlueprint.parseFrom(bytes)
        bridge.clearSlots()
        for (sig in bp.signalsList) {
            bridge.registerSignal(sig.id.toUInt(), sig.initialValue)
        }
        bp.root?.let { registerSignalsFromTree(it, bridge) }
        bp
    }

    Column(modifier = Modifier.fillMaxSize().statusBarsPadding().padding(top = 8.dp)) {
        // Tab bar — horizontally scrollable to fit all demos
        Row(
            modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.Start,
        ) {
            for ((key, label) in DEMOS) {
                TextButton(onClick = {
                    if (currentDemo != key) {
                        currentDemo = key
                        rerenderTick = 0
                    }
                }) {
                    Text(
                        label,
                        fontWeight = if (currentDemo == key) FontWeight.Bold else FontWeight.Normal,
                    )
                }
            }
        }
        HorizontalDivider()

        // Rendered brick content
        if (currentDemo == "styles") {
            StyleCatalog()
        } else {
            blueprint?.root?.let { root ->
                BrickNode(node = root, bridge = bridge, modifier = Modifier.padding(16.dp))
            }
        }
    }
}

// ── Signal registration helpers ───────────────────────────────────────────────

private fun registerSignalsFromTree(
    node: com.brick.proto.ProtoNode,
    bridge: BrickSignalBridge,
) {
    when {
        node.hasText() && node.text.signalId != 0 ->
            bridge.registerSignal(node.text.signalId.toUInt(), node.text.staticText)

        node.hasInput() && node.input.valueSignalId != 0 ->
            bridge.registerSignal(node.input.valueSignalId.toUInt(), "")

        node.hasToggle() && node.toggle.checkedSignalId != 0 ->
            bridge.registerSignal(node.toggle.checkedSignalId.toUInt(), "false")

        node.hasColumn() ->
            node.column.childrenList.forEach { registerSignalsFromTree(it, bridge) }

        node.hasRow() ->
            node.row.childrenList.forEach { registerSignalsFromTree(it, bridge) }

        node.hasScroll() ->
            node.scroll.childrenList.forEach { registerSignalsFromTree(it, bridge) }
    }
}
