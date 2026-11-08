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
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.core.content.IntentCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import app.solferry.design.MotionTokens
import app.solferry.ui.AddDeviceScreen
import app.solferry.ui.HistoryScreen
import app.solferry.ui.HomeScreen
import app.solferry.ui.SendScreen
import app.solferry.ui.SettingsScreen
import app.solferry.ui.SolferryTheme
import app.solferry.ui.Tab
import app.solferry.ui.TabBar
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.solferry_ffi.PairedDevice

private enum class Screen { Home, History, Settings, AddDevice, Send }

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
                var target by remember { mutableStateOf<PairedDevice?>(null) }
                val leave = { shared = emptyList(); screen = Screen.Home }
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
                LaunchedEffect(state.pairedCount) { if (state.pairedCount > 0 && screen != Screen.Send) screen = Screen.Home }
                LaunchedEffect(shared) { if (shared.isNotEmpty()) screen = Screen.Send }
                LaunchedEffect(state.offers.size + state.pairings.size) {
                    if ((state.offers.isNotEmpty() || state.pairings.isNotEmpty()) && screen != Screen.Send) screen = Screen.Home
                }
                BackHandler(enabled = screen != Screen.Home) { leave() }
                Column(Modifier.fillMaxSize()) {
                    AnimatedContent(
                        screen,
                        Modifier.weight(1f),
                        transitionSpec = {
                            val enter = if (targetState == Screen.Send) slideInVertically(tween(MotionTokens.slow)) { it / 3 } + fadeIn(tween(MotionTokens.standard)) else fadeIn(tween(MotionTokens.standard))
                            enter togetherWith fadeOut(tween(MotionTokens.quick))
                        },
                    ) { shown ->
                        when (shown) {
                            Screen.Home -> HomeScreen(
                                state, engine,
                                onPickFiles = { device -> target = device; picker.launch(arrayOf("*/*")) },
                                onUseCode = { screen = Screen.AddDevice },
                            )
                            Screen.History -> HistoryScreen(state, engine)
                            Screen.Settings -> SettingsScreen(state, engine)
                            Screen.AddDevice -> AddDeviceScreen(state, engine) { screen = Screen.Home }
                            Screen.Send -> SendScreen(
                                state,
                                files = remember(shared) { shared.map(engine::describe) },
                                onRemove = { file -> shared = shared - file.uri; if (shared.isEmpty()) screen = Screen.Home },
                                onAddMore = { target = null; picker.launch(arrayOf("*/*")) },
                                onSend = { peers -> peers.forEach { engine.sendUris(it, shared) }; leave() },
                                onBack = { leave() },
                            )
                        }
                    }
                    if (screen in listOf(Screen.Home, Screen.History, Screen.Settings)) {
                        TabBar(
                            listOf(
                                Tab("Home", R.drawable.ic_home, screen == Screen.Home) { screen = Screen.Home },
                                Tab("History", R.drawable.ic_history, screen == Screen.History) { screen = Screen.History },
                                Tab("Settings", R.drawable.ic_settings, screen == Screen.Settings) { screen = Screen.Settings },
                            ),
                        )
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
