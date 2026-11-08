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
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.core.content.IntentCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import app.solferry.design.MotionTokens
import app.solferry.ui.AddDeviceSheet
import app.solferry.ui.HistoryScreen
import app.solferry.ui.HomeScreen
import app.solferry.ui.SendSheet
import app.solferry.ui.SettingsScreen
import app.solferry.ui.SolferryTheme
import app.solferry.ui.Tab
import app.solferry.ui.TabBar
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.solferry_ffi.PairedDevice

private enum class Screen { Home, History, Settings }

private const val PRESENCE_EVERY_MS = 20_000L

class MainActivity : ComponentActivity() {
    private var shared by mutableStateOf<List<Uri>>(emptyList())

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        shared = sharedUris(intent)
        val engine = (application as SolferryApp).engine
        engine.resumeStandby()
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
                var pairing by rememberSaveable { mutableStateOf(false) }
                var target by remember { mutableStateOf<PairedDevice?>(null) }
                val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
                    if (uris.isEmpty()) return@rememberLauncherForActivityResult
                    val device = target
                    if (device != null) engine.sendUris(device.identity.id, uris) else shared = (shared + uris).distinct()
                }
                val notifications = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}
                LaunchedEffect(Unit) {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        notifications.launch(Manifest.permission.POST_NOTIFICATIONS)
                    }
                }
                LaunchedEffect(state.pairedCount) {
                    if (state.pairedCount > 0 && shared.isEmpty()) {
                        pairing = false
                        screen = Screen.Home
                    }
                }
                LaunchedEffect(state.offers.size + state.pairings.size) {
                    if ((state.offers.isNotEmpty() || state.pairings.isNotEmpty()) && shared.isEmpty()) screen = Screen.Home
                }
                BackHandler(enabled = screen != Screen.Home) { screen = Screen.Home }
                Box(Modifier.fillMaxSize()) {
                    AnimatedContent(
                        screen,
                        Modifier.fillMaxSize(),
                        transitionSpec = { fadeIn(tween(MotionTokens.standard)) togetherWith fadeOut(tween(MotionTokens.quick)) },
                    ) { shown ->
                        when (shown) {
                            Screen.Home -> HomeScreen(
                                state, engine,
                                onPickFiles = { device -> target = device; picker.launch(arrayOf("*/*")) },
                                onUseCode = { pairing = true },
                            )
                            Screen.History -> HistoryScreen(state, engine)
                            Screen.Settings -> SettingsScreen(state, engine)
                        }
                    }
                    if (shared.isNotEmpty()) {
                        SendSheet(
                            state,
                            files = remember(shared) { shared.map(engine::describe) },
                            onRemove = { file -> shared = shared - file.uri },
                            onSend = { peers -> peers.forEach { engine.sendUris(it, shared) } },
                            onClose = { shared = emptyList() },
                        )
                    }
                    if (pairing) AddDeviceSheet(state, engine) { pairing = false }
                    TabBar(
                        listOf(
                            Tab("Transfer", R.drawable.ic_home, screen == Screen.Home) { screen = Screen.Home },
                            Tab("History", R.drawable.ic_history, screen == Screen.History) { screen = Screen.History },
                            Tab("Settings", R.drawable.ic_settings, screen == Screen.Settings) { screen = Screen.Settings },
                        ),
                        Modifier.align(Alignment.BottomCenter),
                    )
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
