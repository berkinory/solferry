package app.solferry

import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.DocumentsContract.Document
import android.webkit.MimeTypeMap

private const val ACCESS = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION

/** A folder the user picked through the system, which received files go to instead of Downloads/Solferry. */
data class SaveFolder(val tree: Uri, val name: String) {
    private val root get() = rootOf(tree)

    /** Creates an empty file at [path] inside this folder, making any folders on the way, or null if it can't. */
    fun create(resolver: ContentResolver, path: String, fileName: String, folders: MutableMap<String, Uri>): Uri? = runCatching {
        val parent = folder(resolver, path, folders)
        val mime = MimeTypeMap.getSingleton().getMimeTypeFromExtension(fileName.substringAfterLast('.', "").lowercase())
        DocumentsContract.createDocument(resolver, parent, mime ?: "application/octet-stream", fileName)
    }.getOrNull()

    /** Finds or makes [path] one level at a time, remembering each folder for the next file in the same pass. */
    private fun folder(resolver: ContentResolver, path: String, folders: MutableMap<String, Uri>): Uri {
        if (path.isEmpty()) return root
        folders[path]?.let { return it }
        val parent = folder(resolver, path.substringBeforeLast('/', ""), folders)
        val name = path.substringAfterLast('/')
        val made = child(resolver, parent, name) ?: requireNotNull(DocumentsContract.createDocument(resolver, parent, Document.MIME_TYPE_DIR, name))
        folders[path] = made
        return made
    }

    private fun child(resolver: ContentResolver, parent: Uri, name: String): Uri? {
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(tree, DocumentsContract.getDocumentId(parent))
        val columns = arrayOf(Document.COLUMN_DOCUMENT_ID, Document.COLUMN_DISPLAY_NAME, Document.COLUMN_MIME_TYPE)
        resolver.query(children, columns, null, null, null)?.use { cursor ->
            while (cursor.moveToNext()) {
                if (cursor.getString(1) == name && cursor.getString(2) == Document.MIME_TYPE_DIR) {
                    return DocumentsContract.buildDocumentUriUsingTree(tree, cursor.getString(0))
                }
            }
        }
        return null
    }

    companion object {
        /** Keeps access to a folder the picker returned, across restarts. Null if the system won't. */
        fun take(context: Context, tree: Uri): SaveFolder? {
            runCatching { context.contentResolver.takePersistableUriPermission(tree, ACCESS) }.getOrElse { return null }
            return check(context, tree)
        }

        fun release(context: Context, tree: Uri) {
            runCatching { context.contentResolver.releasePersistableUriPermission(tree, ACCESS) }
        }

        /** The folder as it is now, or null once access was taken back or the folder is gone. */
        fun check(context: Context, tree: Uri): SaveFolder? {
            val granted = context.contentResolver.persistedUriPermissions
                .any { it.uri == tree && it.isReadPermission && it.isWritePermission }
            if (!granted) return null
            val name = runCatching {
                context.contentResolver.query(rootOf(tree), arrayOf(Document.COLUMN_DISPLAY_NAME), null, null, null)?.use { cursor ->
                    if (cursor.moveToFirst()) cursor.getString(0) else null
                }
            }.getOrNull()
            return name?.let { SaveFolder(tree, it) }
        }

        private fun rootOf(tree: Uri) = DocumentsContract.buildDocumentUriUsingTree(tree, DocumentsContract.getTreeDocumentId(tree))
    }
}
