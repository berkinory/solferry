package app.solferry.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
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
fun HistoryScreen(state: AppState, engine: Engine) = Page("History", null) {
    if (state.history.isEmpty()) {
        Card(R.drawable.ic_history, "History") {
            PaddedWell {
                Column(Modifier.fillMaxWidth().padding(vertical = SpaceTokens.lg.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text("No transfers yet")
                    Muted("Files you send or receive are listed here.")
                }
            }
        }
    } else {
        Card(
            R.drawable.ic_history, "History",
            aside = { Button({ engine.bridge.clearHistory() }, label = "Clear", kind = ButtonKind.Ghost, size = ButtonSize.Sm) },
        ) {
            Well { state.history.forEach { Past(it) } }
        }
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
