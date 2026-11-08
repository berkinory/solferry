package app.solferry.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import uniffi.solferry_ffi.qrCode

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddDeviceScreen(
    state: AppState,
    engine: Engine,
    autoAccept: Boolean,
    onAutoAccept: (Boolean) -> Unit,
    onBack: () -> Unit,
) {
    val palette = LocalPalette.current
    var code by remember { mutableStateOf("") }
    var copied by remember { mutableStateOf(false) }
    val clipboard = LocalClipboardManager.current
    val join = { if (code.isNotBlank()) engine.join(code.trim(), autoAccept) }
    Scaffold(topBar = {
        TopAppBar(
            title = { Text("Add device") },
            navigationIcon = { IconButton(onClick = onBack) { Glyph(R.drawable.ic_arrow_left, MaterialTheme.colorScheme.onSurface, label = "Back") } },
        )
    }) { padding ->
        Column(Modifier.fillMaxSize().padding(padding).verticalScroll(rememberScrollState())) {
            state.pairingProblem?.let { ProblemCard(it, null, null) }
            SectionLabel("Show a code")
            Segmented(1) { _, shape ->
                Column(
                    Modifier.fillMaxWidth().clip(shape).background(MaterialTheme.colorScheme.surfaceContainer).padding(16.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    val invite = state.invite
                    if (invite == null) {
                        Text(
                            "Nearby devices pair from the home screen. For anything else, show a code here and enter it there.",
                            color = palette.inkMuted,
                        )
                        FilledTonalButton(onClick = { engine.bridge.invite(autoAccept) }) {
                            Glyph(R.drawable.ic_qr, size = 18)
                            Spacer(Modifier.width(8.dp))
                            Text("Show code")
                        }
                    } else {
                        Qr(invite)
                        Text("Works once, for 10 minutes.", color = palette.inkMuted)
                        FilledTonalButton(onClick = { clipboard.setText(androidx.compose.ui.text.AnnotatedString(invite)); copied = true }) {
                            Glyph(if (copied) R.drawable.ic_check else R.drawable.ic_copy, size = 18)
                            Spacer(Modifier.width(8.dp))
                            Text(if (copied) "Copied" else "Copy code")
                        }
                    }
                }
            }
            SectionLabel("Enter a code")
            Segmented(1) { _, shape ->
                Row(
                    Modifier.fillMaxWidth().clip(shape).background(MaterialTheme.colorScheme.surfaceContainer).padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    OutlinedTextField(
                        value = code,
                        onValueChange = { code = it },
                        modifier = Modifier.weight(1f),
                        placeholder = { Text("Paste a sf1 code") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Go),
                        keyboardActions = KeyboardActions(onGo = { join() }),
                        trailingIcon = {
                            IconButton(onClick = { clipboard.getText()?.text?.let { code = it.trim() } }) { Glyph(R.drawable.ic_clipboard, label = "Paste") }
                        },
                    )
                    Button(onClick = join, enabled = !state.joining) { Text(if (state.joining) "Pairing" else "Pair") }
                }
            }
            SectionLabel("Trust")
            Segmented(1) { _, shape ->
                SegmentRow(shape) {
                    Column(Modifier.weight(1f)) {
                        Text("Receive without asking", style = MaterialTheme.typography.bodyLarge)
                        Text("You can change this for each device later.", color = palette.inkMuted, style = MaterialTheme.typography.bodyMedium)
                    }
                    Switch(autoAccept, onAutoAccept)
                }
            }
        }
    }
}

/** Draws the code on a white plate so it scans in both themes. */
@Composable
private fun Qr(text: String) {
    val matrix = remember(text) { qrCode(text) } ?: return
    val width = matrix.width.toInt()
    Canvas(Modifier.size(232.dp).clip(RoundedCornerShape(16.dp)).background(Color.White).padding(16.dp)) {
        val cell = size.width / width
        matrix.dark.forEachIndexed { index, dark ->
            if (dark) {
                drawRect(Color.Black, Offset((index % width) * cell, (index / width) * cell), Size(cell + 0.5f, cell + 0.5f))
            }
        }
    }
}
