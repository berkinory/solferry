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
import androidx.compose.ui.res.stringResource
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
        Page(stringResource(R.string.history_title)) {
            Card(
                R.drawable.ic_history, stringResource(R.string.history_title),
                aside = { Button({ engine.bridge.clearHistory() }, label = stringResource(R.string.history_clear), kind = ButtonKind.Ghost, size = ButtonSize.Sm) },
            ) {
                Well { state.history.forEach { Past(it) } }
            }
        }
    }
}

@Composable
private fun NoHistory() = Column(Modifier.fillMaxSize()) {
    val palette = LocalPalette.current
    Toolbar { Text(stringResource(R.string.history_title), Modifier.padding(start = SpaceTokens.sm.dp), style = MaterialTheme.typography.titleMedium) }
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
        Text(stringResource(R.string.history_empty_title), style = MaterialTheme.typography.titleMedium)
        Muted(stringResource(R.string.history_empty_body))
    }
}

@Composable
private fun Past(entry: HistoryEntry) {
    val palette = LocalPalette.current
    Row(Modifier.fillMaxWidth().padding(8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp)) {
        Tile(if (entry.sending) R.drawable.ic_upload else R.drawable.ic_download, palette.control, palette.inkMuted)
        Column(Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Text(entry.label, Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
                Muted(if (entry.completed) bytes(entry.bytes.toLong()) else stringResource(R.string.history_failed), color = if (entry.completed) palette.inkMuted else palette.danger)
            }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp)) {
                Glyph(platformIcon(entry.platform), palette.inkMuted, 14)
                Text(entry.name, Modifier.weight(1f), style = MaterialTheme.typography.bodyMedium, color = palette.inkMuted, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Caption(ago(entry.finishedMs.toLong()))
            }
        }
    }
}
