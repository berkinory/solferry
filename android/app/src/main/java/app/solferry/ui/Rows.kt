package app.solferry.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.Phase
import app.solferry.R
import app.solferry.Transfer
import app.solferry.design.MotionTokens
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
private fun RowBody(onClick: (() -> Unit)?, highlighted: Boolean = false, content: @Composable RowScope.() -> Unit) {
    val palette = LocalPalette.current
    val press = remember { MutableInteractionSource() }
    val pressed by press.collectIsPressedAsState()
    val ground = if (pressed || highlighted) palette.raised else Color.Transparent
    Row(
        Modifier.fillMaxWidth().background(ground, ControlShape)
            .then(if (onClick != null) Modifier.clickable(press, null, onClick = onClick) else Modifier)
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        content = content,
    )
}

@Composable
private fun Detail(label: String, trailing: @Composable () -> Unit) = Row(
    Modifier.fillMaxWidth().height(46.dp).padding(start = 58.dp, end = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
) {
    Muted(label, Modifier.weight(1f))
    trailing()
}

@Composable
private fun DeviceRow(
    device: PairedDevice,
    near: Nearby?,
    online: Boolean?,
    picking: Boolean,
    engine: Engine,
    onPick: (PairedDevice) -> Unit,
) {
    val palette = LocalPalette.current
    var open by remember { mutableStateOf(false) }
    var forgetting by remember { mutableStateOf(false) }
    Column {
        RowBody({ onPick(device) }, highlighted = open) {
            DeviceTile(device.identity.copy(platform = device.identity.platform ?: near?.identity?.platform))
            Column(Modifier.weight(1f)) {
                Text(device.name, maxLines = 1, overflow = TextOverflow.Ellipsis)
                when {
                    near == null && online == null -> StatusLine("Paired")
                    near == null && online == true -> StatusLine("Online", palette.success)
                    near == null -> StatusLine("Offline", palette.inkMuted.copy(alpha = 0.5f))
                    near.overlay -> StatusLine("On your overlay network", palette.routes[TransferRoute.OVERLAY])
                    else -> StatusLine("On this network", palette.routes[TransferRoute.LOCAL])
                }
            }
            if (picking) {
                Glyph(R.drawable.ic_send, palette.primary, 18)
            } else {
                Row(horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                    Button(
                        { engine.shareClipboard(device.identity.id, device.name) }, icon = R.drawable.ic_clipboard,
                        kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Send clipboard to ${device.name}",
                    )
                    Button(
                        { open = !open }, icon = R.drawable.ic_more,
                        kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Options for ${device.name}",
                    )
                }
            }
        }
        AnimatedVisibility(
            open && !picking,
            enter = expandVertically(tween(MotionTokens.standard)) + fadeIn(tween(MotionTokens.standard)),
            exit = shrinkVertically(tween(MotionTokens.quick)) + fadeOut(tween(MotionTokens.quick)),
        ) {
            Column {
                Detail("Key") { Mono(device.identity.fingerprint) }
                Detail("Receive without asking") {
                    Toggle(device.autoAccept, { engine.bridge.setAutoAccept(device.identity.id, it) }, "Receive without asking")
                }
                Detail("Remove this device") {
                    Button({ forgetting = true }, label = "Forget", icon = R.drawable.ic_delete, size = ButtonSize.Sm)
                }
            }
        }
    }
    if (forgetting) {
        ConfirmDialog(
            "Forget ${device.name}?", "It goes back to being an unpaired device until you pair again.", "Forget",
            onConfirm = { forgetting = false; engine.bridge.forget(device.identity.id) },
            onDismiss = { forgetting = false },
        )
    }
}

@Composable
fun DeviceCard(
    devices: List<PairedDevice>,
    nearby: List<Nearby>,
    reachable: Set<String>?,
    picking: Boolean,
    engine: Engine,
    onPick: (PairedDevice) -> Unit,
) =
    Card(R.drawable.ic_devices, "Devices", aside = { Count(devices.size) }) {
        Well {
            devices.forEach { device ->
                key(device.identity.id) {
                    val online = reachable?.contains(device.identity.id)
                    DeviceRow(device, nearby.find { it.identity.id == device.identity.id }, online, picking, engine, onPick)
                }
            }
        }
    }

@Composable
fun NearbyCard(nearby: List<Nearby>, engine: Engine, onUseCode: () -> Unit) =
    Card(
        R.drawable.ic_nearby, "Nearby",
        aside = { Button(onUseCode, label = "Use a code", icon = R.drawable.ic_qr, kind = ButtonKind.Ghost, size = ButtonSize.Sm) },
    ) {
        Well {
            if (nearby.isEmpty()) Scanning()
            nearby.forEach { device ->
                RowBody(null) {
                    DeviceTile(device.identity)
                    Column(Modifier.weight(1f)) {
                        Text(device.name ?: "Unnamed device", maxLines = 1, overflow = TextOverflow.Ellipsis)
                        StatusLine(if (device.overlay) "Overlay, not paired" else "Not paired")
                    }
                    Button({ engine.bridge.pairNearby(device.identity.id) }, label = "Pair", size = ButtonSize.Sm)
                }
            }
        }
    }

@Composable
private fun TransferRow(transfer: Transfer, engine: Engine) {
    val palette = LocalPalette.current
    RowBody(null) {
        val tone = if (transfer.running) palette.routes[transfer.route] ?: palette.inkMuted else palette.inkMuted
        Tile(if (transfer.sending) R.drawable.ic_upload else R.drawable.ic_download, palette.control, tone)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(transfer.problem?.title ?: transfer.label, maxLines = 1, overflow = TextOverflow.Ellipsis)
            val verb = if (transfer.sending) "To" else "From"
            val where = if (transfer.running) ", ${routeName(transfer.route)}" else ""
            StatusLine("$verb ${transfer.peerName}$where")
            when (transfer.phase) {
                Phase.Active, Phase.Reconnecting -> {
                    val reconnecting = transfer.phase == Phase.Reconnecting
                    Spacer(Modifier.height(4.dp))
                    ProgressBar(transfer.fraction, if (reconnecting) palette.warning else palette.primary)
                    Spacer(Modifier.height(2.dp))
                    Row {
                        Mono("${bytes(transfer.transferred)} / ${bytes(transfer.total)}", Modifier.weight(1f))
                        Mono(if (reconnecting) "reconnecting" else "${bytes(transfer.bytesPerSecond.toLong())}/s")
                    }
                }
                Phase.Done -> Mono(bytes(transfer.total), color = palette.success)
                Phase.Failed -> Muted(transfer.problem?.next.orEmpty(), color = palette.danger)
            }
        }
        Button(
            { if (transfer.running) engine.bridge.cancel(transfer.id) else engine.clear(transfer.id) },
            icon = R.drawable.ic_cancel, kind = ButtonKind.Ghost, size = ButtonSize.Sm,
            description = if (transfer.running) "Cancel" else "Clear",
        )
    }
}

@Composable
fun TransferCard(transfers: List<Transfer>, engine: Engine) =
    Card(R.drawable.ic_transfers, "Transfers", aside = { Count(transfers.size) }) {
        Well { transfers.forEach { TransferRow(it, engine) } }
    }
