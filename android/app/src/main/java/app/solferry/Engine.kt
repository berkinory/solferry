package app.solferry

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.net.Uri
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.provider.OpenableColumns
import android.provider.Settings
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import uniffi.solferry_ffi.Listener
import uniffi.solferry_ffi.Reach
import uniffi.solferry_ffi.SharedFile
import uniffi.solferry_ffi.Solferry
import uniffi.solferry_ffi.Update
import java.io.File
import kotlin.concurrent.thread

/** Owns the Rust node for the whole process and turns its updates into [AppState]. */
class Engine(private val context: Context) : Listener {
    private val mutable = MutableStateFlow(AppState())
    val state: StateFlow<AppState> = mutable
    private val main = Handler(Looper.getMainLooper())
    private val received = File(context.getExternalFilesDir(null) ?: context.filesDir, "Received").apply { mkdirs() }
    private val multicast = context.getSystemService(WifiManager::class.java)
        .createMulticastLock("solferry").apply { setReferenceCounted(false); acquire() }
    val bridge = Solferry(File(context.filesDir, "solferry").path, received.path, deviceName(), this)

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
            is Update.Offer -> Notifier.offer(context, update.incoming.name)
            is Update.PairingRequest -> Notifier.pairing(context, update.name)
            is Update.Completed -> {
                val done = state.value.transfers.find { it.id == update.id }
                if (done != null && !done.sending) {
                    thread(name = "solferry-export") { Exporter.export(context, received) }
                    Notifier.received(context, done.label, done.peerName)
                }
            }
            else -> {}
        }
        Keepalive.sync(context, state.value.transfers)
    }

    fun copy(text: String) {
        val clipboard = context.getSystemService(ClipboardManager::class.java)
        clipboard.setPrimaryClip(ClipData.newPlainText("Solferry", text))
    }

    /** Opens each picked document for the core, which reads it through /proc/self/fd and closes it. */
    fun sendUris(peer: String, uris: List<Uri>) {
        val files = uris.mapNotNull { uri ->
            val fd = context.contentResolver.openFileDescriptor(uri, "r")?.detachFd() ?: return@mapNotNull null
            SharedFile(displayName(uri), fd)
        }
        if (files.isNotEmpty()) bridge.sendFiles(peer, files)
    }

    fun shareClipboard(peer: String, name: String) {
        val clipboard = context.getSystemService(ClipboardManager::class.java)
        val text = clipboard.primaryClip?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.coerceToText(context)?.toString()
        if (text.isNullOrEmpty()) {
            mutable.update { it.copy(problems = it.problems + (name to emptyClipboard)) }
        } else {
            bridge.sendText(peer, text)
        }
    }

    fun join(code: String) {
        mutable.update { it.copy(joining = true, pairingProblem = null) }
        bridge.join(code)
    }

    fun setReach(reach: Reach) = bridge.setReach(reach)

    fun cancelRunning() = state.value.transfers.filter { it.running }.forEach { bridge.cancel(it.id) }

    fun dismissProblem(index: Int) = mutable.update { it.copy(problems = it.problems.filterIndexed { i, _ -> i != index }) }

    fun dismissText(key: Long) = mutable.update { state -> state.copy(texts = state.texts.filter { it.key != key }) }

    fun markCopied(key: Long) = mutable.update { state ->
        state.copy(texts = state.texts.map { if (it.key == key) it.copy(copied = true) else it })
    }

    fun clear(id: String) = mutable.update { state -> state.copy(transfers = state.transfers.filter { it.id != id }) }

    fun answerOffer(key: ULong, accept: Boolean) {
        mutable.update { state -> state.copy(offers = state.offers.filter { it.key != key }) }
        bridge.answerOffer(key, accept)
    }

    fun acceptAlways(key: ULong) {
        mutable.update { state -> state.copy(offers = state.offers.filter { it.key != key }) }
        bridge.acceptAlways(key)
    }

    fun blockOffer(key: ULong) {
        mutable.update { state -> state.copy(offers = state.offers.filter { it.key != key }) }
        bridge.blockOffer(key)
    }

    fun answerPairing(key: ULong, accept: Boolean) {
        mutable.update { state -> state.copy(pairings = state.pairings.filter { it.key != key }) }
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
        val emptyClipboard = uniffi.solferry_ffi.Problem("Your clipboard has no text", "Copy some text, then send it again.")
    }
}
