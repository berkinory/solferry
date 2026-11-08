package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.draw.innerShadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import app.solferry.design.MotionTokens

enum class ButtonKind { Primary, Secondary, Ghost, Danger }

enum class ButtonSize { Sm, Md }

private data class Look(val fill: Color, val pressed: Color, val text: Color, val border: Color?)

@Composable
private fun look(kind: ButtonKind): Look = with(LocalPalette.current) {
    when (kind) {
        ButtonKind.Primary -> Look(primaryFill, primaryHover, onPrimary, primaryBorder)
        ButtonKind.Secondary -> Look(page, raised, ink, line)
        ButtonKind.Ghost -> Look(Color.Transparent, raised, inkMuted, null)
        ButtonKind.Danger -> Look(dangerFill, dangerFill.copy(alpha = 0.9f), onPrimary, null)
    }
}

/** The app's one button, like the desktop's: a lit primary, a framed secondary, quiet ghosts. */
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
    val look = look(kind)
    val press = remember { MutableInteractionSource() }
    val pressed by press.collectIsPressedAsState()
    val fill by animateColorAsState(if (pressed) look.pressed else look.fill, tween(MotionTokens.quick))
    val height = if (size == ButtonSize.Md) 44.dp else 36.dp
    val lit = if (kind == ButtonKind.Primary) {
        Modifier
            .innerShadow(ControlShape, Shadow(radius = 0.dp, color = Color.White.copy(alpha = 0.22f), offset = DpOffset(0.dp, 1.dp)))
            .innerShadow(ControlShape, Shadow(radius = 0.dp, color = palette.primaryDeep.copy(alpha = 0.3f), offset = DpOffset(0.dp, (-1).dp)))
    } else {
        Modifier
    }
    val lifted = if (kind == ButtonKind.Secondary) {
        Modifier.dropShadow(ControlShape, Shadow(radius = 2.dp, color = Color.Black.copy(alpha = 0.05f), offset = DpOffset(0.dp, 1.dp)))
    } else {
        Modifier
    }
    val outline = look.border?.let { Modifier.border(1.dp, it, ControlShape) } ?: Modifier
    Row(
        modifier.alpha(if (enabled) 1f else 0.5f).height(height)
            .then(if (label == null) Modifier.size(height) else Modifier)
            .then(lifted).background(fill, ControlShape).then(lit).then(outline)
            .clickable(press, null, enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(horizontal = if (label == null) 0.dp else 14.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally),
    ) {
        if (icon != null) Glyph(icon, look.text, 18, if (label == null) description else null)
        if (label != null) Text(label, style = MaterialTheme.typography.labelLarge, color = look.text, maxLines = 1)
    }
}

@Composable
fun Toggle(checked: Boolean, onChange: (Boolean) -> Unit, label: String) {
    val palette = LocalPalette.current
    val track by animateColorAsState(if (checked) palette.primaryFill else palette.control, tween(MotionTokens.standard))
    val travel by animateDpAsState(if (checked) 20.dp else 0.dp, tween(MotionTokens.standard))
    Box(
        Modifier.size(44.dp, 26.dp).background(track, CircleShape)
            .semantics { contentDescription = label }
            .toggleable(checked, remember { MutableInteractionSource() }, null, role = Role.Switch, onValueChange = onChange)
            .padding(3.dp),
    ) {
        Box(Modifier.offset(x = travel).size(20.dp).background(Color.White, CircleShape))
    }
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
    val focus = remember { MutableInteractionSource() }
    val focused by focus.collectIsFocusedAsState()
    val edge by animateColorAsState(if (focused) palette.primary else palette.line, tween(MotionTokens.quick))
    BasicTextField(
        value, onChange, modifier,
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyLarge.copy(color = palette.ink),
        cursorBrush = SolidColor(palette.primary),
        interactionSource = focus,
        keyboardOptions = keyboardOptions,
        keyboardActions = keyboardActions,
        decorationBox = { input ->
            Row(
                Modifier.height(44.dp).background(palette.page, ControlShape).border(1.dp, edge, ControlShape)
                    .padding(start = 12.dp, end = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                    if (value.isEmpty()) Text(placeholder, color = palette.inkMuted, maxLines = 1)
                    input()
                }
                trailing?.invoke(this)
            }
        },
    )
}

@Composable
fun ProgressBar(fraction: Float, tone: Color) = Box(
    Modifier.fillMaxWidth().height(6.dp).background(LocalPalette.current.control, CircleShape),
) {
    Box(Modifier.fillMaxHeight().fillMaxWidth(fraction.coerceIn(0f, 1f)).background(tone, CircleShape))
}
