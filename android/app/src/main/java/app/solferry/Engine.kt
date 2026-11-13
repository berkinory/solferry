package app.solferry

import android.app.UiModeManager
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.net.Uri
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import androidx.core.content.edit
import androidx.core.net.toUri
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import uniffi.solferry_ffi.Listener
import uniffi.solferry_ffi.Problem
import uniffi.solferry_ffi.Reach
import uniffi.solferry_ffi.SharedFile
import uniffi.solferry_ffi.Solferry
import uniffi.solferry_ffi.Update
import uniffi.solferry_ffi.setLocale
import java.io.File
import java.util.Locale
import kotlin.concurrent.thread

/** Owns the Rust node for the whole process and turns its updates into [AppState]. */
class Engine(private val context: Context) : Listener {
    private val settings = context.getSharedPreferences("solferry", Context.MODE_PRIVATE)
    private val mutable = MutableStateFlow(
        AppState(
            standby = settings.getBoolean(STANDBY, false),
            saveFolder = savedFolder(),
            convertHeic = settings.getBoolean(CONVERT_HEIC, true),
        ),
    )
    val state: StateFlow<AppState> = mutable
    private val outbox = Outbox(context)
    private val main = Handler(Looper.getMainLooper())
    private val received = File(context.getExternalFilesDir(null) ?: context.filesDir, "Received").apply { mkdirs() }
    private val multicast = context.getSystemService(WifiManager::class.java)
        .createMulticastLock("solferry").apply { setReferenceCounted(false); acquire() }
    init {
        setLocale(Locale.getDefault().toLanguageTag())
    }
    val bridge = Solferry(File(context.filesDir, "solferry").path, received.path, deviceName(), this)

    init {
        thread(name = "solferry-recover") {
            val folder = destination()
            if (!Exporter.recover(context, received, folder)) main.post { saveFailed(context.getString(R.string.error_save_recovered), folder) }
        }
    }

    override fun update(update: Update) {
        val now = System.nanoTime()
        mutable.update { it.apply(update, now, context.resources) }
        main.post { react(update) }
    }

    private fun react(update: Update) {
        when (update) {
            is Update.TextReceived -> {
                if (update.autoCopy) copy(update.text)
                Notifier.text(context, update.name, update.autoCopy)
            }
            is Update.Offer -> Notifier.offer(context, update.incoming.key, update.incoming.name)
            is Update.OfferGone -> Notifier.settleOffer(context, update.key)
            is Update.PairingRequest -> Notifier.pairing(context, update.key, update.name)
            is Update.PairingGone -> Notifier.settlePairing(context, update.key)
            is Update.LinkReceived -> thread(name = "solferry-export") {
                val folder = destination()
                if (!Exporter.export(context, received, folder)) main.post { saveFailed(update.name, folder) }
            }
            is Update.Completed -> {
                val done = state.value.transfers.find { it.id == update.id }
                if (done != null && !done.sending) {
                    thread(name = "solferry-export") {
                        val folder = destination()
                        val saved = Exporter.export(context, received, folder)
                        main.post {
                            if (saved) Notifier.received(context, done.label, done.peerName, folder?.name ?: context.getString(R.string.folder_downloads))
                            else saveFailed(done.peerName, folder)
                        }
                    }
                }
            }
            else -> {}
        }
        Keepalive.sync(context, state.value.transfers, state.value.standby)
    }

    val onboarded get() = settings.getBoolean(ONBOARDED, false)

    fun finishOnboarding() = settings.edit { putBoolean(ONBOARDED, true) }

    /**
     * Android freezes a closed app within seconds unless it receives in the background or is in the
     * middle of a transfer, so paired devices hear that it's going instead of waiting for probes to fail.
     */
    fun leaveUnlessStaying() {
        if (!state.value.standby && state.value.transfers.none { it.running }) bridge.leave()
    }

