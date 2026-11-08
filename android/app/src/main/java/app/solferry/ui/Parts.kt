package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandHorizontally
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import app.solferry.R
import app.solferry.design.MotionTokens
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

/** A device's tile: its platform logo in its own color while it is there, gray while it isn't. */
@Composable
fun DeviceTile(identity: Identity, online: Boolean = true) {
    val palette = LocalPalette.current
    val tone = when (identity.platform) {
        DevicePlatform.MACOS, DevicePlatform.IOS -> palette.platformApple
        DevicePlatform.ANDROID -> palette.platformAndroid
        DevicePlatform.WINDOWS -> palette.platformWindows
        DevicePlatform.LINUX -> palette.platformLinux
        null -> palette.inkMuted
    }
    if (online) Tile(platformIcon(identity.platform), tone.copy(alpha = 0.16f), tone) else Tile(platformIcon(identity.platform), palette.control, palette.inkMuted)
}

@Composable
fun Caption(text: String, modifier: Modifier = Modifier, color: Color = LocalPalette.current.inkMuted) =
    Text(text, modifier, style = MaterialTheme.typography.bodySmall, color = color, maxLines = 1)

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

data class Tab(val label: String, @DrawableRes val icon: Int, val selected: Boolean, val onSelect: () -> Unit)

/** Room scrolling content leaves so nothing ends up under the floating bar. */
@Composable
fun tabClearance() = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding() + 80.dp

/** The icon for a file, from what its name says it is. */
@DrawableRes
fun fileIcon(name: String) = when (name.substringAfterLast('.', "").lowercase()) {
    "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "bmp", "tiff", "svg", "raw" -> R.drawable.ic_type_image
    "mp4", "mov", "mkv", "avi", "webm", "m4v" -> R.drawable.ic_type_video
    "mp3", "wav", "flac", "aac", "ogg", "m4a", "opus" -> R.drawable.ic_type_audio
    "pdf" -> R.drawable.ic_type_pdf
    "doc", "docx", "odt", "rtf", "pages" -> R.drawable.ic_type_doc
    "xls", "xlsx", "ods", "csv", "numbers" -> R.drawable.ic_type_sheet
    "ppt", "pptx", "odp", "key" -> R.drawable.ic_type_slides
    "zip", "rar", "7z", "tar", "gz", "tgz", "xz", "bz2" -> R.drawable.ic_type_archive
    "txt", "md", "log" -> R.drawable.ic_type_text
    "rs", "js", "ts", "tsx", "py", "go", "c", "cpp", "h", "java", "kt", "swift", "html", "css", "json", "toml", "yaml", "yml", "sh" -> R.drawable.ic_type_code
    "apk", "exe", "msi", "dmg", "pkg", "deb", "rpm", "appimage", "ipa" -> R.drawable.ic_type_package
    else -> R.drawable.ic_file
}

/** The app's three places: a floating toolbar where the current one opens up to show its name. */
@Composable
fun TabBar(tabs: List<Tab>, modifier: Modifier = Modifier) {
    val palette = LocalPalette.current
    Row(
        modifier.navigationBarsPadding().padding(bottom = SpaceTokens.md.dp)
            .dropShadow(CircleShape, Shadow(radius = 16.dp, color = Color.Black.copy(alpha = 0.28f), offset = DpOffset(0.dp, 6.dp)))
            .background(palette.raised, CircleShape).padding(6.dp),
        horizontalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        tabs.forEach { tab ->
            val width by animateDpAsState(if (tab.selected) 128.dp else 60.dp, spring(stiffness = Spring.StiffnessMediumLow, dampingRatio = 0.8f))
            val fill by animateColorAsState(if (tab.selected) palette.primaryFill else Color.Transparent, tween(MotionTokens.standard))
            val tone by animateColorAsState(if (tab.selected) palette.onPrimary else palette.inkMuted, tween(MotionTokens.standard))
            Row(
                Modifier.width(width).height(52.dp).clip(CircleShape).background(fill).selectable(tab.selected, role = Role.Tab, onClick = tab.onSelect),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.Center,
            ) {
                Glyph(tab.icon, tone, 22, if (tab.selected) null else tab.label)
                AnimatedVisibility(tab.selected, enter = fadeIn() + expandHorizontally(), exit = fadeOut() + shrinkHorizontally()) {
                    Text(tab.label, Modifier.padding(start = SpaceTokens.sm.dp), style = MaterialTheme.typography.labelLarge, color = tone, maxLines = 1)
                }
            }
        }
    }
}

/** A pushed screen: a back button, a title, and cards that scroll clear of the keyboard. */
@Composable
fun Page(title: String, onBack: (() -> Unit)?, content: @Composable ColumnScope.() -> Unit) = Column(Modifier.fillMaxSize()) {
    Toolbar {
        if (onBack != null) Button(onBack, icon = R.drawable.ic_arrow_left, kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Back")
        Text(title, Modifier.padding(start = if (onBack == null) SpaceTokens.sm.dp else 0.dp), style = MaterialTheme.typography.titleMedium)
    }
    Column(
        Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState())
            .imePadding().then(if (onBack != null) Modifier.navigationBarsPadding() else Modifier.padding(bottom = tabClearance())).padding(horizontal = SpaceTokens.md.dp, vertical = SpaceTokens.xs.dp),
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

/** How long ago a moment was, in the largest unit that still reads naturally. */
fun ago(ms: Long): String {
    val minutes = (System.currentTimeMillis() - ms).coerceAtLeast(0) / 60_000
    return when {
        minutes < 1 -> "just now"
        minutes < 60 -> "${minutes}m ago"
        minutes < 1440 -> "${minutes / 60}h ago"
        minutes < 2880 -> "yesterday"
        minutes < 43_200 -> "${minutes / 1440}d ago"
        else -> "${minutes / 43_200}mo ago"
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
