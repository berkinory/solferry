package app.solferry.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.R
import uniffi.solferry_ffi.QueueEntry
import uniffi.solferry_ffi.queueDays

/** Sends waiting for their devices, each with a way to take it back. */
@Composable
fun QueueCard(queue: List<QueueEntry>, engine: Engine) =
    Card(R.drawable.ic_send, stringResource(R.string.queue_title), aside = { Count(queue.size) }) {
        Well {
            queue.forEach { entry -> key(entry.id) { QueueRow(entry, engine) } }
            Caption(stringResource(R.string.queue_hint, queueDays().toInt()), Modifier.padding(horizontal = 8.dp, vertical = 4.dp))
        }
    }

@Composable
private fun QueueRow(entry: QueueEntry, engine: Engine) {
    val palette = LocalPalette.current
    Row(
        Modifier.fillMaxWidth().padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Tile(fileIcon(entry.label), palette.control, palette.inkMuted)
        Column(Modifier.weight(1f)) {
            Text(entry.label, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Muted(stringResource(R.string.queue_to, entry.name))
        }
        Button(
            { engine.bridge.unqueue(entry.id) }, icon = R.drawable.ic_cancel,
            kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = stringResource(R.string.queue_cancel),
        )
    }
}
