package app.solferry.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
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

private const val GUIDE = "https://github.com/berkinory/solferry/blob/main/docs/relay.md"

@Composable
private fun RelayRow(relay: Relay, engine: Engine) {
    val palette = LocalPalette.current
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp)) {
        Tile(R.drawable.ic_relay, palette.control, palette.inkMuted)
        Column(Modifier.weight(1f)) {
            Text(relay.url, maxLines = 1, overflow = TextOverflow.Ellipsis)
            when (relay.status) {
                RelayStatus.CHECKING -> StatusLine("Checking")
                RelayStatus.CONNECTED -> StatusLine("Connected", palette.success)
                RelayStatus.UDP_BLOCKED -> StatusLine("Connected, UDP 7842 is blocked", palette.warning)
                RelayStatus.REFUSED -> StatusLine("Refused, check the link", palette.danger)
                RelayStatus.UNREACHABLE -> StatusLine("Can't reach the server", palette.danger)
            }
        }
        Button({ engine.bridge.removeRelay(relay.link) }, icon = R.drawable.ic_delete, kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Remove ${relay.url}")
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
        R.drawable.ic_relay, "Relay",
        aside = { Button({ links.openUri(GUIDE) }, label = "Setup Guide", icon = R.drawable.ic_external, kind = ButtonKind.Ghost, size = ButtonSize.Sm) },
    ) {
        PaddedWell {
            Muted("Reach devices on other networks through a relay you run.")
            state.relays.forEach { RelayRow(it, engine) }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Field(
                    link, { link = it }, "https://relay.example.com#token", Modifier.weight(1f),
                    trailing = {
                        Button(
                            { engine.pasted()?.let { link = it.trim() } }, icon = R.drawable.ic_clipboard,
                            kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Paste",
                        )
                    },
                )
                Button(add, label = "Add")
            }
        }
    }
}

@Composable
fun UpdatesCard(enabled: Boolean, update: AvailableUpdate?, engine: Engine) {
    val links = LocalUriHandler.current
    Card(R.drawable.ic_download, "Updates") {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Muted("Check for updates automatically", Modifier.weight(1f))
                Toggle(enabled, { engine.setCheckUpdates(it) }, "Check for updates automatically")
            }
            if (update != null) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                    Muted("Solferry ${update.version} is available", Modifier.weight(1f))
                    Button({ links.openUri(update.page) }, label = "Download", icon = R.drawable.ic_external, size = ButtonSize.Sm)
                }
            }
        }
    }
}

@Composable
fun BlockedCard(blocked: List<BlockedDevice>, engine: Engine) {
    val palette = LocalPalette.current
    Card(R.drawable.ic_blocked, "Blocked", aside = { Count(blocked.size) }) {
        Well {
            blocked.forEach { device ->
                Row(Modifier.fillMaxWidth().padding(8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp)) {
                    Tile(R.drawable.ic_blocked, palette.control, palette.inkMuted)
                    Column(Modifier.weight(1f)) {
                        Text(device.name ?: device.identity.fingerprint, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        if (device.name != null) Caption(device.identity.fingerprint)
                    }
                    Button({ engine.bridge.unblock(device.identity.id) }, label = "Unblock", size = ButtonSize.Sm)
                }
            }
        }
    }
}
