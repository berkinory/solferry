package app.solferry.ui

import android.app.UiModeManager
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens

private val CHOICES = listOf(
    UiModeManager.MODE_NIGHT_AUTO to R.string.appearance_system,
    UiModeManager.MODE_NIGHT_NO to R.string.appearance_light,
    UiModeManager.MODE_NIGHT_YES to R.string.appearance_dark,
)

/**
 * Light, dark or the system's. Android keeps the choice and redraws the app in it, system bars and
 * dialogs included, which it can only do from Android 12.
 */
@RequiresApi(Build.VERSION_CODES.S)
@Composable
fun AppearanceCard(engine: Engine) {
    var chosen by remember { mutableIntStateOf(engine.appearance) }
    Card(R.drawable.ic_appearance, stringResource(R.string.appearance_title)) {
        PaddedWell {
            Row(horizontalArrangement = Arrangement.spacedBy(SpaceTokens.xs.dp)) {
                CHOICES.forEach { (mode, label) ->
                    Button(
                        {
                            chosen = mode
                            engine.appearance = mode
                        },
                        label = stringResource(label),
                        kind = if (chosen == mode) ButtonKind.Secondary else ButtonKind.Ghost,
                        size = ButtonSize.Sm,
                    )
                }
            }
        }
    }
}