    /** The UiModeManager night mode chosen in settings. Android applies it but only reports the system's. */
    var appearance: Int
        get() = settings.getInt(APPEARANCE, UiModeManager.MODE_NIGHT_AUTO)
        set(mode) {
            settings.edit { putInt(APPEARANCE, mode) }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                context.getSystemService(UiModeManager::class.java).setApplicationNightMode(mode)
            }
        }

    /** Call from the foreground: Android refuses to start the service from anywhere else. */
    fun resumeStandby() = StandbyService.sync(context, state.value.standby)

    fun setStandby(on: Boolean) {
        settings.edit { putBoolean(STANDBY, on) }
        mutable.update { it.copy(standby = on) }
        StandbyService.sync(context, on)
    }

    private fun saveFailed(name: String, folder: SaveFolder?) = mutable.update {
        it.toast(Tone.Error, context.getString(R.string.error_save_failed, name, folder?.name ?: context.getString(R.string.error_save_downloads)))
    }

    @Synchronized
    fun chooseFolder(tree: Uri) {
        val picked = SaveFolder.take(context, tree) ?: return mutable.update { it.toast(Tone.Error, context.getString(R.string.error_folder_unusable)) }
        state.value.saveFolder?.takeIf { it.tree != tree }?.let { SaveFolder.release(context, it.tree) }
        keepFolder(picked)
    }

    @Synchronized
    fun resetFolder() {
        state.value.saveFolder?.let { SaveFolder.release(context, it.tree) }
        keepFolder(null)
    }

    /** The picked folder if it still works. Once it doesn't, files go to Downloads/Solferry and the user hears about it once. */
    @Synchronized
    private fun destination(): SaveFolder? {
        val picked = state.value.saveFolder ?: return null
        val current = SaveFolder.check(context, picked.tree)
        if (current == null) {
            SaveFolder.release(context, picked.tree)
            mutable.update { it.toast(Tone.Error, context.getString(R.string.error_folder_lost, picked.name, context.getString(R.string.folder_downloads))) }
        }
        if (current != picked) keepFolder(current)
        return current
    }

    private fun keepFolder(folder: SaveFolder?) {
        settings.edit {
            if (folder == null) remove(SAVE_FOLDER).remove(SAVE_FOLDER_NAME)
            else putString(SAVE_FOLDER, folder.tree.toString()).putString(SAVE_FOLDER_NAME, folder.name)
        }
        mutable.update { it.copy(saveFolder = folder) }
    }

    private fun savedFolder(): SaveFolder? {
        val tree = settings.getString(SAVE_FOLDER, null) ?: return null
        return SaveFolder(tree.toUri(), settings.getString(SAVE_FOLDER_NAME, null).orEmpty())
    }

    fun copy(text: String) {
        val clipboard = context.getSystemService(ClipboardManager::class.java)
        clipboard.setPrimaryClip(ClipData.newPlainText(context.getString(R.string.app_name), text))
    }

    /** Off the main thread: HEIC photos may need converting for some of [peers] first. */
    fun sendUris(peers: Collection<String>, uris: List<Uri>) {
        val wantJpeg = if (state.value.convertHeic) peers.filter { Heic.wanted(platform(it)) }.toSet() else emptySet()
        thread(name = "solferry-send") {
            val folder = if (wantJpeg.isEmpty()) null else outbox.stage()
            try {
                val jpegs = folder?.let { outbox.convertHeic(uris, it) }.orEmpty()
                for (peer in peers) {
                    val files = open(uris, if (peer in wantJpeg) jpegs else emptyMap()) ?: return@thread
                    bridge.sendFiles(peer, files)
                }
            } finally {
                folder?.deleteRecursively()
            }
        }
    }

    fun shareLink(uris: List<Uri>) {
        open(uris)?.let { bridge.shareLink(it) }
    }

    private fun open(uris: List<Uri>, jpegs: Map<Uri, File> = emptyMap()): List<SharedFile>? {
        val files = outbox.open(uris, jpegs)
        if (files == null && uris.isNotEmpty()) mutable.update { it.toast(Tone.Error, context.getString(R.string.error_open_file)) }
        return files
    }

    private fun platform(peer: String) = state.value.run {
        (devices.find { it.identity.id == peer }?.identity ?: nearby.find { it.identity.id == peer }?.identity)?.platform
    }

    fun describe(uri: Uri): PickedFile = outbox.describe(uri)

    fun setConvertHeic(on: Boolean) {
        settings.edit { putBoolean(CONVERT_HEIC, on) }
        mutable.update { it.copy(convertHeic = on) }
    }

    fun pasted(): String? {
        val clip = context.getSystemService(ClipboardManager::class.java).primaryClip?.takeIf { it.itemCount > 0 }
        return clip?.getItemAt(0)?.coerceToText(context)?.toString()
    }

    fun clearPairingErrors() = mutable.update { it.copy(joinError = null, addressError = null) }

    fun join(code: String) {
        mutable.update { it.copy(joining = true, joinError = null) }
        bridge.join(code)
    }

    fun pairAddress(address: String) {
        mutable.update { it.copy(pairingAddress = true, addressError = null) }
        bridge.pairAddress(address)
    }

    fun setReach(reach: Reach) = bridge.setReach(reach)

    fun setCheckUpdates(enabled: Boolean) {
        bridge.setCheckUpdates(enabled)
        mutable.update { it.copy(me = it.me?.copy(checkUpdates = enabled)) }
    }

    fun cancelRunning() = state.value.transfers.filter { it.running }.forEach { bridge.cancel(it.id) }

    fun dismissToast(key: Long) = mutable.update { state -> state.copy(toasts = state.toasts.filter { it.key != key }) }

    fun stopComparing() = mutable.update { it.copy(pairingCode = null) }

    fun dismissText(key: Long) = mutable.update { state -> if (state.text?.key == key) state.copy(text = null) else state }

    fun markCopied(key: Long) = mutable.update { state ->
        if (state.text?.key == key) state.copy(text = state.text.copy(copied = true)) else state
    }

    fun clear(id: String) = mutable.update { state -> state.copy(transfers = state.transfers.filter { it.id != id }) }

    private fun settleOffer(key: ULong) {
        mutable.update { state -> state.copy(offers = state.offers.filter { it.key != key }) }
        Notifier.settleOffer(context, key)
    }

    fun answerOffer(key: ULong, accept: Boolean) {
        settleOffer(key)
        bridge.answerOffer(key, accept)
    }

    fun acceptAlways(key: ULong) {
        settleOffer(key)
        bridge.acceptAlways(key)
    }

    fun blockOffer(key: ULong) {
        settleOffer(key)
        bridge.blockOffer(key)
    }

    fun answerPairing(key: ULong, accept: Boolean) {
        mutable.update { state -> state.copy(pairings = state.pairings.filter { it.key != key }) }
        Notifier.settlePairing(context, key)
        bridge.answerPairing(key, accept)
    }

    private fun deviceName(): String =
        Settings.Global.getString(context.contentResolver, Settings.Global.DEVICE_NAME)
            ?: "${Build.MANUFACTURER} ${Build.MODEL}"

    private companion object {
        const val STANDBY = "standby"
        const val CONVERT_HEIC = "convert_heic"
        const val ONBOARDED = "onboarded"
        const val APPEARANCE = "appearance"
        const val SAVE_FOLDER = "save_folder"
        const val SAVE_FOLDER_NAME = "save_folder_name"
    }
}
