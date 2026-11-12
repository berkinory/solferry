package app.solferry.ui

import android.content.Intent
import android.os.Build
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.LifecycleResumeEffect
import app.solferry.Notifier
import app.solferry.R

/**
 * What Android has to allow Solferry. Notifications are the one permission it asks for: they
 * carry offers and pairing requests, and keep receiving alive in the background.
 */
@Composable
fun PermissionsCard() {
    val context = LocalContext.current
    val palette = LocalPalette.current
    var allowed by remember { mutableStateOf(Notifier.allowed(context)) }
    // Read again on every return, as after a trip to the system settings.
    LifecycleResumeEffect(Unit) {
        allowed = Notifier.allowed(context)
        onPauseOrDispose {}
    }
    // Android stops showing its prompt after a refusal or two, so after one the button opens settings.
    var asked by rememberSaveable { mutableStateOf(false) }
    val request = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {
        allowed = it
        asked = true
    }
    val canAsk = Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && !asked
    Card(R.drawable.ic_shield, stringResource(R.string.permissions_title)) {
        PaddedWell {
            Row(verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Tile(R.drawable.ic_notification, palette.control, palette.inkMuted)
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(stringResource(R.string.permissions_notifications))
                    if (allowed) {
                        StatusLine(stringResource(R.string.permissions_allowed), palette.successFill)
                    } else {
                        StatusLine(stringResource(R.string.permissions_off), palette.danger)
                    }
                    Muted(stringResource(R.string.permissions_notifications_detail))
                    if (!allowed && canAsk) {
                        Button({ request.launch(android.Manifest.permission.POST_NOTIFICATIONS) }, label = stringResource(R.string.permissions_allow), kind = ButtonKind.Primary, size = ButtonSize.Sm)
                    } else if (!allowed) {
                        Button({
                            context.startActivity(
                                Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName),
                            )
                        }, label = stringResource(R.string.permissions_open_settings), size = ButtonSize.Sm)
                    }
                }
            }
        }
    }
}
