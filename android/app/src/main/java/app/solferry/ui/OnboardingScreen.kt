package app.solferry.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens

/** The first screen on a fresh install: name this device, then go home. */
@Composable
fun OnboardingScreen(state: AppState, engine: Engine, onDone: () -> Unit) {
    val current = state.me?.name
    var name by rememberSaveable(current) { mutableStateOf(current.orEmpty()) }
    val ready = name.isNotBlank()
    val start = {
        if (ready) {
            if (name.trim() != current) engine.bridge.rename(name.trim())
            engine.finishOnboarding()
            onDone()
        }
    }
    BoxWithConstraints(Modifier.fillMaxSize().safeDrawingPadding()) { Column(
        Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).heightIn(min = maxHeight)
            .padding(horizontal = SpaceTokens.lg.dp, vertical = SpaceTokens.xl.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp, Alignment.CenterVertically),
    ) {
        Glyph(R.drawable.ic_mark, LocalPalette.current.brand, 56)
        Text(stringResource(R.string.onboarding_title), style = MaterialTheme.typography.titleLarge, textAlign = TextAlign.Center)
        Muted(stringResource(R.string.onboarding_body), textAlign = TextAlign.Center)
        Spacer(Modifier.height(SpaceTokens.md.dp))
        Card(R.drawable.ic_computer, stringResource(R.string.onboarding_name_title)) {
            PaddedWell {
                Field(
                    name, { name = it }, stringResource(R.string.settings_name_placeholder), Modifier.fillMaxWidth(),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                    keyboardActions = KeyboardActions(onDone = { start() }),
                )
                Muted(stringResource(R.string.onboarding_name_hint))
            }
        }
        Spacer(Modifier.height(SpaceTokens.xs.dp))
        Button(start, Modifier.fillMaxWidth(), label = stringResource(R.string.onboarding_start), kind = ButtonKind.Primary, enabled = ready)
    } }
}
