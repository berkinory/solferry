package app.solferry.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.HistoryEntry

@Composable
fun HistoryScreen(state: AppState, engine: Engine) {
    if (state.history.isEmpty()) {
        NoHistory()
    } else {
        Page("History") {
            Card(
                R.drawable.ic_history, "History",
                aside = { Button({ engine.bridge.clearHistory() }, label = "Clear", kind = ButtonKind.Ghost, size = ButtonSize.Sm) },
            ) {
                Well { state.history.forEach { Past(it) } }
            }
        }
    }
}

@Composable
private fun NoHistory() = Column(Modifier.fillMaxSize()) {
    val palette = LocalPalette.current
    Toolbar { Text("History", Modifier.padding(start = SpaceTokens.sm.dp), style = MaterialTheme.typography.titleMedium) }
    Column(
        Modifier.fillMaxWidth().weight(1f).padding(bottom = tabClearance()),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp, Alignment.CenterVertically),
    ) {
        Box(Modifier.padding(bottom = SpaceTokens.md.dp), contentAlignment = Alignment.Center) {
            Box(Modifier.size(56.dp).background(palette.control, FieldShape).border(1.dp, palette.line, FieldShape), contentAlignment = Alignment.Center) {
                Glyph(R.drawable.ic_history, palette.inkMuted, 24)
            }
        }
        Text("No transfers yet", style = MaterialTheme.typography.titleMedium)
        Muted("Files you send or receive show up here.")
    }
}

@Composable
private fun Past(entry: HistoryEntry) {
    val palette = LocalPalette.current
    Row(Modifier.fillMaxWidth().padding(8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp)) {
        Tile(if (entry.sending) R.drawable.ic_upload else R.drawable.ic_download, palette.control, palette.inkMuted)
        Column(Modifier.weight(1f)) {
            Text(entry.label, maxLines = 1, overflow = TextOverflow.Ellipsis)
            val outcome = if (entry.completed) bytes(entry.bytes.toLong()) else "Failed"
            Text(
                "${if (entry.sending) "To" else "From"} ${entry.name}  ·  $outcome  ·  ${ago(entry.finishedMs.toLong())}",
                style = androidx.compose.material3.MaterialTheme.typography.bodyMedium,
                color = if (entry.completed) palette.inkMuted else palette.danger,
                maxLines = 1, overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
