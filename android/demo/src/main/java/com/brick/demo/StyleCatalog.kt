package com.brick.demo

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

private val DANGER = Color(0xFFBB0033)

// ── Entry point ───────────────────────────────────────────────────────────────

@Composable
fun StyleCatalog() {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(28.dp),
    ) {
        Text("Style Catalog", style = MaterialTheme.typography.headlineMedium,
            color = MaterialTheme.colorScheme.onSurface)

        ColorSchemeSection()
        TypographySection()
        ButtonSection()
        InputSection()
    }
}

// ── Color scheme ──────────────────────────────────────────────────────────────

@Composable
private fun ColorSchemeSection() {
    CatalogSection("Color Scheme") {
        val cs = MaterialTheme.colorScheme
        val groups = listOf(
            "Primary" to listOf(
                Triple("primary",            cs.primary,            cs.onPrimary),
                Triple("onPrimary",          cs.onPrimary,          cs.primary),
                Triple("primaryContainer",   cs.primaryContainer,   cs.onPrimaryContainer),
                Triple("onPrimaryContainer", cs.onPrimaryContainer, cs.primaryContainer),
            ),
            "Secondary" to listOf(
                Triple("secondary",             cs.secondary,             cs.onSecondary),
                Triple("onSecondary",           cs.onSecondary,           cs.secondary),
                Triple("secondaryContainer",    cs.secondaryContainer,    cs.onSecondaryContainer),
                Triple("onSecondaryContainer",  cs.onSecondaryContainer,  cs.secondaryContainer),
            ),
            "Tertiary" to listOf(
                Triple("tertiary",           cs.tertiary,           cs.onTertiary),
                Triple("tertiaryContainer",  cs.tertiaryContainer,  cs.onTertiaryContainer),
            ),
            "Error" to listOf(
                Triple("error",              cs.error,              cs.onError),
                Triple("onError",            cs.onError,            cs.error),
                Triple("errorContainer",     cs.errorContainer,     cs.onErrorContainer),
                Triple("onErrorContainer",   cs.onErrorContainer,   cs.errorContainer),
            ),
            "Surface" to listOf(
                Triple("background",         cs.background,         cs.onBackground),
                Triple("onBackground",       cs.onBackground,       cs.background),
                Triple("surface",            cs.surface,            cs.onSurface),
                Triple("onSurface",          cs.onSurface,          cs.surface),
                Triple("surfaceVariant",     cs.surfaceVariant,     cs.onSurfaceVariant),
                Triple("onSurfaceVariant",   cs.onSurfaceVariant,   cs.surfaceVariant),
                Triple("outline",            cs.outline,            cs.surface),
                Triple("outlineVariant",     cs.outlineVariant,     cs.onSurface),
            ),
        )
        for ((groupName, entries) in groups) {
            Text(groupName, style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(top = 8.dp))
            entries.chunked(2).forEach { row ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    row.forEach { (name, color, onColor) ->
                        ColorSwatch(name, color, onColor, Modifier.weight(1f))
                    }
                    if (row.size == 1) Spacer(Modifier.weight(1f))
                }
            }
        }
    }
}

@Composable
private fun ColorSwatch(name: String, color: Color, onColor: Color, modifier: Modifier = Modifier) {
    Row(
        modifier = modifier.padding(vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(
            modifier = Modifier
                .size(36.dp)
                .background(color, RoundedCornerShape(4.dp))
                .border(1.dp, MaterialTheme.colorScheme.outlineVariant, RoundedCornerShape(4.dp)),
            contentAlignment = Alignment.Center,
        ) {
            Text("Aa", color = onColor, style = MaterialTheme.typography.labelSmall)
        }
        Text(name, style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface, maxLines = 2)
    }
}

// ── Typography ────────────────────────────────────────────────────────────────

@Composable
private fun TypographySection() {
    CatalogSection("Typography") {
        val t = MaterialTheme.typography
        listOf(
            "displayLarge"   to t.displayLarge,
            "displayMedium"  to t.displayMedium,
            "displaySmall"   to t.displaySmall,
            "headlineLarge"  to t.headlineLarge,
            "headlineMedium" to t.headlineMedium,
            "headlineSmall"  to t.headlineSmall,
            "titleLarge"     to t.titleLarge,
            "titleMedium"    to t.titleMedium,
            "titleSmall"     to t.titleSmall,
            "bodyLarge"      to t.bodyLarge,
            "bodyMedium"     to t.bodyMedium,
            "bodySmall"      to t.bodySmall,
            "labelLarge"     to t.labelLarge,
            "labelMedium"    to t.labelMedium,
            "labelSmall"     to t.labelSmall,
        ).forEach { (name, style) ->
            TypographySample(name, style)
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
        }
    }
}

@Composable
private fun TypographySample(name: String, style: TextStyle) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("Brick", style = style, color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.width(120.dp))
        Text(name, style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

// ── Buttons ───────────────────────────────────────────────────────────────────

@Composable
private fun ButtonSection() {
    CatalogSection("Buttons") {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = {}) { Text("Primary") }
            FilledTonalButton(onClick = {}) { Text("Secondary") }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedButton(onClick = {}) { Text("Outlined") }
            TextButton(onClick = {}) { Text("Ghost") }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(
                onClick = {},
                colors = ButtonDefaults.buttonColors(containerColor = DANGER),
            ) { Text("Danger") }
            OutlinedButton(
                onClick = {},
                border = BorderStroke(2.dp, DANGER),
                colors = ButtonDefaults.outlinedButtonColors(contentColor = DANGER),
            ) { Text("Danger Alt") }
        }
        Spacer(Modifier.height(4.dp))
        Text("Disabled states", style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = {}, enabled = false) { Text("Primary") }
            OutlinedButton(onClick = {}, enabled = false) { Text("Outlined") }
            TextButton(onClick = {}, enabled = false) { Text("Ghost") }
        }
    }
}

// ── Inputs & controls ─────────────────────────────────────────────────────────

@Composable
private fun InputSection() {
    CatalogSection("Inputs & Controls") {
        OutlinedTextField(value = "", onValueChange = {},
            placeholder = { Text("Placeholder text") },
            label = { Text("Label") },
            modifier = Modifier.fillMaxWidth())
        OutlinedTextField(value = "Filled value", onValueChange = {},
            label = { Text("Label") },
            modifier = Modifier.fillMaxWidth())
        Row(verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Switch(checked = false, onCheckedChange = {})
            Text("Toggle off", style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface)
        }
        Row(verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Switch(checked = true, onCheckedChange = {})
            Text("Toggle on", style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface)
        }
    }
}

// ── Section wrapper ───────────────────────────────────────────────────────────

@Composable
private fun CatalogSection(title: String, content: @Composable ColumnScope.() -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(title, style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.SemiBold)
        HorizontalDivider(color = MaterialTheme.colorScheme.outline)
        content()
    }
}
