package app.solferry

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.net.Uri
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import android.provider.Settings
import androidx.core.content.edit
import java.io.FileNotFoundException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import uniffi.solferry_ffi.Listener
import uniffi.solferry_ffi.Problem
import uniffi.solferry_ffi.Reach
import uniffi.solferry_ffi.SharedFile
import uniffi.solferry_ffi.Solferry
import uniffi.solferry_ffi.Update
import java.io.File
import kotlin.concurrent.thread

/** Owns the Rust node for the whole process and turns its updates into [AppState]. */
class Engine(private val context: Context) : Listener {
    private val settings = context.getSharedPreferences("solferry", Context.MODE_PRIVATE)
    private val mutable = MutableStateFlow(AppState(standby = settings.getBoolean(STANDBY, false)))
    val state: StateFlow<AppState> = mutable
    private val main = Handler(Looper.getMainLooper())
    private val received = File(context.getExternalFilesDir(null) ?: context.filesDir, "Received").apply { mkdirs() }
    private val multicast = context.getSystemService(WifiManager::class.java)
        .createMulticastLock("solferry").apply { setReferenceCounted(false); acquire() }
    val bridge = Solferry(File(context.filesDir, "solferry").path, received.path, deviceName(), this)

    init {
        thread(name = "solferry-recover") {
            if (!Exporter.recover(context, received)) main.post { saveFailed("Downloads") }
        }
    }

    override fun update(update: Update) {
        val now = System.nanoTime()
        mutable.update { it.apply(update, now) }
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
                if (!Exporter.export(context, received)) main.post { saveFailed("a browser") }
            }
            is Update.Completed -> {
                val done = state.value.transfers.find { it.id == update.id }
                if (done != null && !done.sending) {
                    thread(name = "solferry-export") {
                        val saved = Exporter.export(context, received)
                        main.post { if (saved) Notifier.received(context, done.label, done.peerName) else saveFailed(done.peerName) }
                    }
                }
            }
            else -> {}
        }
        Keepalive.sync(context, state.value.transfers, state.value.standby)
    }

    /** Call from the foreground: Android refuses to start the service from anywhere else. */
    fun resumeStandby() = StandbyService.sync(context, state.value.standby)

    fun setStandby(on: Boolean) {
        settings.edit { putBoolean(STANDBY, on) }
        mutable.update { it.copy(standby = on) }
        StandbyService.sync(context, on)
    }

    private fun saveFailed(name: String) = mutable.update {
        it.toast(Tone.Error, "$name: Couldn't save to Downloads")
    }

    fun copy(text: String) {
        val clipboard = context.getSystemService(ClipboardManager::class.java)
        clipboard.setPrimaryClip(ClipData.newPlainText("Solferry", text))
    }

    fun sendUris(peer: String, uris: List<Uri>) {
        open(uris)?.let { bridge.sendFiles(peer, it) }
    }

    fun shareLink(uris: List<Uri>) {
        open(uris)?.let { bridge.shareLink(it) }
    }

    /** Opens each picked document for the core, which reads it through /proc/self/fd and closes it. */
    private fun open(uris: List<Uri>): List<SharedFile>? {
        val opened = mutableListOf<ParcelFileDescriptor>()
        val files = try {
            uris.map { uri ->
                val file = context.contentResolver.openFileDescriptor(uri, "r") ?: throw FileNotFoundException(uri.toString())
                opened += file
                displayName(uri) to file
            }
        } catch (_: Exception) {
            // These come from other apps, whose providers can refuse or fail in any way.
            opened.forEach { it.close() }
            mutable.update { it.toast(Tone.Error, "Couldn't open that file, try sharing it again") }
            return null
        }
        return files.map { (name, file) -> SharedFile(name, file.detachFd()) }.takeIf { it.isNotEmpty() }
    }

    /** What a picked document is called and how big it is, for the send screen. */
    fun describe(uri: Uri): PickedFile {
        var size: Long? = null
        context.contentResolver.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst() && !cursor.isNull(0)) size = cursor.getLong(0)
        }
        return PickedFile(uri, displayName(uri), size)
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

    private fun displayName(uri: Uri): String {
        context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) cursor.getString(0)?.let { return it }
        }
        return uri.lastPathSegment?.substringAfterLast('/') ?: "file"
    }

    private fun deviceName(): String =
        Settings.Global.getString(context.contentResolver, Settings.Global.DEVICE_NAME)
            ?: "${Build.MANUFACTURER} ${Build.MODEL}"

    private companion object {
        const val STANDBY = "standby"
    }
}
