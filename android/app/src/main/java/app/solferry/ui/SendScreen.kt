package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.FastOutLinearInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.PickedFile
import app.solferry.R
import app.solferry.design.MotionTokens
import app.solferry.design.SpaceTokens
import kotlinx.coroutines.delay
import uniffi.solferry_ffi.Identity

private data class Target(val identity: Identity, val name: String, val detail: String)

private const val FLY_MS = 380L

@Composable
fun SendSheet(
    state: AppState,
    files: List<PickedFile>,
    onRemove: (PickedFile) -> Unit,
    onSend: (Set<String>) -> Unit,
    onShareLink: () -> Unit,
    onClose: () -> Unit,
) = Sheet("Send", onClose) { close ->
    var chosen by remember { mutableStateOf(emptySet<String>()) }
    var appeared by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { appeared = true }
    val mine = state.devices.map { device ->
        val near = state.nearby.find { it.identity.id == device.identity.id }
        val detail = when {
            near != null || state.reachable?.contains(device.identity.id) == true -> "Online"
            state.reachable != null -> "Offline"
            else -> "Paired"
        }
        Target(device.identity.copy(platform = device.identity.platform ?: near?.identity?.platform), device.name, detail)
    }
    val others = state.strangers.map { Target(it.identity, it.name ?: "Unnamed device", "Not paired") }
    Column {
        files.forEachIndexed { index, file -> FileRow(file, index, appeared) { onRemove(file) } }
    }
    Frame {
        Well {
            if (mine.isEmpty() && others.isEmpty()) Scanning()
            (mine + others).forEach { target ->
                TargetRow(target, target.identity.id in chosen) {
                    chosen = if (target.identity.id in chosen) chosen - target.identity.id else chosen + target.identity.id
                }
            }
        }
    }
    Box(Modifier.fillMaxWidth().padding(top = SpaceTokens.sm.dp), contentAlignment = Alignment.Center) {
        LaunchButton(chosen.size, { onSend(chosen) }, close)
    }
    Button(onShareLink, Modifier.fillMaxWidth(), label = "Share via link", icon = R.drawable.ic_link, kind = ButtonKind.Ghost)
}

/** Where the files come from, as on iOS: the photo picker or the file browser. */
@Composable
fun SourceSheet(onMedia: () -> Unit, onFiles: () -> Unit, onClose: () -> Unit) = Sheet(null, onClose) { close ->
    Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
        SourceTile("Photos and videos", R.drawable.ic_type_image, Modifier.weight(1f)) { close(); onMedia() }
        SourceTile("Files", R.drawable.ic_folder, Modifier.weight(1f)) { close(); onFiles() }
    }
}

@Composable
private fun SourceTile(label: String, @DrawableRes icon: Int, modifier: Modifier, onClick: () -> Unit) = Frame(modifier) {
    Well(Modifier.aspectRatio(1f), onClick = onClick) {
        Column(
            Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp, Alignment.CenterVertically),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Glyph(icon, LocalPalette.current.ink, 32)
            Text(label, style = MaterialTheme.typography.titleSmall, textAlign = TextAlign.Center)
        }
    }
}

@Composable
private fun FileRow(file: PickedFile, index: Int, appeared: Boolean, onRemove: () -> Unit) {
    val palette = LocalPalette.current
    val shown by animateFloatAsState(if (appeared) 1f else 0f, tween(MotionTokens.slow, delayMillis = 45 * index))
    val density = LocalDensity.current
    Row(
        Modifier.fillMaxWidth().height(48.dp).padding(start = SpaceTokens.xs.dp).graphicsLayer {
            alpha = shown
            translationX = with(density) { (1 - shown) * -14.dp.toPx() }
        },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp),
    ) {
        Glyph(fileIcon(file.name), palette.inkMuted, 22)
        Text(file.name, Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
        file.size?.let { Muted(bytes(it)) }
        Box(Modifier.size(36.dp).clip(ControlShape).clickable(onClick = onRemove), contentAlignment = Alignment.Center) {
            Glyph(R.drawable.ic_delete, palette.danger, 18, "Remove ${file.name}")
        }
    }
}

/** Lets the cursor fly, then closes the sheet. The transfer pill takes over from there. */
@Composable
private fun LaunchButton(chosen: Int, onLaunch: () -> Unit, onDone: () -> Unit) {
    val palette = LocalPalette.current
    var flying by remember { mutableStateOf(false) }
    val haptics = LocalHapticFeedback.current
    LaunchedEffect(flying) { if (flying) { delay(FLY_MS); onDone() } }
    val fly by animateFloatAsState(if (flying) 1f else 0f, tween(380, easing = FastOutLinearInEasing))
    val label = if (chosen > 1) "Send to $chosen devices" else "Send"
    val density = LocalDensity.current
    Box(
        Modifier.width(210.dp).height(56.dp).graphicsLayer { alpha = if (chosen > 0) 1f else 0.5f }.clip(FieldShape)
            .background(palette.primaryFill).clickable(enabled = chosen > 0 && !flying) {
                haptics.performHapticFeedback(HapticFeedbackType.Confirm)
                onLaunch()
                flying = true
            },
        contentAlignment = Alignment.Center,
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
            Box(
                Modifier.graphicsLayer {
                    rotationZ = 90f
                    translationX = with(density) { fly * 36.dp.toPx() }
                    translationY = with(density) { fly * -36.dp.toPx() }
                    alpha = 1 - fly
                },
            ) { Glyph(R.drawable.ic_cursor, palette.onPrimary, 22) }
            AnimatedContent(label, transitionSpec = { fadeIn() togetherWith fadeOut() }) {
                Text(it, style = MaterialTheme.typography.titleMedium, color = palette.onPrimary)
            }
        }
    }
}

@Composable
private fun TargetRow(target: Target, chosen: Boolean, onToggle: () -> Unit) {
    val palette = LocalPalette.current
    val haptics = LocalHapticFeedback.current
    Row(
        Modifier.fillMaxWidth().clip(ControlShape).background(if (chosen) palette.raised else Color.Transparent).clickable {
            haptics.performHapticFeedback(if (chosen) HapticFeedbackType.ToggleOff else HapticFeedbackType.ToggleOn)
            onToggle()
        }.padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        if (chosen) Tile(R.drawable.ic_check, palette.primaryFill, palette.onPrimary) else DeviceTile(target.identity, target.detail != "Offline" && target.detail != "Paired")
        Column(Modifier.weight(1f)) {
            Text(target.name, maxLines = 1, overflow = TextOverflow.Ellipsis)
            StatusLine(target.detail, if (target.detail == "Online") palette.successFill else null)
        }
    }
}
