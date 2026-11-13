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
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
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
fun PairRequestSheet(prompt: PairingPrompt, engine: Engine) {
    // Answered once the sheet has slid away, since the answer takes the prompt off the screen.
    var accepted by remember(prompt.key) { mutableStateOf(false) }
    Sheet(stringResource(R.string.pair_request_title), { engine.answerPairing(prompt.key, accepted) }) { close ->
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp)) {
            DeviceTile(prompt.peer)
            Text(stringResource(R.string.pair_request_wants, prompt.name), maxLines = 1, overflow = TextOverflow.Ellipsis)
            Caption(prompt.peer.fingerprint)
        }
        Frame { Well(padding = 12.dp) { Digits(prompt.code) } }
        Muted(stringResource(R.string.pair_request_hint), Modifier.fillMaxWidth().padding(top = SpaceTokens.sm.dp), textAlign = TextAlign.Center)
        Row(Modifier.fillMaxWidth().padding(top = SpaceTokens.md.dp), horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp, Alignment.CenterHorizontally)) {
            Button({ accepted = true; close() }, label = stringResource(R.string.pair_confirm), icon = R.drawable.ic_check, kind = ButtonKind.Primary)
            Button(close, label = stringResource(R.string.common_decline), kind = ButtonKind.Warning)
        }
    }
}

@Composable
fun CompareSheet(code: String, engine: Engine) = Sheet(stringResource(R.string.pair_compare_title), { engine.stopComparing() }) {
    Frame { Well(padding = 12.dp) { Digits(code) } }
    Muted(stringResource(R.string.pair_compare_hint), Modifier.fillMaxWidth().padding(top = SpaceTokens.sm.dp), textAlign = TextAlign.Center)
}

@Composable
fun OfferCard(incoming: Incoming, engine: Engine) {
    val palette = LocalPalette.current
    var blocking by remember { mutableStateOf(false) }
    val count = incoming.files.toInt()
    val files = pluralStringResource(R.plurals.offer_files, count, count)
    Card(R.drawable.ic_download, stringResource(R.string.offer_title), palette.primary) {
        PaddedWell {
            Asker(incoming.sender, stringResource(R.string.offer_wants_to_send, incoming.name), stringResource(R.string.offer_size, files, bytes(incoming.bytes.toLong())))
            if (incoming.files == 1UL) Text(incoming.firstPath, maxLines = 1, overflow = TextOverflow.Ellipsis, color = palette.inkMuted)
            Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Button({ engine.answerOffer(incoming.key, true) }, Modifier.weight(1f), label = stringResource(R.string.offer_accept), kind = ButtonKind.Primary)
                if (incoming.paired) Button({ engine.acceptAlways(incoming.key) }, label = stringResource(R.string.offer_always_accept))
                Button({ engine.answerOffer(incoming.key, false) }, label = stringResource(R.string.common_decline), kind = ButtonKind.Warning)
                if (!incoming.paired) Button({ blocking = true }, label = stringResource(R.string.offer_block), kind = ButtonKind.Danger)
            }
        }
    }
    if (blocking) {
        ConfirmDialog(
            stringResource(R.string.offer_block_title, incoming.name), stringResource(R.string.offer_block_body), stringResource(R.string.offer_block),
            onConfirm = { blocking = false; engine.blockOffer(incoming.key) },
            onDismiss = { blocking = false },
        )
    }
}

@Composable
fun TextCard(shared: ReceivedText, engine: Engine) = Card(R.drawable.ic_clipboard, stringResource(R.string.text_from, shared.name)) {
    PaddedWell {
        Text(linked(shared.text), maxLines = 4, overflow = TextOverflow.Ellipsis)
        Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
            Button(
                { engine.copy(shared.text); engine.markCopied(shared.key) }, label = stringResource(if (shared.copied) R.string.common_copied else R.string.text_copy),
                icon = if (shared.copied) R.drawable.ic_check else R.drawable.ic_copy, size = ButtonSize.Sm,
            )
            Button({ engine.dismissText(shared.key) }, label = stringResource(R.string.common_dismiss), kind = ButtonKind.Ghost, size = ButtonSize.Sm)
        }
    }
}
