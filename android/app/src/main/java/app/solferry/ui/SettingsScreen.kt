package app.solferry.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
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

@Composable
fun SettingsScreen(state: AppState, engine: Engine, onBack: () -> Unit) = Page("Settings", onBack) {
    val me = state.me
    var name by remember(me?.name) { mutableStateOf(me?.name.orEmpty()) }
    val focus = LocalFocusManager.current
    val changed = name.isNotBlank() && name != me?.name
    val save = {
        if (changed) engine.bridge.rename(name.trim())
        focus.clearFocus()
    }
    Card(R.drawable.ic_computer, "Device name") {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Field(
                    name, { name = it }, "Name", Modifier.weight(1f),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                    keyboardActions = KeyboardActions(onDone = { save() }),
                )
                Button(save, label = "Save", enabled = changed)
            }
        }
    }
    Card(R.drawable.ic_folder, "Save files to") {
        PaddedWell { Muted("Downloads/Solferry") }
    }
    Card(R.drawable.ic_view, "Who can see this device") {
        PaddedWell {
            val current = reachOf(me?.visibility.orEmpty())
            Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp)) {
                reaches.forEach { (reach, label, _) ->
                    val chosen = reach == current
                    Button(
                        { engine.setReach(reach) }, Modifier.weight(1f).semantics { selected = chosen }, label = label,
                        kind = if (chosen) ButtonKind.Primary else ButtonKind.Ghost, size = ButtonSize.Sm,
                    )
                }
            }
            Muted(reaches.first { it.first == current }.third)
        }
    }
    RelayCard(state, engine)
    if (state.blocked.isNotEmpty()) BlockedCard(state.blocked, engine)
    Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp)) {
        Mono("Solferry 0.1.0", Modifier.weight(1f))
        Mono(me?.fingerprint.orEmpty())
    }
}
