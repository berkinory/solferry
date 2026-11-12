package app.solferry.ui

import android.app.LocaleManager
import android.os.Build
import android.os.LocaleList
import androidx.annotation.RequiresApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.selection.selectable
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import app.solferry.R
import app.solferry.design.SpaceTokens

/** Each language by its own name, so anyone can find theirs whatever the app shows now, with its flag. */
private val LANGUAGES = listOf(
    Triple("en", "English", "🇬🇧"),
    Triple("tr", "Türkçe", "🇹🇷"),
    Triple("de", "Deutsch", "🇩🇪"),
    Triple("fr", "Français", "🇫🇷"),
    Triple("it", "Italiano", "🇮🇹"),
    Triple("es", "Español", "🇪🇸"),
    Triple("pt", "Português", "🇧🇷"),
    Triple("ru", "Русский", "🇷🇺"),
    Triple("uk", "Українська", "🇺🇦"),
    Triple("ja", "日本語", "🇯🇵"),
    Triple("ko", "한국어", "🇰🇷"),
    Triple("zh", "简体中文", "🇨🇳"),
    Triple("nl", "Nederlands", "🇳🇱"),
    Triple("pl", "Polski", "🇵🇱"),
    Triple("he", "עברית", "🇮🇱"),
)

/** The app's own language. Android recreates the activity once it changes. */
@RequiresApi(Build.VERSION_CODES.TIRAMISU)
@Composable
fun LanguageCard() {
    val locales = LocalContext.current.getSystemService(LocaleManager::class.java)
    // Until someone picks one, the app shows the first of the system's languages it has.
    val shown = locales.applicationLocales.takeUnless { it.isEmpty } ?: locales.systemLocales
    var chosen by remember { mutableStateOf(supported(shown)) }
    var picking by remember { mutableStateOf(false) }
    val label = LANGUAGES.first { it.first == chosen }.let { "${it.third}  ${it.second}" }
    Card(R.drawable.ic_language, stringResource(R.string.language_title)) {
        Well(padding = SpaceTokens.lg.dp, onClick = { picking = true }) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Muted(label, Modifier.weight(1f))
                Glyph(R.drawable.ic_chevron_down, size = 18)
            }
        }
    }
    if (picking) {
        Sheet(stringResource(R.string.language_title), { picking = false }) { close ->
            Frame {
                Well {
                    LANGUAGES.forEach { (tag, name, flag) ->
                        LanguageRow(name, flag, tag == chosen) {
                            close()
                            chosen = tag
                            locales.applicationLocales = LocaleList.forLanguageTags(tag)
                        }
                    }
                }
            }
        }
    }
}

private fun supported(list: LocaleList): String =
    (0 until list.size()).map { list[it].toLanguageTag().substringBefore('-') }
        .firstOrNull { tag -> LANGUAGES.any { it.first == tag } } ?: "en"

@Composable
private fun LanguageRow(name: String, flag: String, selected: Boolean, onClick: () -> Unit) {
    val palette = LocalPalette.current
    Row(
        Modifier.fillMaxWidth().clip(ControlShape).background(if (selected) palette.raised else Color.Transparent)
            .selectable(selected, role = Role.RadioButton, onClick = onClick)
            .padding(horizontal = SpaceTokens.md.dp, vertical = SpaceTokens.md.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(flag, Modifier.padding(end = SpaceTokens.md.dp))
        Text(name, Modifier.weight(1f), maxLines = 1)
        if (selected) Glyph(R.drawable.ic_check, palette.primary, 18)
    }
}
