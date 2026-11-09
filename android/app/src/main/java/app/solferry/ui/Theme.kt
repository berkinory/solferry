package app.solferry.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import app.solferry.R
import app.solferry.design.DarkColors
import app.solferry.design.IdentityColors
import app.solferry.design.LightColors
import app.solferry.design.TypeTokens
import uniffi.solferry_ffi.TransferRoute

/** The design tokens the app reads directly. */
data class Palette(
    val night: Boolean,
    val page: Color,
    val well: Color,
    val raised: Color,
    val control: Color,
    val line: Color,
    val ink: Color,
    val inkMuted: Color,
    val primary: Color,
    val brand: Color,
    val primaryFill: Color,
    val primaryHover: Color,
    val primaryBorder: Color,
    val primaryDeep: Color,
    val onPrimary: Color,
    val success: Color,
    val successFill: Color,
    val platformApple: Color,
    val platformAndroid: Color,
    val platformWindows: Color,
    val platformLinux: Color,
    val warning: Color,
    val danger: Color,
    val dangerFill: Color,
    val warningFill: Color,
    val dangerWell: Color,
    val routes: Map<TransferRoute, Color>,
    val identity: List<Color>,
)

private val light = with(LightColors) {
    Palette(
        false, page, well, raised, control, line, ink, inkMuted, primary, brand, primaryFill, primaryHover, primaryBorder,
        primaryDeep, onPrimary, success, successFill, platformApple, platformAndroid, platformWindows, platformLinux, warning, danger, dangerFill, warningFill, dangerWell,
        mapOf(
            TransferRoute.LOCAL to routeLocal, TransferRoute.OVERLAY to routeOverlay,
            TransferRoute.DIRECT to routeDirect, TransferRoute.RELAY to routeRelay,
        ),
        IdentityColors.light,
    )
}

private val dark = with(DarkColors) {
    Palette(
        true, page, well, raised, control, line, ink, inkMuted, primary, brand, primaryFill, primaryHover, primaryBorder,
        primaryDeep, onPrimary, success, successFill, platformApple, platformAndroid, platformWindows, platformLinux, warning, danger, dangerFill, warningFill, dangerWell,
        mapOf(
            TransferRoute.LOCAL to routeLocal, TransferRoute.OVERLAY to routeOverlay,
            TransferRoute.DIRECT to routeDirect, TransferRoute.RELAY to routeRelay,
        ),
        IdentityColors.dark,
    )
}

val LocalPalette = staticCompositionLocalOf { light }

private val sans = FontFamily(
    Font(R.font.inter_tight_regular, FontWeight.Normal),
    Font(R.font.inter_tight_medium, FontWeight.Medium),
)

/** A phone reads one step larger than the desktop's dense scale. */
private const val PHONE_STEP = 1

private fun style(size: Int, weight: FontWeight = FontWeight.Normal) =
    TextStyle(fontFamily = sans, fontSize = size.sp, lineHeight = (size * 1.35f).sp, fontWeight = weight, letterSpacing = 0.sp)

private val type = Typography(
    titleLarge = style(TypeTokens.title, FontWeight.Medium),
    titleMedium = style(TypeTokens.body + PHONE_STEP, FontWeight.Medium),
    bodyLarge = style(TypeTokens.body + PHONE_STEP),
    bodyMedium = style(TypeTokens.label + PHONE_STEP),
    bodySmall = style(TypeTokens.caption),
    labelLarge = style(TypeTokens.body, FontWeight.Medium),
    labelMedium = style(TypeTokens.label, FontWeight.Medium),
    labelSmall = style(11, FontWeight.Medium),
)

@Composable
fun SolferryTheme(content: @Composable () -> Unit) {
    val night = isSystemInDarkTheme()
    val palette = if (night) dark else light
    val scheme = with(palette) {
        (if (night) darkColorScheme() else lightColorScheme()).copy(
            primary = primaryFill, onPrimary = onPrimary, background = page, onBackground = ink,
            surface = page, onSurface = ink, onSurfaceVariant = inkMuted, outline = line, error = danger,
            surfaceContainerLowest = well, surfaceContainerLow = well, surfaceContainer = raised, surfaceContainerHigh = raised, surfaceContainerHighest = control,
            outlineVariant = line, primaryContainer = primaryFill, onPrimaryContainer = onPrimary, secondaryContainer = raised, onSecondaryContainer = ink,
        )
    }
    CompositionLocalProvider(LocalPalette provides palette) {
        MaterialTheme(colorScheme = scheme, typography = type) {
            Surface(color = palette.page, contentColor = palette.ink, content = content)
        }
    }
}
