package app.solferry.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.PairedDevice

/** Text to send to one device, starting from what is on the clipboard. */
@Composable
fun TextSheet(device: PairedDevice, engine: Engine, onClose: () -> Unit) = Sheet("Send text", onClose) { close ->
    var text by remember { mutableStateOf(engine.pasted().orEmpty()) }
    Muted("To ${device.name}")
    TextBox(text, { text = it }, "Type or paste text") { engine.pasted()?.let { text += it } }
    Button(
        { engine.bridge.sendText(device.identity.id, text); close() }, Modifier.fillMaxWidth(),
        label = "Send", icon = R.drawable.ic_send, kind = ButtonKind.Primary, enabled = text.isNotBlank(),
    )
}

@Composable
private fun TextBox(value: String, onChange: (String) -> Unit, placeholder: String, onPaste: () -> Unit) {
    val palette = LocalPalette.current
    val typed = MaterialTheme.typography.bodyLarge
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    BasicTextField(
        value, onChange, Modifier.fillMaxWidth(),
        textStyle = typed.copy(color = palette.ink),
        cursorBrush = SolidColor(palette.primary),
        interactionSource = interaction,
        decorationBox = { input ->
            Box(
                Modifier.fillMaxWidth().heightIn(min = 120.dp, max = 280.dp).background(palette.page, ControlShape)
                    .border(1.dp, if (focused) palette.primary else palette.line, ControlShape)
                    .padding(start = SpaceTokens.md.dp, top = SpaceTokens.sm.dp, bottom = SpaceTokens.sm.dp, end = SpaceTokens.xs.dp),
            ) {
                Box(Modifier.fillMaxWidth().padding(end = 40.dp)) {
                    if (value.isEmpty()) Text(placeholder, style = typed, color = palette.inkMuted)
                    input()
                }
                Button(
                    onPaste, Modifier.align(Alignment.TopEnd), icon = R.drawable.ic_clipboard,
                    kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Paste",
                )
            }
        },
    )
}
