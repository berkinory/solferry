package app.solferry

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import androidx.core.content.edit
import androidx.core.net.toUri
import java.io.File

/**
 * The core writes into app storage, which other apps can't see. Finished files move into
 * Downloads/Solferry or the folder the user picked, keeping their folders, where the Files app
 * and everything else finds them.
 */
object Exporter {
    private const val PARTIAL_DIR = ".solferry-partial"
    /** A picked folder has no pending flag, so the copy being written is written down here first. */
    private const val UNFINISHED = "unfinished_copy"
    private val lock = Any()

    /** Returns false if a file couldn't be moved. It stays in app storage for the next attempt. */
    fun export(context: Context, received: File, folder: SaveFolder?): Boolean = synchronized(lock) { moveAll(context, received, folder) }

    /** Finishes what a killed process left behind: drops its half-written copies and moves the files again. */
    fun recover(context: Context, received: File, folder: SaveFolder?): Boolean = synchronized(lock) {
        val downloads = "${Environment.DIRECTORY_DOWNLOADS}/Solferry%"
        val where = "${MediaStore.Downloads.RELATIVE_PATH} LIKE ? AND ${MediaStore.Downloads.IS_PENDING} = 1"
        context.contentResolver.delete(pendingDownloads(), where, arrayOf(downloads))
        prefs(context).getString(UNFINISHED, null)?.let { copy ->
            runCatching { DocumentsContract.deleteDocument(context.contentResolver, copy.toUri()) }
            prefs(context).edit(commit = true) { remove(UNFINISHED) }
        }
        moveAll(context, received, folder)
    }

    @Suppress("DEPRECATION")
    private fun pendingDownloads() = MediaStore.setIncludePending(MediaStore.Downloads.EXTERNAL_CONTENT_URI)

    private fun prefs(context: Context) = context.getSharedPreferences("solferry", Context.MODE_PRIVATE)

    private fun moveAll(context: Context, received: File, folder: SaveFolder?): Boolean {
        val folders = mutableMapOf<String, Uri>()
        val moved = received.walkTopDown()
            .onEnter { it.name != PARTIAL_DIR }
            .filter { it.isFile }
            .toList()
            .map { file ->
                val path = file.parentFile?.relativeTo(received)?.path.orEmpty()
                if (folder == null) moveToDownloads(context, path, file) else moveInto(context, folder, path, file, folders)
            }
        received.walkBottomUp().onEnter { it.name != PARTIAL_DIR }.filter { it != received && it.isDirectory }
            .forEach { it.delete() }
        return moved.all { it }
    }

    private fun moveToDownloads(context: Context, path: String, file: File): Boolean {
        val relative = listOf(Environment.DIRECTORY_DOWNLOADS, "Solferry", path).filter { it.isNotEmpty() }.joinToString("/")
        val values = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, file.name)
            put(MediaStore.Downloads.RELATIVE_PATH, relative)
            put(MediaStore.Downloads.IS_PENDING, 1)
        }
        val resolver = context.contentResolver
        val target = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values) ?: return false
        val copied = copy(context, file, target)
        if (copied) {
            resolver.update(target, ContentValues().apply { put(MediaStore.Downloads.IS_PENDING, 0) }, null, null)
            file.delete()
        } else {
            resolver.delete(target, null, null)
        }
        return copied
    }

    private fun moveInto(context: Context, folder: SaveFolder, path: String, file: File, folders: MutableMap<String, Uri>): Boolean {
        val resolver = context.contentResolver
        val target = folder.create(resolver, path, file.name, folders) ?: return false
        prefs(context).edit(commit = true) { putString(UNFINISHED, target.toString()) }
        val copied = copy(context, file, target)
        if (copied) file.delete() else runCatching { DocumentsContract.deleteDocument(resolver, target) }
        prefs(context).edit(commit = true) { remove(UNFINISHED) }
        return copied
    }

    private fun copy(context: Context, file: File, target: Uri) = runCatching {
        requireNotNull(context.contentResolver.openOutputStream(target)).use { output -> file.inputStream().use { it.copyTo(output, 1 shl 20) } }
    }.isSuccess
}
