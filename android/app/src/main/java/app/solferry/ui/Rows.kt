package app.solferry.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.Phase
import app.solferry.R
import app.solferry.Transfer
import app.solferry.design.MotionTokens
import uniffi.solferry_ffi.HistoryEntry
import uniffi.solferry_ffi.Nearby
import uniffi.solferry_ffi.PairedDevice

@Composable
private fun RowBody(onClick: (() -> Unit)?, highlighted: Boolean = false, content: @Composable RowScope.() -> Unit) {
    val palette = LocalPalette.current
    val ground = if (highlighted) palette.raised else Color.Transparent
    Row(
        Modifier.fillMaxWidth().clip(ControlShape).background(ground)
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        content = content,
    )
}

@Composable
private fun Detail(label: String, moment: String? = null, trailing: @Composable () -> Unit) = Row(
    Modifier.fillMaxWidth().height(46.dp).padding(start = 58.dp, end = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(4.dp),
) {
    Muted(label)
    moment?.let { Muted(it, color = LocalPalette.current.inkMuted.copy(alpha = 0.6f)) }
    Spacer(Modifier.weight(1f))
    trailing()
}

@Composable
private fun DeviceRow(
    device: PairedDevice,
    near: Nearby?,
    online: Boolean?,
    lastTransfer: Long?,
    twin: Boolean,
    engine: Engine,
    onText: (PairedDevice) -> Unit,
) {
    val palette = LocalPalette.current
    var open by remember { mutableStateOf(false) }
    var forgetting by remember { mutableStateOf(false) }
    Column {
        RowBody({ open = !open }, highlighted = open) {
            DeviceTile(device.identity.copy(platform = device.identity.platform ?: near?.identity?.platform), (online ?: (near != null)))
            Column(Modifier.weight(1f)) {
                Text(device.name, maxLines = 1, overflow = TextOverflow.Ellipsis)
                // Two paired devices under one name, usually one reinstalled, tell apart by key.
                val key = if (twin) " · ${device.identity.fingerprint}" else ""
                when {
                    (online ?: (near != null)) -> StatusLine(stringResource(R.string.device_online) + key, palette.successFill)
                    online == null -> StatusLine(stringResource(R.string.device_paired) + key)
                    else -> StatusLine(stringResource(R.string.device_offline) + key, palette.inkMuted.copy(alpha = 0.5f))
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                Button(
                    { onText(device) }, icon = R.drawable.ic_type_text,
                    kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = stringResource(R.string.devices_send_text, device.name),
                )
                Button(
                    { open = !open }, icon = R.drawable.ic_more,
                    kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = stringResource(R.string.devices_options, device.name),
                )
            }
        }
        AnimatedVisibility(
            open,
            enter = expandVertically(tween(MotionTokens.standard)) + fadeIn(tween(MotionTokens.standard)),
            exit = shrinkVertically(tween(MotionTokens.quick)) + fadeOut(tween(MotionTokens.quick)),
        ) {
            Column {
                Detail(stringResource(R.string.devices_auto_accept)) {
                    Toggle(device.autoAccept, { engine.bridge.setAutoAccept(device.identity.id, it) }, stringResource(R.string.devices_auto_accept))
                }
                Detail(stringResource(if (lastTransfer != null) R.string.devices_last_transfer else R.string.device_paired), ago(lastTransfer ?: device.pairedAtMs.toLong())) {
                    Button({ forgetting = true }, label = stringResource(R.string.devices_forget), icon = R.drawable.ic_delete, kind = ButtonKind.Danger, size = ButtonSize.Sm)
                }
            }
        }
    }
    if (forgetting) {
        ConfirmDialog(
            stringResource(R.string.devices_forget_title, device.name), stringResource(R.string.devices_forget_body), stringResource(R.string.devices_forget),
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
    history: List<HistoryEntry>,
    engine: Engine,
    onText: (PairedDevice) -> Unit,
) =
    Card(R.drawable.ic_devices, stringResource(R.string.devices_title), aside = { Count(devices.size) }) {
        Well {
            devices.forEach { device ->
                key(device.identity.id) {
                    val online = reachable?.contains(device.identity.id)
                    val last = history.firstOrNull { it.peer == device.identity.id && it.completed }?.finishedMs?.toLong()
                    val twin = devices.count { it.name == device.name } > 1
                    DeviceRow(device, nearby.find { it.identity.id == device.identity.id }, online, last, twin, engine, onText)
                }
            }
        }
    }

@Composable
fun NearbyCard(nearby: List<Nearby>, alone: Boolean, engine: Engine, onUseCode: () -> Unit) =
    Card(
        R.drawable.ic_nearby, stringResource(R.string.nearby_title),
        aside = { Button(onUseCode, label = stringResource(R.string.nearby_use_code), icon = R.drawable.ic_qr, kind = ButtonKind.Ghost, size = ButtonSize.Sm) },
    ) {
        Well {
            if (nearby.isEmpty()) { if (alone) Searching() else Scanning() }
            nearby.forEach { device ->
                RowBody(null) {
                    DeviceTile(device.identity)
                    Column(Modifier.weight(1f)) {
                        Text(device.name ?: stringResource(R.string.device_unnamed), maxLines = 1, overflow = TextOverflow.Ellipsis)
                        StatusLine(stringResource(R.string.device_not_paired))
                    }
                    Button({ engine.bridge.pairNearby(device.identity.id) }, label = stringResource(R.string.common_pair), icon = R.drawable.ic_link, size = ButtonSize.Sm)
                }
            }
        }
    }
