package app.solferry.ui

import android.os.Build
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.BuildConfig
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.Reach

@Composable
fun SettingsScreen(state: AppState, engine: Engine) = Page(stringResource(R.string.settings_title)) {
    val me = state.me
    var name by remember(me?.name) { mutableStateOf(me?.name.orEmpty()) }
    val focus = LocalFocusManager.current
    val changed = name.isNotBlank() && name != me?.name
    val save = {
        if (changed) engine.bridge.rename(name.trim())
        focus.clearFocus()
    }
    Card(R.drawable.ic_computer, stringResource(R.string.settings_name_title)) {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Field(
                    name, { name = it }, stringResource(R.string.settings_name_placeholder), Modifier.weight(1f),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                    keyboardActions = KeyboardActions(onDone = { save() }),
                )
                Button(save, label = stringResource(R.string.settings_save), enabled = changed)
            }
        }
    }
    PermissionsCard()
    RelayCard(state, engine)
    SaveFolderCard(state, engine)
    Card(R.drawable.ic_view, stringResource(R.string.settings_find_title)) {
        PaddedWell {
            val found = me?.visibility != "Hidden"
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Muted(
                    stringResource(if (found) R.string.settings_find_on else R.string.settings_find_off),
                    Modifier.weight(1f),
                )
                Toggle(found, { engine.setReach(if (it) Reach.EVERYONE else Reach.HIDDEN) }, stringResource(R.string.settings_find_title))
            }
        }
    }
    BackgroundCard(state, engine)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) LanguageCard()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) AppearanceCard(engine)
    me?.checkUpdates?.let { UpdatesCard(it, state.update, engine) }
    if (state.blocked.isNotEmpty()) BlockedCard(state.blocked, engine)
    Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp)) {
        Caption(stringResource(R.string.settings_version, BuildConfig.VERSION_NAME), Modifier.weight(1f))
        Caption(me?.fingerprint.orEmpty())
    }
}

/** Receiving while Solferry is closed, in settings and in onboarding. */
@Composable
fun BackgroundCard(state: AppState, engine: Engine) =
    Card(R.drawable.ic_download, stringResource(R.string.settings_background_title)) {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Muted(stringResource(R.string.settings_background_body), Modifier.weight(1f))
                Toggle(state.standby, { engine.setStandby(it) }, stringResource(R.string.settings_background_title))
            }
        }
    }
