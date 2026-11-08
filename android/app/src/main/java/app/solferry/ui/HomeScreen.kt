package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
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
    onPickFiles: (PairedDevice?) -> Unit,
    onUseCode: () -> Unit,
) = Column(Modifier.fillMaxSize()) {
    HomeBar()
    LazyColumn(
        Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = SpaceTokens.md.dp, end = SpaceTokens.md.dp, top = SpaceTokens.xs.dp, bottom = tabClearance()),
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
        item { SendZone { onPickFiles(null) } }
        if (state.devices.isEmpty() && state.strangers.isEmpty()) item { RadarHero(onUseCode) }
        if (state.devices.isNotEmpty()) {
            item { DeviceCard(state.devices, state.nearby, state.reachable, state.history, engine) { onPickFiles(it) } }
        }
        if (state.devices.isNotEmpty() || state.strangers.isNotEmpty()) item { NearbyCard(state.strangers, engine, onUseCode) }
        if (state.transfers.isNotEmpty()) item { TransferCard(state.transfers, engine) }
    }
}

@Composable
private fun HomeBar() {
    val palette = LocalPalette.current
    Row(
        Modifier.fillMaxWidth().statusBarsPadding().height(52.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally),
    ) {
        Glyph(R.drawable.ic_mark, palette.brand, 20)
        Text("Solferry", style = MaterialTheme.typography.titleMedium)
    }
}

@Composable
private fun SendZone(onClick: () -> Unit) {
    val palette = LocalPalette.current
    Frame {
        Well(Modifier.height(148.dp), padding = 14.dp, onClick = onClick) {
            Column(Modifier.fillMaxSize(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
                Box(Modifier.width(140.dp).height(64.dp), contentAlignment = Alignment.Center) {
                    FanTile(R.drawable.ic_type_image, (-46).dp)
                    FanTile(R.drawable.ic_type_pdf, 46.dp)
                    Box(
                        Modifier.size(56.dp).background(palette.primaryFill, FieldShape).border(1.dp, palette.primaryBorder, FieldShape),
                        contentAlignment = Alignment.Center,
                    ) { Glyph(R.drawable.ic_send, palette.onPrimary, 24) }
                }
                Spacer(Modifier.height(SpaceTokens.sm.dp))
                Text("Send files", style = MaterialTheme.typography.titleMedium)
                Muted("Tap to choose what to send")
            }
        }
    }
}

@Composable
private fun FanTile(@DrawableRes icon: Int, offset: Dp) {
    val palette = LocalPalette.current
    Box(
        Modifier.offset(x = offset, y = 6.dp).size(44.dp).background(palette.control, ControlShape).border(1.dp, palette.line, ControlShape),
        contentAlignment = Alignment.Center,
    ) { Glyph(icon, palette.inkMuted, 20) }
}
