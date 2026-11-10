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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import kotlinx.coroutines.delay
import uniffi.solferry_ffi.PairedDevice

private const val REFRESH_MS = 700L

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    state: AppState,
    engine: Engine,
    onPickFiles: (PairedDevice?) -> Unit,
    onText: (PairedDevice) -> Unit,
    onUseCode: () -> Unit,
) = Column(Modifier.fillMaxSize()) {
    HomeBar()
    var refreshing by remember { mutableStateOf(false) }
    LaunchedEffect(refreshing) {
        if (refreshing) {
            engine.bridge.checkPresence()
            delay(REFRESH_MS)
            refreshing = false
        }
    }
    PullToRefreshBox(refreshing, { refreshing = true }, Modifier.fillMaxSize()) { LazyColumn(
        Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = SpaceTokens.md.dp, end = SpaceTokens.md.dp, top = SpaceTokens.xs.dp, bottom = tabClearance()),
        verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp),
    ) {
        items(state.offers, key = { "offer-${it.key}" }) { OfferCard(it, engine) }
        state.text?.let { item(key = "text-${it.key}") { TextCard(it, engine) } }
        item { SendZone { onPickFiles(null) } }
        if (state.devices.isNotEmpty()) {
            item { DeviceCard(state.devices, state.nearby, state.reachable, state.history, engine, { onPickFiles(it) }, onText) }
        }
        item { NearbyCard(state.strangers, state.devices.isEmpty(), engine, onUseCode) }
    } }
}

@Composable
private fun HomeBar() = Toolbar {
    Row(Modifier.padding(start = SpaceTokens.sm.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        Glyph(R.drawable.ic_mark, LocalPalette.current.brand, 20)
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
                    FanTile(R.drawable.ic_type_image, (-38).dp, -11f)
                    FanTile(R.drawable.ic_type_pdf, 38.dp, 11f)
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
private fun FanTile(@DrawableRes icon: Int, offset: Dp, tilt: Float) {
    val palette = LocalPalette.current
    Box(
        Modifier.offset(x = offset).rotate(tilt).size(56.dp).background(palette.control, FieldShape).border(1.dp, palette.line, FieldShape),
        contentAlignment = Alignment.Center,
    ) { Glyph(icon, palette.inkMuted, 22) }
}
