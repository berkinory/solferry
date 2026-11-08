package app.solferry

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.core.content.IntentCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import app.solferry.design.MotionTokens
import app.solferry.ui.AddDeviceScreen
import app.solferry.ui.HomeScreen
import app.solferry.ui.SettingsScreen
import app.solferry.ui.SolferryTheme
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.solferry_ffi.PairedDevice

private enum class Screen { Home, AddDevice, Settings }

private const val PRESENCE_EVERY_MS = 20_000L

class MainActivity : ComponentActivity() {
    private var shared by mutableStateOf<List<Uri>>(emptyList())

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        shared = sharedUris(intent)
        val engine = (application as SolferryApp).engine
        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                while (true) {
                    engine.bridge.checkPresence()
                    delay(PRESENCE_EVERY_MS)
                }
            }
        }
        setContent {
            SolferryTheme {
                val state by engine.state.collectAsStateWithLifecycle()
                var screen by rememberSaveable { mutableStateOf(Screen.Home) }
                var target by remember { mutableStateOf<PairedDevice?>(null) }
                val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
                    if (uris.isEmpty()) return@rememberLauncherForActivityResult
                    val device = target
                    if (device != null) engine.sendUris(device.identity.id, uris) else shared = uris
                }
                val notifications = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}
                LaunchedEffect(Unit) {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        notifications.launch(Manifest.permission.POST_NOTIFICATIONS)
                    }
                }
                LaunchedEffect(state.pairedCount) { if (state.pairedCount > 0) screen = Screen.Home }
                LaunchedEffect(shared) { if (shared.isNotEmpty()) screen = Screen.Home }
                BackHandler(enabled = screen != Screen.Home || shared.isNotEmpty()) {
                    if (screen != Screen.Home) screen = Screen.Home else shared = emptyList()
                }
                AnimatedContent(
                    screen,
                    transitionSpec = { fadeIn(tween(MotionTokens.standard)) togetherWith fadeOut(tween(MotionTokens.quick)) },
                ) { shown ->
                    when (shown) {
                        Screen.Home -> HomeScreen(
                            state, engine, shared,
                            onPickFiles = { device -> target = device; picker.launch(arrayOf("*/*")) },
                            onSendStaged = { device -> engine.sendUris(device.identity.id, shared); shared = emptyList() },
                            onClearStaged = { shared = emptyList() },
                            onUseCode = { screen = Screen.AddDevice },
                            onSettings = { screen = Screen.Settings },
                        )
                        Screen.AddDevice -> AddDeviceScreen(state, engine) { screen = Screen.Home }
                        Screen.Settings -> SettingsScreen(state, engine) { screen = Screen.Home }
                    }
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        sharedUris(intent).takeIf { it.isNotEmpty() }?.let { shared = it }
    }

    private fun sharedUris(intent: Intent?): List<Uri> = when (intent?.action) {
        Intent.ACTION_SEND -> listOfNotNull(IntentCompat.getParcelableExtra(intent, Intent.EXTRA_STREAM, Uri::class.java))
        Intent.ACTION_SEND_MULTIPLE ->
            IntentCompat.getParcelableArrayListExtra(intent, Intent.EXTRA_STREAM, Uri::class.java).orEmpty()
        else -> emptyList()
    }
}
