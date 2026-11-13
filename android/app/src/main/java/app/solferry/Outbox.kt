package app.solferry

import android.content.Context
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import java.io.File
import java.io.FileNotFoundException
import java.util.UUID
import uniffi.solferry_ffi.SharedFile

/** Turns picked documents into open files for the core, with HEIC photos swapped for JPEG copies where asked. */
class Outbox(private val context: Context) {
    private val staging = File(context.cacheDir, "outbox")

    /** What a picked document is called and how big it is, for the send screen. */
    fun describe(uri: Uri): PickedFile {
        var size: Long? = null
        context.contentResolver.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst() && !cursor.isNull(0)) size = cursor.getLong(0)
        }
        return PickedFile(uri, displayName(uri), size)
    }

    /** A fresh folder for one send's JPEG copies. Delete it once their files are open: the descriptors keep the bytes. */
    fun stage(): File = File(staging, UUID.randomUUID().toString()).apply { mkdirs() }

    /** JPEG copies of the HEIC photos among [uris], one at a time so only one decoded photo is ever in memory. */
    fun convertHeic(uris: List<Uri>, folder: File): Map<Uri, File> = uris.mapNotNull { uri ->
        val name = displayName(uri)
        if (!Heic.matches(name, context.contentResolver.getType(uri))) return@mapNotNull null
        Heic.convert(context, uri, name, folder)?.let { uri to it }
    }.toMap()

    /** Opens each document, or its JPEG copy, for the core, which reads it through /proc/self/fd and closes it. */
    fun open(uris: List<Uri>, jpegs: Map<Uri, File> = emptyMap()): List<SharedFile>? {
        val opened = mutableListOf<ParcelFileDescriptor>()
        val files = try {
            uris.map { uri ->
                val jpeg = jpegs[uri]
                val file = if (jpeg != null) ParcelFileDescriptor.open(jpeg, ParcelFileDescriptor.MODE_READ_ONLY)
                else context.contentResolver.openFileDescriptor(uri, "r") ?: throw FileNotFoundException(uri.toString())
                opened += file
                (jpeg?.name ?: displayName(uri)) to file
            }
        } catch (_: Exception) {
            // These come from other apps, whose providers can refuse or fail in any way.
            opened.forEach { it.close() }
            return null
        }
        return files.map { (name, file) -> SharedFile(name, file.detachFd()) }.takeIf { it.isNotEmpty() }
    }

    fun displayName(uri: Uri): String {
        context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) cursor.getString(0)?.let { return it }
        }
        return uri.lastPathSegment?.substringAfterLast('/') ?: "file"
    }
}
