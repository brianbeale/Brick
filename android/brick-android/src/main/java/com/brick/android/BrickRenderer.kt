package com.brick.android

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.ui.Alignment
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.BorderStroke
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.ui.graphics.Color
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import com.brick.proto.ProtoNode

// BrickNode is intentionally recursive: each ProtoNode maps to a Compose widget.
// The top-level modifier (padding, fillMaxSize, etc.) is forwarded to the root
// container — Column, Row, or Scroll. Leaf nodes (Text, Button, Input) ignore it.

/**
 * Renders a `BrickViewBlueprint` proto tree as Jetpack Compose UI.
 *
 * Each `ProtoNode` variant maps to a Material 3 composable:
 *   Text    → [Text]
 *   Button  → [Button]
 *   Input   → [OutlinedTextField]
 *   Column  → [Column]
 *   Row     → [Row]
 *   Scroll  → [Column] with [verticalScroll]
 *
 * Reactive signal values are read from [BrickSignalBridge.stateFor]; the bridge
 * pushes updates from the Rust drain queue every Compose frame, so reactive
 * nodes recompose automatically when their signal changes.
 *
 * Actions are dispatched back to Rust via [BrickJni.dispatch].
 */
@Composable
fun BrickNode(node: ProtoNode, bridge: BrickSignalBridge, modifier: Modifier = Modifier) {
    when (node.kindCase) {

        ProtoNode.KindCase.TEXT -> {
            val proto = node.text
            val text = if (proto.signalId != 0) {
                val state by bridge.stateFor(proto.signalId.toUInt())
                state
            } else {
                proto.staticText
            }
            val style = when (proto.textVariant) {
                1 -> MaterialTheme.typography.titleLarge
                2 -> MaterialTheme.typography.bodySmall
                3 -> MaterialTheme.typography.labelMedium
                else -> MaterialTheme.typography.bodyMedium
            }
            Text(text = text, style = style, color = MaterialTheme.colorScheme.onSurface)
        }

        ProtoNode.KindCase.BUTTON -> {
            val proto = node.button
            val onClick: () -> Unit = {
                when {
                    proto.intAction.isNotEmpty() -> BrickJni.dispatchInt(proto.intAction, proto.payloadInt)
                    proto.action.isNotEmpty() -> BrickJni.dispatch(proto.action)
                }
            }
            when (proto.variant) {
                1 -> FilledTonalButton(onClick = onClick) { Text(proto.label) }
                2 -> OutlinedButton(onClick = onClick) { Text(proto.label) }
                3 -> TextButton(onClick = onClick) { Text(proto.label) }
                4 -> Button(
                    onClick = onClick,
                    colors = ButtonDefaults.buttonColors(containerColor = Color(0xFFBB0033)),
                ) { Text(proto.label) }
                5 -> OutlinedButton(
                    onClick = onClick,
                    border = BorderStroke(2.dp, Color(0xFFBB0033)),
                    colors = ButtonDefaults.outlinedButtonColors(contentColor = Color(0xFFBB0033)),
                ) { Text(proto.label) }
                else -> Button(onClick = onClick) { Text(proto.label) }
            }
        }

        ProtoNode.KindCase.INPUT -> {
            val proto = node.input
            // Use the bridge slot as the controlled value so Compose recomposes
            // reactively when Rust pushes a signal update.
            val valueState = bridge.stateFor(proto.valueSignalId.toUInt())
            OutlinedTextField(
                value = valueState.value,
                onValueChange = { newText ->
                    // Update immediately for smooth typing, then tell Rust.
                    valueState.value = newText
                    if (proto.changeAction.isNotEmpty()) {
                        BrickJni.dispatchString(proto.changeAction, newText)
                    }
                },
                placeholder = { if (proto.placeholder.isNotEmpty()) Text(proto.placeholder) },
            )
        }

        ProtoNode.KindCase.COLUMN -> {
            Column(
                modifier = modifier,
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                for (child in node.column.childrenList) {
                    BrickNode(child, bridge)
                }
            }
        }

        ProtoNode.KindCase.ROW -> {
            Row(
                modifier = modifier,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                for (child in node.row.childrenList) {
                    BrickNode(child, bridge)
                }
            }
        }

        ProtoNode.KindCase.SCROLL -> {
            val scrollState = rememberScrollState()
            Column(
                modifier = modifier.verticalScroll(scrollState),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                for (child in node.scroll.childrenList) {
                    BrickNode(child, bridge)
                }
            }
        }

        ProtoNode.KindCase.TOGGLE -> {
            val proto = node.toggle
            val checkedState = if (proto.checkedSignalId != 0) {
                bridge.stateFor(proto.checkedSignalId.toUInt())
            } else {
                null
            }
            val checked = checkedState?.value?.toBooleanStrictOrNull() ?: false
            Row {
                Switch(
                    checked = checked,
                    onCheckedChange = { newChecked ->
                        checkedState?.value = newChecked.toString()
                        if (proto.changeAction.isNotEmpty()) {
                            BrickJni.dispatchBool(proto.changeAction, newChecked)
                        }
                    }
                )
                Text(proto.label)
            }
        }

        ProtoNode.KindCase.EXTERNAL_INTENT -> {
            val proto = node.externalIntent
            val context = LocalContext.current
            Button(onClick = {
                when (proto.intentType) {
                    0 -> { // ShareText
                        val intent = Intent(Intent.ACTION_SEND).apply {
                            type = "text/plain"
                            putExtra(Intent.EXTRA_TEXT, proto.primaryPayload)
                        }
                        context.startActivity(Intent.createChooser(intent, null))
                    }
                    1 -> { // OpenUrl
                        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(proto.primaryPayload))
                        context.startActivity(intent)
                    }
                    4 -> { // OpenSettings
                        val intent = Intent(android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
                            data = Uri.fromParts("package", context.packageName, null)
                        }
                        context.startActivity(intent)
                    }
                    5 -> { // ComposeEmail
                        val intent = Intent(Intent.ACTION_SENDTO, Uri.parse("mailto:${proto.primaryPayload}")).apply {
                            putExtra(Intent.EXTRA_SUBJECT, proto.secondaryPayload)
                        }
                        context.startActivity(intent)
                    }
                    // LaunchCamera (2) and PickImage (3) require ActivityResultContracts;
                    // host Activity wires these via bridge.onPermissionRequest pattern.
                    else -> {}
                }
            }) {
                Text(proto.label)
            }
        }

        ProtoNode.KindCase.PERMISSION_REQUEST -> {
            val proto = node.permissionRequest
            Button(onClick = {
                bridge.onPermissionRequest?.invoke(proto.permission, proto.resultAction)
            }) {
                Text(proto.label)
            }
        }

        else -> { /* unknown node type — render nothing */ }
    }
}
