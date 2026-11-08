package app.solferry.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.PickedFile
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.Identity

private data class Target(val identity: Identity, val name: String, val detail: String)

@Composable
fun SendScreen(
    state: AppState,
    files: List<PickedFile>,
    onRemove: (PickedFile) -> Unit,
    onAddMore: () -> Unit,
    onSend: (Set<String>) -> Unit,
    onBack: () -> Unit,
) = Page("Send", onBack) {
    var chosen by remember { mutableStateOf(emptySet<String>()) }
    val mine = state.devices.map { device ->
        val near = state.nearby.find { it.identity.id == device.identity.id }
        val detail = when {
            near?.overlay == true -> "On your overlay network"
            near != null -> "On this network"
            state.reachable?.contains(device.identity.id) == true -> "Online"
            state.reachable != null -> "Offline"
            else -> "Paired"
        }
        Target(device.identity.copy(platform = device.identity.platform ?: near?.identity?.platform), device.name, detail)
    }
    val others = state.strangers.map { Target(it.identity, it.name ?: "Unnamed device", if (it.overlay) "Overlay, not paired" else "Not paired") }
    Card(R.drawable.ic_file, "Files", aside = { Button(onAddMore, label = "Add", icon = R.drawable.ic_add, kind = ButtonKind.Ghost, size = ButtonSize.Sm) }) {
        Well(spacing = 0.dp) { files.forEach { FileRow(it) { onRemove(it) } } }
    }
    Card(R.drawable.ic_devices, "Send to") {
        Well {
            if (mine.isEmpty() && others.isEmpty()) Scanning()
            (mine + others).forEach { target ->
                TargetRow(target, target.identity.id in chosen) {
                    chosen = if (target.identity.id in chosen) chosen - target.identity.id else chosen + target.identity.id
                }
            }
        }
    }
    val label = if (chosen.size > 1) "Send to ${chosen.size} devices" else "Send"
    Button({ onSend(chosen) }, Modifier.fillMaxWidth(), label = label, icon = R.drawable.ic_send, kind = ButtonKind.Primary, enabled = chosen.isNotEmpty())
}

@Composable
private fun FileRow(file: PickedFile, onRemove: () -> Unit) {
    val palette = LocalPalette.current
    Row(Modifier.fillMaxWidth().height(44.dp).padding(start = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
        Glyph(R.drawable.ic_file, palette.inkMuted, 18)
        Text(file.name, Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
        file.size?.let { Mono(bytes(it)) }
        Button(onRemove, icon = R.drawable.ic_cancel, kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Remove ${file.name}")
    }
}

@Composable
private fun TargetRow(target: Target, chosen: Boolean, onToggle: () -> Unit) {
    val palette = LocalPalette.current
    Row(
        Modifier.fillMaxWidth().background(if (chosen) palette.raised else androidx.compose.ui.graphics.Color.Transparent, ControlShape).clickable(onClick = onToggle).padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        if (chosen) Tile(R.drawable.ic_check, palette.primaryFill, palette.onPrimary) else DeviceTile(target.identity)
        Column(Modifier.weight(1f)) {
            Text(target.name, maxLines = 1, overflow = TextOverflow.Ellipsis)
            StatusLine(target.detail)
        }
    }
}
