package app.solferry

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.webkit.MimeTypeMap
import androidx.core.content.edit
import androidx.core.net.toUri
import org.json.JSONArray
import org.json.JSONObject

/**
 * History knows where the core put a transfer's first file, but [Exporter] moves it on. This remembers where it went,
 * so tapping the transfer opens it. The core reuses a path once its file has moved out, so a record also notes when it
 * was made, and a transfer takes the first one made after it finished.
 */
object SavedFiles {
    private const val RECORDS = "records"
    private const val KEPT = 200
    private val lock = Any()

    enum class Opened { Yes, Missing, NoApp }

    fun record(context: Context, path: String, uri: Uri) = synchronized(lock) {
        val record = JSONObject().put("path", path).put("at", System.currentTimeMillis()).put("uri", uri.toString())
        val kept = listOf(record) + records(context).take(KEPT - 1)
        prefs(context).edit { putString(RECORDS, JSONArray(kept).toString()) }
    }

    fun open(context: Context, path: String, finishedMs: Long): Opened {
        val uri = find(context, path, finishedMs)?.takeIf { exists(context, it) } ?: return Opened.Missing
        val view = Intent(Intent.ACTION_VIEW)
            .setDataAndType(uri, mimeType(context, uri, path))
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
        return try {
            context.startActivity(view)
            Opened.Yes
        } catch (_: ActivityNotFoundException) {
            Opened.NoApp
        }
    }

    private fun find(context: Context, path: String, finishedMs: Long): Uri? = synchronized(lock) {
        records(context).filter { it.optString("path") == path && it.optLong("at") >= finishedMs }
            .minByOrNull { it.optLong("at") }?.optString("uri")?.toUri()
    }

    private fun exists(context: Context, uri: Uri) = runCatching { context.contentResolver.openFileDescriptor(uri, "r")?.close() }.isSuccess

    /** The extension decides when the provider only knows bytes, which MediaStore says for anything it doesn't index. */
    private fun mimeType(context: Context, uri: Uri, path: String): String {
        val extension = path.substringAfterLast('/').substringAfterLast('.', "").lowercase()
        val guessed = MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension)
        return context.contentResolver.getType(uri)?.takeIf { it != "application/octet-stream" } ?: guessed ?: "*/*"
    }

    private fun records(context: Context): List<JSONObject> {
        val stored = runCatching { JSONArray(prefs(context).getString(RECORDS, null) ?: "[]") }.getOrDefault(JSONArray())
        return (0 until stored.length()).mapNotNull { stored.optJSONObject(it) }
    }

    private fun prefs(context: Context) = context.getSharedPreferences("saved_files", Context.MODE_PRIVATE)
}
