package app.solferry.ui

import android.net.Uri
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.PairedDevice

@Composable
fun HomeScreen(
    state: AppState,
    engine: Engine,
    staged: List<Uri>,
    onPickFiles: (PairedDevice?) -> Unit,
    onSendStaged: (PairedDevice) -> Unit,
    onClearStaged: () -> Unit,
    onUseCode: () -> Unit,
    onSettings: () -> Unit,
) = Column(Modifier.fillMaxSize()) {
    Toolbar {
        Text("Solferry", Modifier.padding(start = SpaceTokens.sm.dp), style = MaterialTheme.typography.titleMedium)
        Spacer(Modifier.width(6.dp))
        Muted(state.me?.name.orEmpty(), Modifier.weight(1f))
        Button(onSettings, icon = R.drawable.ic_settings, kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Settings")
    }
    val bottom = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
    LazyColumn(
        Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = SpaceTokens.md.dp, end = SpaceTokens.md.dp, top = SpaceTokens.xs.dp, bottom = bottom + SpaceTokens.md.dp),
        verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp),
    ) {
        state.startProblem?.let { item { ProblemCard(it, null, null) } }
        items(state.problems.size) { index ->
            val (name, problem) = state.problems[index]
            ProblemCard(problem, name, onDismiss = { engine.dismissProblem(index) })
        }
        items(state.pairings, key = { "pair-${it.key}" }) { PairingCard(it, engine) }
        state.pairingCode?.let { item { CompareCard(it) } }
        items(state.offers, key = { "offer-${it.key}" }) { OfferCard(it, engine) }
        items(state.texts, key = { "text-${it.key}" }) { TextCard(it, engine) }
        if (staged.isNotEmpty()) {
            item { Staged(staged.size, onClearStaged) }
        } else if (state.devices.isNotEmpty()) {
            item { SendZone { onPickFiles(null) } }
        }
        if (state.devices.isEmpty() && state.strangers.isEmpty()) item { RadarHero(onUseCode) }
        if (state.devices.isNotEmpty()) {
            item {
                DeviceCard(state.devices, state.nearby, state.reachable, staged.isNotEmpty(), engine) { device ->
                    if (staged.isNotEmpty()) onSendStaged(device) else onPickFiles(device)
                }
            }
        }
        if (state.devices.isNotEmpty() || state.strangers.isNotEmpty()) item { NearbyCard(state.strangers, engine, onUseCode) }
        if (state.transfers.isNotEmpty()) item { TransferCard(state.transfers, engine) }
    }
}

@Composable
private fun SendZone(onClick: () -> Unit) {
    val palette = LocalPalette.current
    Frame {
        Well(padding = 14.dp, onClick = onClick) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp)) {
                Box(
                    Modifier.size(44.dp).background(palette.primaryFill, ControlShape).border(1.dp, palette.primaryBorder, ControlShape),
                    contentAlignment = Alignment.Center,
                ) { Glyph(R.drawable.ic_send, palette.onPrimary, 20) }
                Column {
                    Text("Send files")
                    Muted("Choose what to send, then tap a device")
                }
            }
        }
    }
}

@Composable
private fun Staged(count: Int, onClear: () -> Unit) = Card(R.drawable.ic_upload, "Ready to send", aside = { Count(count) }) {
    PaddedWell {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Muted(if (count == 1) "Tap a device to send this file." else "Tap a device to send these files.", Modifier.weight(1f))
            Button(onClear, label = "Cancel", kind = ButtonKind.Ghost, size = ButtonSize.Sm)
        }
    }
}
