package app.solferry.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
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

@Composable
fun SettingsScreen(state: AppState, engine: Engine) = Page("Settings") {
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
    Card(R.drawable.ic_view, "Let others find me") {
        PaddedWell {
            val found = me?.visibility != "Hidden"
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Muted(
                    if (found) "Devices on your network can see this one and ask to send." else "Nobody new can find this device. Paired devices can still reach it.",
                    Modifier.weight(1f),
                )
                Toggle(found, { engine.setReach(if (it) Reach.EVERYONE else Reach.HIDDEN) }, "Let others find me")
            }
        }
    }
    Card(R.drawable.ic_download, "Receive in the background") {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Muted("Stay reachable while Solferry is closed. Shows a notification and uses more battery.", Modifier.weight(1f))
                Toggle(state.standby, { engine.setStandby(it) }, "Receive in the background")
            }
        }
    }
    RelayCard(state, engine)
    if (state.blocked.isNotEmpty()) BlockedCard(state.blocked, engine)
    Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp)) {
        Caption("Solferry 0.1.0", Modifier.weight(1f))
        Caption(me?.fingerprint.orEmpty())
    }
}
