package app.solferry.ui

import android.net.Uri
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.IconButton
import androidx.compose.material3.LargeTopAppBar
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import uniffi.solferry_ffi.PairedDevice

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    state: AppState,
    engine: Engine,
    staged: List<Uri>,
    autoAccept: Boolean,
    onPickFiles: (PairedDevice?) -> Unit,
    onSendStaged: (PairedDevice) -> Unit,
    onClearStaged: () -> Unit,
    onAddDevice: () -> Unit,
    onSettings: () -> Unit,
) {
    val scroll = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()
    val palette = LocalPalette.current
    Scaffold(
        modifier = Modifier.nestedScroll(scroll.nestedScrollConnection),
        topBar = {
            LargeTopAppBar(
                title = { Text("Solferry") },
                actions = {
                    IconButton(onClick = onAddDevice) { Glyph(R.drawable.ic_add, MaterialTheme.colorScheme.onSurface) }
                    IconButton(onClick = onSettings) { Glyph(R.drawable.ic_settings, MaterialTheme.colorScheme.onSurface) }
                },
                scrollBehavior = scroll,
            )
        },
        floatingActionButton = {
            if (staged.isEmpty() && state.devices.isNotEmpty()) {
                ExtendedFloatingActionButton(
                    onClick = { onPickFiles(null) },
                    icon = { Glyph(R.drawable.ic_send, palette.onPrimary) },
                    text = { Text("Send") },
                    containerColor = palette.primaryFill,
                    contentColor = palette.onPrimary,
                )
            }
        },
    ) { padding ->
        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(top = padding.calculateTopPadding(), bottom = 112.dp)) {
            state.startProblem?.let { item { ProblemCard(it, null, null) } }
            items(state.problems.size) { index ->
                val (name, problem) = state.problems[index]
                ProblemCard(problem, name, onDismiss = { engine.dismissProblem(index) })
            }
            items(state.pairings, key = { "pair-${it.key}" }) { PairingCard(it, engine, autoAccept) }
            state.pairingCode?.let { item { CompareCard(it) } }
            items(state.offers, key = { "offer-${it.key}" }) { OfferCard(it, engine) }
            items(state.texts, key = { "text-${it.key}" }) { TextCard(it, engine) }
            if (staged.isNotEmpty()) item { Staged(staged.size, onClearStaged) }
            if (state.devices.isEmpty() && state.strangers.isEmpty()) item { Empty(onAddDevice) }
            if (state.devices.isNotEmpty()) {
                item { SectionLabel("Devices") }
                item {
                    DeviceRows(state.devices, state.nearby, staged.isNotEmpty(), engine) { device ->
                        if (staged.isNotEmpty()) onSendStaged(device) else onPickFiles(device)
                    }
                }
            }
            if (state.strangers.isNotEmpty()) {
                item { SectionLabel("Nearby") }
                item { NearbyRows(state.strangers, autoAccept, engine) }
            }
            if (state.transfers.isNotEmpty()) {
                item { SectionLabel("Transfers") }
                item { TransferRows(state.transfers, engine) }
            }
        }
    }
}

@Composable
private fun Staged(count: Int, onClear: () -> Unit) {
    val palette = LocalPalette.current
    Notice(R.drawable.ic_upload, if (count == 1) "1 file ready" else "$count files ready", palette.primary, palette.well) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Tap a device to send.", Modifier.weight(1f), color = palette.inkMuted)
            TextButton(onClick = onClear) { Text("Cancel") }
        }
    }
}

@Composable
private fun Empty(onAddDevice: () -> Unit) {
    val palette = LocalPalette.current
    Column(
        Modifier.fillMaxWidth().padding(horizontal = 32.dp, vertical = 48.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Glyph(R.drawable.ic_devices, palette.inkMuted, 40)
        Spacer(Modifier.height(4.dp))
        Text("No devices yet", style = MaterialTheme.typography.titleMedium)
        Text(
            "Open Solferry on another device on this network, or pair with a code.",
            textAlign = TextAlign.Center, color = palette.inkMuted,
        )
        Spacer(Modifier.height(8.dp))
        FilledTonalButton(onClick = onAddDevice) {
            Glyph(R.drawable.ic_qr, size = 18)
            Spacer(Modifier.width(8.dp))
            Text("Add with a code")
        }
    }
}
