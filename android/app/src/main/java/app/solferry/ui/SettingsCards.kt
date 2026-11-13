package app.solferry.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.AvailableUpdate
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.BlockedDevice
import uniffi.solferry_ffi.Relay
import uniffi.solferry_ffi.RelayStatus
import uniffi.solferry_ffi.relaySpeedLimits

private const val GUIDE = "https://github.com/berkinory/solferry/blob/main/docs/relay.md"
private val MIB = 1024uL * 1024uL

@Composable
private fun RelayRow(relay: Relay, engine: Engine) {
    val palette = LocalPalette.current
    var copied by remember(relay.link) { mutableStateOf(false) }
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp)) {
        Tile(R.drawable.ic_relay, palette.control, palette.inkMuted)
        Column(Modifier.weight(1f)) {
            Text(relay.shown, maxLines = 1, overflow = TextOverflow.Ellipsis)
            when (relay.status) {
                RelayStatus.CHECKING -> StatusLine(stringResource(R.string.relay_checking))
                RelayStatus.CONNECTED -> StatusLine(stringResource(R.string.relay_connected), palette.success)
                RelayStatus.UDP_BLOCKED -> StatusLine(stringResource(R.string.relay_udp_blocked), palette.warning)
                RelayStatus.REFUSED -> StatusLine(stringResource(R.string.relay_refused), palette.danger)
                RelayStatus.UNREACHABLE -> StatusLine(stringResource(R.string.relay_unreachable), palette.danger)
            }
        }
        Button(
            { engine.copy(relay.link); copied = true }, icon = if (copied) R.drawable.ic_check else R.drawable.ic_copy,
            kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = stringResource(if (copied) R.string.common_copied else R.string.link_copy),
        )
        Button({ engine.bridge.removeRelay(relay.link) }, icon = R.drawable.ic_delete, kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = stringResource(R.string.common_remove, relay.shown))
    }
}

@Composable
fun RelayCard(state: AppState, engine: Engine) {
    val palette = LocalPalette.current
    val links = LocalUriHandler.current
    var link by remember { mutableStateOf("") }
    LaunchedEffect(state.relays.size) { link = "" }
    val add = { if (link.isNotBlank()) engine.bridge.addRelay(link.trim()) }
    Card(
        R.drawable.ic_relay, stringResource(R.string.relay_title),
        aside = { Button({ links.openUri(GUIDE) }, label = stringResource(R.string.relay_guide), icon = R.drawable.ic_external, kind = ButtonKind.Ghost, size = ButtonSize.Sm) },
    ) {
        PaddedWell {
            Muted(stringResource(R.string.relay_body))
            state.relays.forEach { RelayRow(it, engine) }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Field(
                    link, { link = it }, stringResource(R.string.relay_placeholder), Modifier.weight(1f),
                    trailing = {
                        Button(
                            { engine.pasted()?.let { link = it.trim() } }, icon = R.drawable.ic_clipboard,
                            kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = stringResource(R.string.common_paste),
                        )
                    },
                )
                Button(add, label = stringResource(R.string.relay_add))
            }
            RelaySpeedLimit(state.me?.relaySpeedLimit, engine)
        }
    }
}

/** How fast files may go through a relay. Choosing restarts the node, which applies it. */
@Composable
private fun RelaySpeedLimit(current: ULong?, engine: Engine) {
    Column(Modifier.padding(top = SpaceTokens.sm.dp), verticalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp)) {
        Text(stringResource(R.string.relay_limit_title))
        Muted(stringResource(R.string.relay_limit_hint))
        FlowRow(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp), verticalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp)) {
            (listOf<ULong?>(null) + relaySpeedLimits()).forEach { limit ->
                Button(
                    { engine.bridge.setRelaySpeedLimit(limit) },
                    label = limit?.let { stringResource(R.string.relay_limit_speed, (it / MIB).toInt()) } ?: stringResource(R.string.relay_limit_none),
                    kind = if (current == limit) ButtonKind.Secondary else ButtonKind.Ghost,
                    size = ButtonSize.Sm,
                )
            }
        }
    }
}

@Composable
fun SaveFolderCard(state: AppState, engine: Engine) {
    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { tree -> tree?.let(engine::chooseFolder) }
    Card(R.drawable.ic_folder, stringResource(R.string.folder_title)) {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Muted(state.saveFolder?.name ?: stringResource(R.string.folder_downloads), Modifier.weight(1f))
                if (state.saveFolder != null) Button(engine::resetFolder, label = stringResource(R.string.folder_reset), kind = ButtonKind.Ghost, size = ButtonSize.Sm)
                Button({ picker.launch(state.saveFolder?.tree) }, label = stringResource(R.string.folder_change), icon = R.drawable.ic_folder, size = ButtonSize.Sm)
            }
        }
    }
}

@Composable
fun UpdatesCard(enabled: Boolean, update: AvailableUpdate?, engine: Engine) {
    val links = LocalUriHandler.current
    Card(R.drawable.ic_download, stringResource(R.string.updates_title)) {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Muted(stringResource(R.string.updates_auto), Modifier.weight(1f))
                Toggle(enabled, { engine.setCheckUpdates(it) }, stringResource(R.string.updates_auto))
            }
            if (update != null) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                    Muted(stringResource(R.string.updates_available, update.version), Modifier.weight(1f))
                    Button({ links.openUri(update.page) }, label = stringResource(R.string.updates_download), icon = R.drawable.ic_external, size = ButtonSize.Sm)
                }
            }
        }
    }
}

@Composable
fun BlockedCard(blocked: List<BlockedDevice>, engine: Engine) {
    val palette = LocalPalette.current
    Card(R.drawable.ic_blocked, stringResource(R.string.blocked_title), aside = { Count(blocked.size) }) {
        Well {
            blocked.forEach { device ->
                Row(Modifier.fillMaxWidth().padding(8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp)) {
                    Tile(R.drawable.ic_blocked, palette.control, palette.inkMuted)
                    Column(Modifier.weight(1f)) {
                        Text(device.name ?: device.identity.fingerprint, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        if (device.name != null) Caption(device.identity.fingerprint)
                    }
                    Button({ engine.bridge.unblock(device.identity.id) }, label = stringResource(R.string.blocked_unblock), size = ButtonSize.Sm)
                }
            }
        }
    }
}
