package app.solferry.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.qrCode

@Composable
fun AddDeviceSheet(
    state: AppState,
    engine: Engine,
    onClose: () -> Unit,
) = Sheet("Pair with a code", { engine.clearJoinError(); onClose() }) {
    var code by remember { mutableStateOf("") }
    val join = { if (code.isNotBlank()) engine.join(code.trim()) }
    Card(R.drawable.ic_qr, "Show a code") {
        PaddedWell {
            val invite = state.invite
            if (invite == null) {
                Muted("For a device on another network, or when this one can't be found nearby. Show a code here and enter it there.")
                Button({ engine.bridge.invite() }, Modifier.fillMaxWidth(), label = "Show code", icon = R.drawable.ic_qr, kind = ButtonKind.Primary)
            } else {
                Invite(invite) { engine.copy(invite) }
            }
        }
    }
    Card(R.drawable.ic_key, "Enter a code") {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Field(
                    code, { code = it }, "Paste a sf1 code", Modifier.weight(1f),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Go),
                    keyboardActions = KeyboardActions(onGo = { join() }),
                    trailing = {
                        Button(
                            { engine.pasted()?.let { code = it.trim() } }, icon = R.drawable.ic_clipboard,
                            kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = "Paste",
                        )
                    },
                )
                Button(join, label = if (state.joining) "Pairing" else "Pair", kind = ButtonKind.Primary, enabled = !state.joining)
            }
            state.joinError?.let { Muted(it, color = LocalPalette.current.danger) }
        }
    }
}

@Composable
private fun ColumnScope.Invite(invite: String, onCopy: () -> Unit) {
    var copied by remember(invite) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
        Qr(invite)
        Muted("Works once, for 10 minutes.")
        Button(
            { onCopy(); copied = true }, label = if (copied) "Copied" else "Copy code",
            icon = if (copied) R.drawable.ic_check else R.drawable.ic_copy, size = ButtonSize.Sm,
        )
    }
}

/** Draws the code on a white plate so it scans in both themes. */
@Composable
fun Qr(text: String) {
    val matrix = remember(text) { qrCode(text) } ?: return
    val width = matrix.width.toInt()
    Canvas(Modifier.size(232.dp).background(Color.White, RoundedCornerShape(16.dp)).padding(16.dp)) {
        val cell = size.width / width
        matrix.dark.forEachIndexed { index, dark ->
            if (dark) {
                drawRect(Color.Black, Offset((index % width) * cell, (index / width) * cell), Size(cell + 0.5f, cell + 0.5f))
            }
        }
    }
}
