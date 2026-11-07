package app.solferry.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.Phase
import app.solferry.R
import app.solferry.Transfer
import uniffi.solferry_ffi.Nearby
import uniffi.solferry_ffi.PairedDevice
import uniffi.solferry_ffi.TransferRoute

private fun routeName(route: TransferRoute) = when (route) {
    TransferRoute.LOCAL -> "local network"
    TransferRoute.OVERLAY -> "overlay network"
    TransferRoute.DIRECT -> "direct"
    TransferRoute.RELAY -> "relay"
}

@Composable
fun DeviceRows(
    devices: List<PairedDevice>,
    nearby: List<Nearby>,
    picking: Boolean,
    engine: Engine,
    onPick: (PairedDevice) -> Unit,
) {
    val palette = LocalPalette.current
    Segmented(devices.size) { index, shape ->
        val device = devices[index]
        val near = nearby.find { it.identity.id == device.identity.id }
        var menu by remember { mutableStateOf(false) }
        var forgetting by remember { mutableStateOf(false) }
        SegmentRow(shape, onClick = { onPick(device) }) {
            DeviceTile(device.identity.copy(platform = device.identity.platform ?: near?.identity?.platform))
            Column(Modifier.weight(1f)) {
                Text(device.name, style = MaterialTheme.typography.bodyLarge, maxLines = 1, overflow = TextOverflow.Ellipsis)
                when {
                    near == null -> StatusLine("Paired", null)
                    near.overlay -> StatusLine("On your overlay network", palette.routes[TransferRoute.OVERLAY])
                    else -> StatusLine("On this network", palette.routes[TransferRoute.LOCAL])
                }
            }
            if (!picking) {
                IconButton(onClick = { engine.shareClipboard(device.identity.id, device.name) }) { Glyph(R.drawable.ic_clipboard) }
                IconButton(onClick = { menu = true }) {
                    Glyph(R.drawable.ic_more)
                    DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                        DropdownMenuItem(
                            text = { Text("Receive without asking") },
                            onClick = { engine.bridge.setAutoAccept(device.identity.id, !device.autoAccept) },
                            trailingIcon = {
                                Switch(device.autoAccept, { engine.bridge.setAutoAccept(device.identity.id, it) })
                            },
                        )
                        DropdownMenuItem(
                            text = { Text("Key  ${device.identity.fingerprint}") },
                            onClick = {},
                            enabled = false,
                        )
                        DropdownMenuItem(
                            text = { Text("Forget", color = palette.danger) },
                            leadingIcon = { Glyph(R.drawable.ic_delete, palette.danger) },
                            onClick = { menu = false; forgetting = true },
                        )
                    }
                }
            }
        }
        if (forgetting) {
            AlertDialog(
                onDismissRequest = { forgetting = false },
                title = { Text("Forget ${device.name}?") },
                text = { Text("It won't send or receive until you pair again.") },
                confirmButton = {
                    TextButton(onClick = { forgetting = false; engine.bridge.forget(device.identity.id) }) {
                        Text("Forget", color = palette.danger)
                    }
                },
                dismissButton = { TextButton(onClick = { forgetting = false }) { Text("Cancel") } },
            )
        }
    }
}

@Composable
fun NearbyRows(nearby: List<Nearby>, autoAccept: Boolean, engine: Engine) {
    Segmented(nearby.size) { index, shape ->
        val device = nearby[index]
        SegmentRow(shape) {
            DeviceTile(device.identity)
            Column(Modifier.weight(1f)) {
                Text(device.name ?: "Unnamed device", style = MaterialTheme.typography.bodyLarge, maxLines = 1)
                StatusLine(if (device.overlay) "Overlay, not paired" else "Not paired", null)
            }
            FilledTonalButton(onClick = { engine.bridge.pairNearby(device.identity.id, autoAccept) }) { Text("Pair") }
        }
    }
}

@Composable
fun TransferRows(transfers: List<Transfer>, engine: Engine) {
    val palette = LocalPalette.current
    Segmented(transfers.size) { index, shape ->
        val transfer = transfers[index]
        SegmentRow(shape) {
            val tone = if (transfer.running) palette.routes[transfer.route] ?: palette.inkMuted else palette.inkMuted
            Glyph(if (transfer.sending) R.drawable.ic_upload else R.drawable.ic_download, tone, 22)
            Column(Modifier.weight(1f)) {
                Text(
                    transfer.problem?.title ?: transfer.label,
                    style = MaterialTheme.typography.bodyLarge, maxLines = 1, overflow = TextOverflow.Ellipsis,
                )
                val verb = if (transfer.sending) "To" else "From"
                val where = if (transfer.running) ", ${routeName(transfer.route)}" else ""
                StatusLine("$verb ${transfer.peerName}$where", null)
                when (transfer.phase) {
                    Phase.Active, Phase.Reconnecting -> {
                        LinearProgressIndicator(
                            progress = { transfer.fraction },
                            modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
                            color = if (transfer.phase == Phase.Reconnecting) palette.warning else palette.primary,
                        )
                        Row {
                            Mono("${bytes(transfer.transferred)} / ${bytes(transfer.total)}", )
                            Text("", Modifier.weight(1f))
                            Mono(if (transfer.phase == Phase.Reconnecting) "reconnecting" else "${bytes(transfer.bytesPerSecond.toLong())}/s")
                        }
                    }
                    Phase.Done -> Mono(bytes(transfer.total), palette.success)
                    Phase.Failed -> Text(transfer.problem?.next.orEmpty(), color = palette.danger, style = MaterialTheme.typography.bodyMedium)
                }
            }
            IconButton(onClick = { if (transfer.running) engine.bridge.cancel(transfer.id) else engine.clear(transfer.id) }) {
                Glyph(R.drawable.ic_cancel)
            }
        }
    }
}
