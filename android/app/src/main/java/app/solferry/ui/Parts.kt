package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import app.solferry.R
import app.solferry.design.RadiusTokens
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.DevicePlatform
import uniffi.solferry_ffi.Identity

@DrawableRes
fun platformIcon(platform: DevicePlatform?) = when (platform) {
    DevicePlatform.MACOS, DevicePlatform.IOS -> R.drawable.ic_apple
    DevicePlatform.WINDOWS -> R.drawable.ic_windows
    DevicePlatform.LINUX -> R.drawable.ic_linux
    DevicePlatform.ANDROID -> R.drawable.ic_android
    null -> R.drawable.ic_computer
}

@Composable
fun Glyph(
    @DrawableRes icon: Int,
    tint: Color = LocalPalette.current.inkMuted,
    size: Int = 20,
    label: String? = null,
) = Icon(painterResource(icon), contentDescription = label, tint = tint, modifier = Modifier.size(size.dp))

@Composable
fun Tile(@DrawableRes icon: Int, fill: Color, tone: Color) = Box(
    Modifier.size(38.dp).background(fill, RoundedCornerShape(RadiusTokens.floor.dp)),
    contentAlignment = Alignment.Center,
) { Glyph(icon, tone, 18) }

/** A device's tile: its platform logo in the color its key gives it, so copied names still differ. */
@Composable
fun DeviceTile(identity: Identity) {
    val palette = LocalPalette.current
    val tone = palette.identity[identity.tone.toInt() % palette.identity.size]
    Tile(platformIcon(identity.platform), tone.copy(alpha = 0.16f), tone)
}

@Composable
fun Mono(text: String, modifier: Modifier = Modifier, color: Color = LocalPalette.current.inkMuted) =
    Text(text, modifier, fontFamily = MonoFamily, style = MaterialTheme.typography.bodySmall, color = color, maxLines = 1)

@Composable
fun Muted(
    text: String,
    modifier: Modifier = Modifier,
    color: Color = LocalPalette.current.inkMuted,
    textAlign: TextAlign? = null,
) = Text(text, modifier, style = MaterialTheme.typography.bodyMedium, color = color, textAlign = textAlign)

/** Muted text after a colored dot, for anything that has a state. */
@Composable
fun StatusLine(text: String, dot: Color? = null) = Row(verticalAlignment = Alignment.CenterVertically) {
    if (dot != null) {
        Box(Modifier.size(6.dp).background(dot, CircleShape))
        Spacer(Modifier.width(6.dp))
    }
    Text(text, style = MaterialTheme.typography.bodyMedium, color = LocalPalette.current.inkMuted, maxLines = 1)
}

@Composable
fun Toolbar(content: @Composable RowScope.() -> Unit) = Row(
    Modifier.fillMaxWidth().statusBarsPadding().height(52.dp).padding(horizontal = SpaceTokens.sm.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(2.dp),
    content = content,
)

/** A pushed screen: a back button, a title, and cards that scroll clear of the keyboard. */
@Composable
fun Page(title: String, onBack: () -> Unit, content: @Composable ColumnScope.() -> Unit) = Column(Modifier.fillMaxSize()) {
    Toolbar {
        Button(onBack, icon = R.drawable.ic_arrow_left, kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Back")
        Text(title, style = MaterialTheme.typography.titleMedium)
    }
    Column(
        Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState())
            .imePadding().navigationBarsPadding().padding(horizontal = SpaceTokens.md.dp, vertical = SpaceTokens.xs.dp),
        verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp),
        content = content,
    )
}

@Composable
fun ConfirmDialog(title: String, body: String, action: String, onConfirm: () -> Unit, onDismiss: () -> Unit) {
    val palette = LocalPalette.current
    Dialog(onDismiss) {
        Column(
            Modifier.fillMaxWidth().background(palette.page, CardShape).border(1.dp, palette.line, CardShape)
                .padding(SpaceTokens.lg.dp),
            verticalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp),
        ) {
            Text(title, style = MaterialTheme.typography.titleMedium)
            Muted(body)
            Row(
                Modifier.fillMaxWidth().padding(top = SpaceTokens.lg.dp),
                horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp, Alignment.End),
            ) {
                Button(onDismiss, label = "Cancel", size = ButtonSize.Sm)
                Button(onConfirm, label = action, kind = ButtonKind.Danger, size = ButtonSize.Sm)
            }
        }
    }
}

fun bytes(value: Long): String {
    val units = listOf("B", "KB", "MB", "GB", "TB")
    var amount = value.toDouble()
    var unit = 0
    while (amount >= 1024 && unit < units.size - 1) {
        amount /= 1024
        unit++
    }
    return if (unit == 0) "$value B" else "%.1f %s".format(amount, units[unit])
}
