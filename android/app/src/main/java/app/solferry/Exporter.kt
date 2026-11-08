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

    /** Returns false if a file couldn't be moved. It stays in app storage for the next attempt. */
    fun export(context: Context, received: File): Boolean = synchronized(lock) { moveAll(context, received) }

    /** Finishes what a killed process left behind: drops its half-written copies and moves the files again. */
    fun recover(context: Context, received: File): Boolean = synchronized(lock) {
        val folder = "${Environment.DIRECTORY_DOWNLOADS}/Solferry%"
        val where = "${MediaStore.Downloads.RELATIVE_PATH} LIKE ? AND ${MediaStore.Downloads.IS_PENDING} = 1"
        context.contentResolver.delete(pendingDownloads(), where, arrayOf(folder))
        moveAll(context, received)
    }

    @Suppress("DEPRECATION")
    private fun pendingDownloads() = MediaStore.setIncludePending(MediaStore.Downloads.EXTERNAL_CONTENT_URI)

    private fun moveAll(context: Context, received: File): Boolean {
        val moved = received.walkTopDown()
            .onEnter { it.name != PARTIAL_DIR }
            .filter { it.isFile }
            .toList()
            .map { file -> move(context, received, file) }
        received.walkBottomUp().onEnter { it.name != PARTIAL_DIR }.filter { it != received && it.isDirectory }
            .forEach { it.delete() }
        return moved.all { it }
    }

    private fun move(context: Context, received: File, file: File): Boolean {
        val folder = file.parentFile?.relativeTo(received)?.path.orEmpty()
        val relative = listOf(Environment.DIRECTORY_DOWNLOADS, "Solferry", folder).filter { it.isNotEmpty() }.joinToString("/")
        val values = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, file.name)
            put(MediaStore.Downloads.RELATIVE_PATH, relative)
            put(MediaStore.Downloads.IS_PENDING, 1)
        }
        val resolver = context.contentResolver
        val target = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values) ?: return false
        val copied = runCatching {
            requireNotNull(resolver.openOutputStream(target)).use { output -> file.inputStream().use { it.copyTo(output, 1 shl 20) } }
        }.isSuccess
        if (copied) {
            resolver.update(target, ContentValues().apply { put(MediaStore.Downloads.IS_PENDING, 0) }, null, null)
            file.delete()
        } else {
            resolver.delete(target, null, null)
        }
        return copied
    }
}
