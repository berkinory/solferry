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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.LinkState
import app.solferry.R
import app.solferry.design.SpaceTokens

/** A link for a browser on the same network. Closing the sheet stops it. */
@Composable
fun LinkSheet(link: LinkState, engine: Engine) =
    Sheet(if (link.receiving) "Receive via link" else "Share via link", { engine.bridge.stopLink() }) { close ->
        val url = link.urls.first()
        var copied by remember(url) { mutableStateOf(false) }
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
            Qr(url)
            Text(url, style = MaterialTheme.typography.bodyMedium, textAlign = TextAlign.Center)
            Muted(
                if (link.receiving) "Open it in a browser on the same network to send files here." else "Open it in a browser on the same network to download the files.",
                textAlign = TextAlign.Center,
            )
            Button(
                { engine.copy(url); copied = true }, label = if (copied) "Copied" else "Copy link",
                icon = if (copied) R.drawable.ic_check else R.drawable.ic_copy, size = ButtonSize.Sm,
            )
        }
        if (link.received.isNotEmpty()) {
            Card(R.drawable.ic_download, "Received") {
                Well { link.received.forEach { Muted(it, Modifier.padding(8.dp)) } }
            }
        }
        Button(close, Modifier.fillMaxWidth(), label = "Stop", kind = ButtonKind.Danger)
    }

@Composable
fun BrowserCard(engine: Engine) = Card(R.drawable.ic_link, "No app on the other device") {
    PaddedWell {
        Muted("A browser on the same network can send you files. To send files to one, pick them and choose Share via link.")
        Button({ engine.bridge.receiveLink() }, Modifier.fillMaxWidth(), label = "Receive via link", icon = R.drawable.ic_download)
    }
}
