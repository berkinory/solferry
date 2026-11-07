package app.solferry.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.solferry.Engine
import app.solferry.PairingPrompt
import app.solferry.R
import app.solferry.ReceivedText
import uniffi.solferry_ffi.Incoming
import uniffi.solferry_ffi.Problem

@Composable
private fun Digits(code: String) {
    val digits = code.filter { it.isDigit() }
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
        digits.forEachIndexed { index, digit ->
            if (index == digits.length / 2) Spacer(Modifier.width(10.dp))
            Box(
                Modifier.padding(horizontal = 2.dp).size(38.dp, 48.dp).clip(RoundedCornerShape(8.dp))
                    .background(MaterialTheme.colorScheme.surface),
                contentAlignment = Alignment.Center,
            ) { Text(digit.toString(), fontFamily = FontFamily.Monospace, fontSize = 22.sp, fontWeight = FontWeight.Medium) }
        }
    }
}

@Composable
fun PairingCard(prompt: PairingPrompt, engine: Engine, autoAccept: Boolean) {
    val palette = LocalPalette.current
    Notice(R.drawable.ic_shield, "Pairing request", palette.primary, palette.well) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            DeviceTile(prompt.peer)
            Column {
                Text("${prompt.name} wants to pair", style = MaterialTheme.typography.bodyLarge)
                Mono(prompt.peer.fingerprint)
            }
        }
        Digits(prompt.code)
        Text("Accept only if the other screen shows the same code.", style = MaterialTheme.typography.bodyMedium, color = palette.inkMuted)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { engine.answerPairing(prompt.key, true, autoAccept) }) { Text("Same code") }
            TextButton(onClick = { engine.answerPairing(prompt.key, false, autoAccept) }) { Text("Decline") }
        }
    }
}

@Composable
fun CompareCard(code: String) {
    val palette = LocalPalette.current
    Notice(R.drawable.ic_shield, "Compare codes", palette.primary, palette.well) {
        Digits(code)
        Text("Check that the other device shows this code, then accept there.", style = MaterialTheme.typography.bodyMedium, color = palette.inkMuted)
    }
}

@Composable
fun OfferCard(incoming: Incoming, engine: Engine) {
    val palette = LocalPalette.current
    var blocking by remember { mutableStateOf(false) }
    val files = if (incoming.files == 1UL) "1 file" else "${incoming.files} files"
    Notice(R.drawable.ic_download, "Incoming files", palette.primary, palette.well) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            DeviceTile(incoming.sender)
            Column(Modifier.weight(1f)) {
                Text("${incoming.name} wants to send", style = MaterialTheme.typography.bodyLarge)
                Mono("$files, ${bytes(incoming.bytes.toLong())}  ·  ${incoming.sender.fingerprint}")
            }
        }
        Text(incoming.firstPath, maxLines = 1, overflow = TextOverflow.Ellipsis, color = palette.inkMuted)
        if (!incoming.paired) {
            Text("Not paired. Accept only if you expected this.", color = palette.warning, style = MaterialTheme.typography.bodyMedium)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { engine.answerOffer(incoming.key, true) }) { Text("Accept") }
            FilledTonalButton(onClick = { engine.answerOffer(incoming.key, false) }) { Text("Decline") }
            if (!incoming.paired) TextButton(onClick = { blocking = true }) { Text("Block") }
        }
    }
    if (blocking) {
        AlertDialog(
            onDismissRequest = { blocking = false },
            title = { Text("Block ${incoming.name}?") },
            text = { Text("It can't send files or ask to pair. You can unblock it later.") },
            confirmButton = { TextButton(onClick = { blocking = false; engine.blockOffer(incoming.key) }) { Text("Block", color = palette.danger) } },
            dismissButton = { TextButton(onClick = { blocking = false }) { Text("Cancel") } },
        )
    }
}

@Composable
fun TextCard(shared: ReceivedText, engine: Engine) {
    val palette = LocalPalette.current
    Notice(R.drawable.ic_clipboard, "Text from ${shared.name}", palette.inkMuted, palette.well) {
        Text(shared.text, maxLines = 4, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.bodyLarge)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            FilledTonalButton(onClick = { engine.copy(shared.text); engine.markCopied(shared.key) }) {
                Glyph(if (shared.copied) R.drawable.ic_check else R.drawable.ic_copy, size = 18)
                Spacer(Modifier.width(8.dp))
                Text(if (shared.copied) "Copied" else "Copy")
            }
            TextButton(onClick = { engine.dismissText(shared.key) }) { Text("Dismiss") }
        }
    }
}

@Composable
fun ProblemCard(problem: Problem, about: String?, onDismiss: (() -> Unit)?) {
    val palette = LocalPalette.current
    Notice(R.drawable.ic_alert, problem.title, palette.danger, palette.dangerWell) {
        Text(listOfNotNull(about, problem.next).joinToString(". "), style = MaterialTheme.typography.bodyMedium, color = palette.inkMuted)
        if (onDismiss != null) TextButton(onClick = onDismiss) { Text("Dismiss") }
    }
}
