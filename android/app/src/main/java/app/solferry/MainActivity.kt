package app.solferry

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.statusBars
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.core.content.IntentCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import app.solferry.design.MotionTokens
import app.solferry.ui.AddDeviceSheet
import app.solferry.ui.CompareSheet
import app.solferry.ui.HistoryScreen
import app.solferry.ui.HomeScreen
import app.solferry.ui.LinkSheet
import app.solferry.ui.LocalToaster
import app.solferry.ui.OnboardingScreen
import app.solferry.ui.PairRequestSheet
import app.solferry.ui.SendSheet
import app.solferry.ui.SettingsScreen
import app.solferry.ui.SolferryTheme
import app.solferry.ui.SourceSheet
import app.solferry.ui.Tab
import app.solferry.ui.TabBar
import app.solferry.ui.TextSheet
import app.solferry.ui.Toaster
import app.solferry.ui.ToasterOverSheet
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.solferry_ffi.PairedDevice
import uniffi.solferry_ffi.setLocale

private enum class Screen { Home, History, Settings }

private const val PRESENCE_EVERY_MS = 3_000L

class MainActivity : ComponentActivity() {
    private var shared by mutableStateOf<List<Uri>>(emptyList())

    override fun onStart() {
        super.onStart()
        Notifier.appVisible = true
    }

    override fun onStop() {
        Notifier.appVisible = false
        (application as SolferryApp).engine.leaveUnlessStaying()
        super.onStop()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        // A language change recreates the activity, so the core's texts follow the app's language from here.
        setLocale(resources.configuration.locales[0].toLanguageTag())
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
                var choosing by rememberSaveable { mutableStateOf(false) }
                var texting by remember { mutableStateOf<PairedDevice?>(null) }
                val picked: (List<Uri>) -> Unit = { uris -> if (uris.isNotEmpty()) shared = (shared + uris).distinct() }
                val files = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments(), picked)
                val media = rememberLauncherForActivityResult(ActivityResultContracts.PickMultipleVisualMedia(), picked)
                var onboarded by remember { mutableStateOf(engine.onboarded) }
                if (!onboarded) {
                    OnboardingScreen(state, engine) { onboarded = true }
                    return@SolferryTheme
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
                val statusTop = WindowInsets.statusBars.getTop(LocalDensity.current)
                CompositionLocalProvider(LocalToaster provides { ToasterOverSheet(state, engine, statusTop) }) { Box(Modifier.fillMaxSize()) {
                    AnimatedContent(
                        screen,
                        Modifier.fillMaxSize(),
                        transitionSpec = { fadeIn(tween(MotionTokens.standard)) togetherWith fadeOut(tween(MotionTokens.quick)) },
                    ) { shown ->
                        when (shown) {
                            Screen.Home -> HomeScreen(
                                state, engine,
                                onPickFiles = { choosing = true },
                                onText = { texting = it },
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
                            onSend = { peers -> engine.sendUris(peers, shared) },
                            onShareLink = { engine.shareLink(shared); shared = emptyList() },
                            onClose = { shared = emptyList() },
                        )
                    }
                    if (choosing) {
                        SourceSheet(
                            onMedia = { media.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageAndVideo)) },
                            onFiles = { files.launch(arrayOf("*/*")) },
                        ) { choosing = false }
                    }
                    texting?.let { device -> TextSheet(device, engine) { texting = null } }
                    state.link?.let { LinkSheet(it, engine) }
                    if (pairing) AddDeviceSheet(state, engine) { pairing = false }
                    val asked = state.pairings.firstOrNull()
                    val code = state.pairingCode
                    if (asked != null) {
                        key(asked.key) { PairRequestSheet(asked, engine) }
                    } else if (code != null) {
                        CompareSheet(code, engine)
                    }
                    TabBar(
                        listOf(
                            Tab(stringResource(R.string.tab_transfer), R.drawable.ic_home, screen == Screen.Home) { screen = Screen.Home },
                            Tab(stringResource(R.string.tab_history), R.drawable.ic_history, screen == Screen.History) { screen = Screen.History },
                            Tab(stringResource(R.string.tab_settings), R.drawable.ic_settings, screen == Screen.Settings) { screen = Screen.Settings },
                        ),
                        Modifier.align(Alignment.BottomCenter),
                    )
                    val sheetOpen = shared.isNotEmpty() || choosing || texting != null || state.link != null || pairing || state.pairings.isNotEmpty() || state.pairingCode != null
                    if (!sheetOpen) Toaster(state, engine, Modifier.align(Alignment.TopCenter))
                } }
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
