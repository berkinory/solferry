package app.solferry.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import uniffi.solferry_ffi.Reach

private val reaches = listOf(
    Triple(Reach.EVERYONE, "Everyone", "Anyone on your network can see this device and ask to send."),
    Triple(Reach.PAIRED, "Paired", "Others can see this device, but only paired devices can send."),
    Triple(Reach.HIDDEN, "Hidden", "This device doesn't show up nearby. Paired devices can still send."),
)

private fun reachOf(visibility: String) = when (visibility) {
    "KnownDevices" -> Reach.PAIRED
    "Hidden" -> Reach.HIDDEN
    else -> Reach.EVERYONE
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(state: AppState, engine: Engine, onBack: () -> Unit) {
    val palette = LocalPalette.current
    val me = state.me
    var name by remember(me?.name) { mutableStateOf(me?.name.orEmpty()) }
    val focus = LocalFocusManager.current
    val save = {
        if (name.isNotBlank() && name != me?.name) engine.bridge.rename(name.trim())
        focus.clearFocus()
    }
    Scaffold(topBar = {
        TopAppBar(
            title = { Text("Settings") },
            navigationIcon = { IconButton(onClick = onBack) { Glyph(R.drawable.ic_arrow_left, MaterialTheme.colorScheme.onSurface, label = "Back") } },
        )
    }) { padding ->
        Column(Modifier.fillMaxSize().padding(padding).verticalScroll(rememberScrollState())) {
            SectionLabel("This device")
            Segmented(2) { index, shape ->
                SegmentRow(shape) {
                    if (index == 0) {
                        OutlinedTextField(
                            value = name, onValueChange = { name = it }, modifier = Modifier.fillMaxWidth(),
                            label = { Text("Name") }, singleLine = true,
                            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                            keyboardActions = KeyboardActions(onDone = { save() }),
                        )
                    } else {
                        Glyph(R.drawable.ic_folder)
                        Column {
                            Text("Received files", style = MaterialTheme.typography.bodyLarge)
                            Text("Downloads/Solferry", color = palette.inkMuted)
                        }
                    }
                }
            }
            SectionLabel("Who can see this device")
            val current = reachOf(me?.visibility.orEmpty())
            Segmented(1) { _, shape ->
                SegmentRow(shape) {
                    Column(Modifier.fillMaxWidth()) {
                        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                            reaches.forEachIndexed { index, (reach, label, _) ->
                                SegmentedButton(
                                    selected = reach == current,
                                    onClick = { engine.setReach(reach) },
                                    shape = SegmentedButtonDefaults.itemShape(index, reaches.size),
                                ) { Text(label) }
                            }
                        }
                        Text(
                            reaches.first { it.first == current }.third,
                            Modifier.padding(top = 10.dp), color = palette.inkMuted,
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                }
            }
            Row(Modifier.fillMaxWidth().padding(horizontal = 28.dp, vertical = 20.dp)) {
                Mono("Solferry 0.1.0")
                Text("", Modifier.weight(1f))
                Mono(me?.fingerprint.orEmpty())
            }
        }
    }
}
