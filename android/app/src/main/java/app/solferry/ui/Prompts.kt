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
import uniffi.solferry_ffi.Problem

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
fun PairingCard(prompt: PairingPrompt, engine: Engine) = Card(R.drawable.ic_shield, "Pairing request", LocalPalette.current.primary) {
    PaddedWell {
        Asker(prompt.peer, "${prompt.name} wants to pair", prompt.peer.fingerprint)
        Digits(prompt.code)
        Muted("Accept only if the other screen shows the same code.")
        Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
            Button({ engine.answerPairing(prompt.key, true) }, Modifier.weight(1f), label = "Same code", kind = ButtonKind.Primary)
            Button({ engine.answerPairing(prompt.key, false) }, label = "Decline")
        }
    }
}

@Composable
fun CompareCard(code: String) = Card(R.drawable.ic_shield, "Compare codes", LocalPalette.current.primary) {
    PaddedWell {
        Digits(code)
        Muted("Check that the other device shows this code, then accept there.")
    }
}

@Composable
fun OfferCard(incoming: Incoming, engine: Engine) {
    val palette = LocalPalette.current
    var blocking by remember { mutableStateOf(false) }
    val files = if (incoming.files == 1UL) "1 file" else "${incoming.files} files"
    Card(R.drawable.ic_download, "Incoming files", palette.primary) {
        PaddedWell {
            Asker(incoming.sender, "${incoming.name} wants to send", "$files, ${bytes(incoming.bytes.toLong())}  ·  ${incoming.sender.fingerprint}")
            Text(incoming.firstPath, maxLines = 1, overflow = TextOverflow.Ellipsis, color = palette.inkMuted)
            if (!incoming.paired) Muted("Not paired. Accept only if you expected this.", color = palette.warning)
            Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Button({ engine.answerOffer(incoming.key, true) }, Modifier.weight(1f), label = "Accept", kind = ButtonKind.Primary)
                if (incoming.paired) Button({ engine.acceptAlways(incoming.key) }, label = "Always accept")
                Button({ engine.answerOffer(incoming.key, false) }, label = "Decline")
                if (!incoming.paired) Button({ blocking = true }, label = "Block", kind = ButtonKind.Ghost)
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

@Composable
fun ProblemCard(problem: Problem, about: String?, onDismiss: (() -> Unit)?) {
    val palette = LocalPalette.current
    Card(R.drawable.ic_alert, problem.title, palette.danger, palette.danger.copy(alpha = 0.5f)) {
        PaddedWell(palette.dangerWell) {
            Muted(listOfNotNull(about, problem.next).joinToString(". "))
            if (onDismiss != null) Button(onDismiss, label = "Dismiss", kind = ButtonKind.Ghost, size = ButtonSize.Sm)
        }
    }
}
