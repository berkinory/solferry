package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.solferry.R
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
    tint: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    size: Int = 20,
    label: String? = null,
) = Icon(painterResource(icon), contentDescription = label, tint = tint, modifier = Modifier.size(size.dp))

/** A device's tile: its platform logo in the color its key gives it, so copied names still differ. */
@Composable
fun DeviceTile(identity: Identity, selected: Boolean = false) {
    val palette = LocalPalette.current
    val tone = palette.identity[identity.tone.toInt() % palette.identity.size]
    Box(
        Modifier.size(40.dp).clip(RoundedCornerShape(12.dp))
            .background(if (selected) palette.primaryFill else tone.copy(alpha = 0.16f)),
        contentAlignment = Alignment.Center,
    ) {
        if (selected) Glyph(R.drawable.ic_check, palette.onPrimary) else Glyph(platformIcon(identity.platform), tone)
    }
}

@Composable
fun Mono(text: String, color: Color = MaterialTheme.colorScheme.onSurfaceVariant) =
    Text(text, fontFamily = FontFamily.Monospace, fontSize = 12.sp, color = color)

@Composable
fun SectionLabel(text: String, trailing: (@Composable RowScope.() -> Unit)? = null) = Row(
    Modifier.fillMaxWidth().padding(start = 28.dp, end = 16.dp, top = 20.dp, bottom = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
) {
    Text(text, style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    Spacer(Modifier.weight(1f))
    trailing?.invoke(this)
}

/** Android's grouped list: rows a hair apart, the group rounded at its ends. */
@Composable
fun Segmented(count: Int, row: @Composable (index: Int, shape: RoundedCornerShape) -> Unit) = Column(
    Modifier.padding(horizontal = 12.dp),
    verticalArrangement = Arrangement.spacedBy(2.dp),
) {
    for (index in 0 until count) {
        val top = if (index == 0) 20.dp else 6.dp
        val bottom = if (index == count - 1) 20.dp else 6.dp
        row(index, RoundedCornerShape(top, top, bottom, bottom))
    }
}

@Composable
fun SegmentRow(
    shape: RoundedCornerShape,
    onClick: (() -> Unit)? = null,
    background: Color = MaterialTheme.colorScheme.surfaceContainer,
    content: @Composable RowScope.() -> Unit,
) = Row(
    Modifier.fillMaxWidth().clip(shape).background(background)
        .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
        .padding(horizontal = 14.dp, vertical = 12.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(14.dp),
    content = content,
)

/** A card that wants a decision or reports something, tinted by what it is about. */
@Composable
fun Notice(@DrawableRes icon: Int, title: String, tone: Color, ground: Color, content: @Composable ColumnScope.() -> Unit) =
    Column(
        Modifier.padding(horizontal = 12.dp, vertical = 4.dp).fillMaxWidth().clip(RoundedCornerShape(20.dp))
            .background(ground).padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Glyph(icon, tone, 18)
            Spacer(Modifier.width(8.dp))
            Text(title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Medium, color = tone)
        }
        content()
    }

@Composable
fun StatusLine(text: String, dot: Color?) = Row(verticalAlignment = Alignment.CenterVertically) {
    if (dot != null) {
        Box(Modifier.size(6.dp).clip(CircleShape).background(dot))
        Spacer(Modifier.width(6.dp))
    }
    Text(text, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
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
