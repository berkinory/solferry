package app.solferry

import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.net.toUri
import java.io.File
import java.util.UUID
import uniffi.solferry_ffi.QueuedFile

/**
 * Keeps a picked file openable until a queued send delivers it: a lasting grant on its URI where
 * the provider offers one, as the document and photo pickers do, else a private copy, since a
 * shared item's grant ends with the activity that received it.
 */
object QueueSources {
    private fun folder(context: Context) = File(context.filesDir, "queue")

    fun keep(context: Context, uri: Uri, name: String): String? {
        val granted = runCatching {
            context.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }.isSuccess
        if (granted) return uri.toString()
        return runCatching {
            val home = home(context)
            val copy = File(home, File(name).name.ifEmpty { "file" })
            // Other apps' providers can fail in any way, so any failure means the file can't queue.
            context.contentResolver.openInputStream(uri)!!.use { input -> copy.outputStream().use { input.copyTo(it) } }
            Uri.fromFile(copy).toString()
        }.getOrNull()
    }

    /** Keeps a JPEG copy of a HEIC photo as its private copy, or returns null so the original queues instead. */
    fun keepJpeg(context: Context, uri: Uri, name: String): QueuedFile? {
        val home = home(context)
        val jpeg = Heic.convert(context, uri, name, home) ?: return null.also { home.deleteRecursively() }
        return QueuedFile(jpeg.name, Uri.fromFile(jpeg).toString())
    }

    private fun home(context: Context) = File(folder(context), UUID.randomUUID().toString()).apply { mkdirs() }

    /** A descriptor the bridge takes ownership of, or null once the file is gone. */
    fun open(context: Context, reference: String): Int? = runCatching {
        context.contentResolver.openFileDescriptor(reference.toUri(), "r")?.detachFd()
    }.getOrNull()

    fun release(context: Context, reference: String) {
        val uri = reference.toUri()
        if (uri.scheme == ContentResolver.SCHEME_FILE) {
            val home = uri.path?.let { File(it).parentFile }
            if (home?.parentFile == folder(context)) home.deleteRecursively()
        } else {
            runCatching { context.contentResolver.releasePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION) }
        }
    }
}
