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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.solferry.Engine
import app.solferry.PairingPrompt
import app.solferry.R
import app.solferry.ReceivedText
import app.solferry.design.RadiusTokens
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.Identity
import uniffi.solferry_ffi.Incoming

@Composable
private fun Digits(code: String) {
    val palette = LocalPalette.current
    val digits = code.filter { it.isDigit() }
    Row(Modifier.fillMaxWidth().padding(vertical = SpaceTokens.xs.dp), horizontalArrangement = Arrangement.Center) {
        digits.forEachIndexed { index, digit ->
            if (index == digits.length / 2) Spacer(Modifier.width(10.dp))
            Box(
                Modifier.padding(horizontal = 2.dp).size(38.dp, 48.dp).background(palette.page, RoundedCornerShape(RadiusTokens.control.dp)),
                contentAlignment = Alignment.Center,
            ) { Text(digit.toString(), fontSize = 22.sp, fontWeight = FontWeight.Medium) }
        }
    }
}

@Composable
private fun Asker(peer: Identity, title: String, detail: String) = Row(
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(SpaceTokens.md.dp),
) {
    DeviceTile(peer)
    Column(Modifier.weight(1f)) {
        Text(title, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Caption(detail)
    }
}

@Composable
fun PairRequestSheet(prompt: PairingPrompt, engine: Engine) = Sheet("Pairing request", { engine.answerPairing(prompt.key, false) }) { close ->
    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp)) {
        DeviceTile(prompt.peer)
        Text("${prompt.name} wants to pair", maxLines = 1, overflow = TextOverflow.Ellipsis)
        Caption(prompt.peer.fingerprint)
    }
    Frame { Well(padding = 12.dp) { Digits(prompt.code) } }
    Muted("Confirm only if the other screen shows the same code.", Modifier.fillMaxWidth().padding(top = SpaceTokens.sm.dp), textAlign = TextAlign.Center)
    Row(Modifier.fillMaxWidth().padding(top = SpaceTokens.md.dp), horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp, Alignment.CenterHorizontally)) {
        Button({ engine.answerPairing(prompt.key, true); close() }, label = "Confirm", icon = R.drawable.ic_check, kind = ButtonKind.Primary)
        Button({ engine.answerPairing(prompt.key, false); close() }, label = "Decline", kind = ButtonKind.Warning)
    }
}

@Composable
fun CompareSheet(code: String, engine: Engine) = Sheet("Compare codes", { engine.stopComparing() }) {
    Frame { Well(padding = 12.dp) { Digits(code) } }
    Muted("Check that the other device shows this code, then accept there.", Modifier.fillMaxWidth().padding(top = SpaceTokens.sm.dp), textAlign = TextAlign.Center)
}

@Composable
fun OfferCard(incoming: Incoming, engine: Engine) {
    val palette = LocalPalette.current
    var blocking by remember { mutableStateOf(false) }
    val files = if (incoming.files == 1UL) "1 file" else "${incoming.files} files"
    Card(R.drawable.ic_download, "Incoming files", palette.primary) {
        PaddedWell {
            Asker(incoming.sender, "${incoming.name} wants to send", "$files, ${bytes(incoming.bytes.toLong())}" + if (incoming.paired) "" else "  ·  ${incoming.sender.fingerprint}")
            if (incoming.files == 1UL) Text(incoming.firstPath, maxLines = 1, overflow = TextOverflow.Ellipsis, color = palette.inkMuted)
            Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Button({ engine.answerOffer(incoming.key, true) }, Modifier.weight(1f), label = "Accept", kind = ButtonKind.Primary)
                if (incoming.paired) Button({ engine.acceptAlways(incoming.key) }, label = "Always accept")
                Button({ engine.answerOffer(incoming.key, false) }, label = "Decline", kind = ButtonKind.Warning)
                if (!incoming.paired) Button({ blocking = true }, label = "Block", kind = ButtonKind.Danger)
            }
        }
    }
    if (blocking) {
        ConfirmDialog(
            "Block ${incoming.name}?", "It can't send files or ask to pair. You can unblock it later.", "Block",
            onConfirm = { blocking = false; engine.blockOffer(incoming.key) },
            onDismiss = { blocking = false },
        )
    }
}

@Composable
fun TextCard(shared: ReceivedText, engine: Engine) = Card(R.drawable.ic_clipboard, "Text from ${shared.name}") {
    PaddedWell {
        Text(shared.text, maxLines = 4, overflow = TextOverflow.Ellipsis)
        Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
            Button(
                { engine.copy(shared.text); engine.markCopied(shared.key) }, label = if (shared.copied) "Copied" else "Copy",
                icon = if (shared.copied) R.drawable.ic_check else R.drawable.ic_copy, size = ButtonSize.Sm,
            )
            Button({ engine.dismissText(shared.key) }, label = "Dismiss", kind = ButtonKind.Ghost, size = ButtonSize.Sm)
        }
    }
}
