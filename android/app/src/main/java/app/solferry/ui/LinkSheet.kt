package app.solferry.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.LinkState
import app.solferry.R
import app.solferry.design.SpaceTokens

/** A link for a browser on the same network. Closing the sheet stops it. */
@Composable
fun LinkSheet(link: LinkState, engine: Engine) =
    Sheet(stringResource(if (link.receiving) R.string.link_receive else R.string.link_share), { engine.bridge.stopLink() }) { close ->
        val url = link.urls.first()
        var copied by remember(url) { mutableStateOf(false) }
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
            Qr(url)
            Text(url, style = MaterialTheme.typography.bodyMedium, textAlign = TextAlign.Center)
            Muted(
                stringResource(if (link.receiving) R.string.link_receive_hint else R.string.link_share_hint),
                textAlign = TextAlign.Center,
            )
            Button(
                { engine.copy(url); copied = true }, label = stringResource(if (copied) R.string.common_copied else R.string.link_copy),
                icon = if (copied) R.drawable.ic_check else R.drawable.ic_copy, size = ButtonSize.Sm,
            )
        }
        if (link.received.isNotEmpty()) {
            Card(R.drawable.ic_download, stringResource(R.string.link_received)) {
                Well { link.received.forEach { Muted(it, Modifier.padding(8.dp)) } }
            }
        }
        Button(close, Modifier.fillMaxWidth(), label = stringResource(R.string.link_stop), kind = ButtonKind.Danger)
    }

@Composable
fun BrowserCard(engine: Engine) = Card(R.drawable.ic_link, stringResource(R.string.browser_title)) {
    PaddedWell {
        Muted(stringResource(R.string.browser_body))
        Button({ engine.bridge.receiveLink() }, Modifier.fillMaxWidth(), label = stringResource(R.string.link_receive), icon = R.drawable.ic_download)
    }
}
