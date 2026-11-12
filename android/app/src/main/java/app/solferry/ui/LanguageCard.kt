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
import java.util.Locale

/** Each language by its own name, so anyone can find theirs whatever the app shows now. */
private val LANGUAGES = listOf(
    "en" to "English",
    "tr" to "Türkçe",
    "de" to "Deutsch",
    "fr" to "Français",
    "es" to "Español",
    "pt" to "Português",
    "it" to "Italiano",
    "nl" to "Nederlands",
    "pl" to "Polski",
    "ru" to "Русский",
    "uk" to "Українська",
    "ja" to "日本語",
    "ko" to "한국어",
    "zh-Hans" to "简体中文",
    "zh-Hant" to "繁體中文",
    "id" to "Bahasa Indonesia",
    "vi" to "Tiếng Việt",
    "sv" to "Svenska",
    "cs" to "Čeština",
    "ro" to "Română",
)

/** The app's own language. Android recreates the activity once it changes. */
@RequiresApi(Build.VERSION_CODES.TIRAMISU)
@Composable
fun LanguageCard() {
    val locales = LocalContext.current.getSystemService(LocaleManager::class.java)
    var chosen by remember { mutableStateOf(locales.applicationLocales[0]?.toLanguageTag()) }
    var picking by remember { mutableStateOf(false) }
    val system = stringResource(R.string.language_system)
    val options = listOf(null to system) + LANGUAGES
    val label = chosen?.let { tag ->
        // A language set from the system's per-app screen can carry a region this list doesn't name.
        LANGUAGES.firstOrNull { it.first == tag }?.second ?: Locale.forLanguageTag(tag).let { it.getDisplayName(it) }
    } ?: system
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
                    options.forEach { (tag, name) ->
                        LanguageRow(name, tag == chosen) {
                            close()
                            chosen = tag
                            locales.applicationLocales = tag?.let { LocaleList.forLanguageTags(it) } ?: LocaleList.getEmptyLocaleList()
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun LanguageRow(name: String, selected: Boolean, onClick: () -> Unit) {
    val palette = LocalPalette.current
    Row(
        Modifier.fillMaxWidth().clip(ControlShape).background(if (selected) palette.raised else Color.Transparent)
            .selectable(selected, role = Role.RadioButton, onClick = onClick)
            .padding(horizontal = SpaceTokens.md.dp, vertical = SpaceTokens.md.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(name, Modifier.weight(1f), maxLines = 1)
        if (selected) Glyph(R.drawable.ic_check, palette.primary, 18)
    }
}
