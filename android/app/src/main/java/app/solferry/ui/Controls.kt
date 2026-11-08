package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp

enum class ButtonKind { Primary, Secondary, Ghost, Danger }

enum class ButtonSize { Sm, Md }

/** Material's buttons, wearing the design system's colors and corners. */
@Composable
fun Button(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    label: String? = null,
    @DrawableRes icon: Int? = null,
    kind: ButtonKind = ButtonKind.Secondary,
    size: ButtonSize = ButtonSize.Md,
    enabled: Boolean = true,
    description: String? = null,
) {
    val palette = LocalPalette.current
    val height = if (size == ButtonSize.Md) 44.dp else 36.dp
    val shape = if (size == ButtonSize.Md) ControlShape else FloorShape
    if (label == null) {
        IconButton(onClick, modifier.size(height), enabled) {
            if (icon != null) Glyph(icon, palette.inkMuted, 18, description)
        }
        return
    }
    val content: @Composable RowScope.(Color) -> Unit = { tone ->
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            if (icon != null) Glyph(icon, tone, 18)
            Text(label, style = MaterialTheme.typography.labelLarge, maxLines = 1)
        }
    }
    val sized = modifier.height(height).defaultMinSize(minWidth = 1.dp)
    val padding = PaddingValues(horizontal = 14.dp)
    when (kind) {
        ButtonKind.Primary -> androidx.compose.material3.Button(
            onClick, sized, enabled, shape,
            ButtonDefaults.buttonColors(containerColor = palette.primaryFill, contentColor = palette.onPrimary),
            contentPadding = padding,
        ) { content(palette.onPrimary) }
        ButtonKind.Danger -> androidx.compose.material3.Button(
            onClick, sized, enabled, shape,
            ButtonDefaults.buttonColors(containerColor = palette.dangerFill, contentColor = palette.onPrimary),
            contentPadding = padding,
        ) { content(palette.onPrimary) }
        ButtonKind.Secondary -> OutlinedButton(
            onClick, sized, enabled, shape,
            ButtonDefaults.outlinedButtonColors(contentColor = palette.ink),
            border = BorderStroke(1.dp, palette.line),
            contentPadding = padding,
        ) { content(palette.ink) }
        ButtonKind.Ghost -> TextButton(
            onClick, sized, enabled, shape,
            ButtonDefaults.textButtonColors(contentColor = palette.inkMuted),
            contentPadding = padding,
        ) { content(palette.inkMuted) }
    }
}

@Composable
fun Toggle(checked: Boolean, onChange: (Boolean) -> Unit, label: String) {
    val palette = LocalPalette.current
    val haptics = LocalHapticFeedback.current
    Switch(
        checked,
        {
            haptics.performHapticFeedback(if (it) HapticFeedbackType.ToggleOn else HapticFeedbackType.ToggleOff)
            onChange(it)
        },
        Modifier.semantics { contentDescription = label },
        colors = SwitchDefaults.colors(
            checkedTrackColor = palette.primaryFill, checkedThumbColor = palette.onPrimary,
            uncheckedTrackColor = palette.control, uncheckedThumbColor = palette.inkMuted, uncheckedBorderColor = palette.line,
        ),
    )
}

@Composable
fun Field(
    value: String,
    onChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    keyboardActions: KeyboardActions = KeyboardActions.Default,
    trailing: (@Composable RowScope.() -> Unit)? = null,
) {
    val palette = LocalPalette.current
    OutlinedTextField(
        value, onChange, modifier,
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyLarge,
        placeholder = { Text(placeholder, maxLines = 1) },
        trailingIcon = trailing?.let { { Row(verticalAlignment = Alignment.CenterVertically, content = it) } },
        keyboardOptions = keyboardOptions,
        keyboardActions = keyboardActions,
        shape = ControlShape,
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = palette.primary, unfocusedBorderColor = palette.line, cursorColor = palette.primary,
            focusedContainerColor = palette.page, unfocusedContainerColor = palette.page,
            focusedPlaceholderColor = palette.inkMuted, unfocusedPlaceholderColor = palette.inkMuted,
        ),
    )
}

@Composable
fun ProgressBar(fraction: Float, tone: Color) = LinearProgressIndicator(
    progress = { fraction.coerceIn(0f, 1f) },
    Modifier.fillMaxWidth(),
    color = tone,
    trackColor = LocalPalette.current.control,
    drawStopIndicator = {},
)
