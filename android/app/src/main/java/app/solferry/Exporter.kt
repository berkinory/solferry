package app.solferry

import android.content.ContentValues
import android.content.Context
import android.os.Environment
import android.provider.MediaStore
import java.io.File

/**
 * The core writes into app storage, which other apps can't see. Finished files move into
 * Downloads/Solferry, keeping their folders, where the Files app and everything else finds them.
 */
object Exporter {
    private const val PARTIAL_DIR = ".solferry-partial"
    private val lock = Any()

    fun export(context: Context, received: File) = synchronized(lock) {
        received.walkTopDown()
            .onEnter { it.name != PARTIAL_DIR }
            .filter { it.isFile }
            .forEach { file -> move(context, received, file) }
        received.walkBottomUp().filter { it != received && it.isDirectory && it.name != PARTIAL_DIR }
            .forEach { it.delete() }
    }

    private fun move(context: Context, received: File, file: File) {
        val folder = file.parentFile?.relativeTo(received)?.path.orEmpty()
        val relative = listOf(Environment.DIRECTORY_DOWNLOADS, "Solferry", folder).filter { it.isNotEmpty() }.joinToString("/")
        val values = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, file.name)
            put(MediaStore.Downloads.RELATIVE_PATH, relative)
            put(MediaStore.Downloads.IS_PENDING, 1)
        }
        val resolver = context.contentResolver
        val target = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values) ?: return
        val copied = runCatching {
            resolver.openOutputStream(target)?.use { output -> file.inputStream().use { it.copyTo(output, 1 shl 20) } }
        }.isSuccess
        if (copied) {
            resolver.update(target, ContentValues().apply { put(MediaStore.Downloads.IS_PENDING, 0) }, null, null)
            file.delete()
        } else {
            resolver.delete(target, null, null)
        }
    }
}
