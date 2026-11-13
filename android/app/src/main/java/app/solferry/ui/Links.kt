package app.solferry.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink

/** Written-out web and mail links, never a bare domain, so nothing opens that the sender didn't spell out. */
private val written = Regex("""(?<![\p{L}\p{N}])(?:https?://|mailto:)[^\s<>"]+""", RegexOption.IGNORE_CASE)
private const val TRAILING = ".,;:!?'\""

/** Received text with its links underlined. They open only when tapped. */
@Composable
fun linked(text: String): AnnotatedString {
    val color = LocalPalette.current.primary
    return remember(text, color) {
        val styles = TextLinkStyles(SpanStyle(color = color, textDecoration = TextDecoration.Underline))
        buildAnnotatedString {
            var from = 0
            for (match in written.findAll(text)) {
                val url = trim(match.value).takeIf { it.substringAfter(':').trimStart('/').isNotEmpty() } ?: continue
                append(text, from, match.range.first)
                withLink(LinkAnnotation.Url(url, styles)) { append(url) }
                from = match.range.first + url.length
            }
            append(text, from, text.length)
        }
    }
}

/** Leaves sentence punctuation and an unopened closing bracket outside the link. */
private fun trim(found: String): String {
    var url = found
    while (url.isNotEmpty()) {
        val last = url.last()
        url = when {
            last in TRAILING -> url.dropLast(1)
            last == ')' && url.count { it == '(' } < url.count { it == ')' } -> url.dropLast(1)
            else -> return url
        }
    }
    return url
}
