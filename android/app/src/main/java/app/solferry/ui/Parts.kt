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
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FloatingToolbarDefaults
import androidx.compose.material3.HorizontalFloatingToolbar
import androidx.compose.material3.SheetValue
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.solferry.R
import app.solferry.design.MotionTokens
import app.solferry.design.RadiusTokens
import app.solferry.design.SpaceTokens
import kotlinx.coroutines.launch
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

/** A device's tile: filled with its platform's color while it is there, flat gray while it isn't. */
@Composable
fun DeviceTile(identity: Identity, online: Boolean = true) {
    val palette = LocalPalette.current
    val brand = when (identity.platform) {
        DevicePlatform.MACOS, DevicePlatform.IOS -> palette.platformApple
        DevicePlatform.ANDROID -> palette.platformAndroid
        DevicePlatform.WINDOWS -> palette.platformWindows
        DevicePlatform.LINUX -> palette.platformLinux
        null -> null
    }
    val icon = platformIcon(identity.platform)
    if (brand != null && online) Tile(icon, brand, palette.ink) else Tile(icon, palette.control, palette.inkMuted.copy(alpha = if (brand == null) 1f else 0.6f))
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

/** Room scrolling content leaves so nothing ends up under the floating toolbar. */
@Composable
fun tabClearance() = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding() + 92.dp

/** Material's floating toolbar, where the current place opens up to show its name. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun TabBar(tabs: List<Tab>, modifier: Modifier = Modifier) {
    val palette = LocalPalette.current
    val haptics = LocalHapticFeedback.current
    HorizontalFloatingToolbar(
        expanded = true,
        modifier = modifier.navigationBarsPadding().padding(bottom = SpaceTokens.md.dp),
        colors = FloatingToolbarDefaults.standardFloatingToolbarColors(toolbarContainerColor = palette.raised, toolbarContentColor = palette.inkMuted),
    ) {
        tabs.forEach { tab ->
            val width by animateDpAsState(if (tab.selected) 124.dp else 56.dp, spring(stiffness = Spring.StiffnessMediumLow, dampingRatio = 0.8f))
            val fill by animateColorAsState(if (tab.selected) palette.primaryFill else Color.Transparent, tween(MotionTokens.standard))
            val tone by animateColorAsState(if (tab.selected) palette.onPrimary else palette.inkMuted, tween(MotionTokens.standard))
            Row(
                Modifier.width(width).height(48.dp).clip(CircleShape).background(fill).selectable(tab.selected, role = Role.Tab) {
                    if (!tab.selected) haptics.performHapticFeedback(HapticFeedbackType.SegmentTick)
                    tab.onSelect()
                },
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

/** A top-level screen: a title and cards that scroll clear of the keyboard. */
@Composable
fun Page(title: String, content: @Composable ColumnScope.() -> Unit) = Column(Modifier.fillMaxSize()) {
    Toolbar {
        Text(title, Modifier.padding(start = SpaceTokens.sm.dp), style = MaterialTheme.typography.titleMedium)
    }
    Column(
        Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState())
            .imePadding().padding(bottom = tabClearance()).padding(horizontal = SpaceTokens.md.dp, vertical = SpaceTokens.xs.dp),
        verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp),
        content = content,
    )
}

/** Draws the toasts inside a sheet, which would otherwise dim the ones on the screen behind it. */
val LocalToaster = staticCompositionLocalOf<@Composable () -> Unit> { {} }

/** Material's bottom sheet with a centered title. `content` gets a way to close it with the animation. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Sheet(title: String?, onClose: () -> Unit, content: @Composable ColumnScope.(close: () -> Unit) -> Unit) {
    val palette = LocalPalette.current
    val state = rememberBottomSheetState(SheetValue.Hidden, setOf(SheetValue.Hidden, SheetValue.Expanded))
    val scope = rememberCoroutineScope()
    val close: () -> Unit = { scope.launch { state.hide() }.invokeOnCompletion { onClose() } }
    ModalBottomSheet(
        onClose,
        sheetState = state,
        containerColor = palette.page,
        contentColor = palette.ink,
        shape = RoundedCornerShape(topStart = RadiusTokens.card.dp, topEnd = RadiusTokens.card.dp),
    ) {
        Column(
            Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).imePadding().navigationBarsPadding()
                .padding(horizontal = SpaceTokens.lg.dp).padding(bottom = SpaceTokens.lg.dp),
            verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp),
        ) {
            title?.let { Text(it, Modifier.fillMaxWidth(), style = MaterialTheme.typography.titleMedium, textAlign = TextAlign.Center) }
            content(close)
        }
        LocalToaster.current()
    }
}

@Composable
fun ConfirmDialog(title: String, body: String, action: String, onConfirm: () -> Unit, onDismiss: () -> Unit) {
    val palette = LocalPalette.current
    AlertDialog(
        onDismiss,
        confirmButton = { TextButton(onConfirm, colors = ButtonDefaults.textButtonColors(contentColor = palette.danger)) { Text(action) } },
        dismissButton = { TextButton(onDismiss, colors = ButtonDefaults.textButtonColors(contentColor = palette.ink)) { Text("Cancel") } },
        title = { Text(title, style = MaterialTheme.typography.titleLarge) },
        text = { Text(body, color = palette.inkMuted) },
        shape = CardShape,
        containerColor = palette.raised,
    )
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
