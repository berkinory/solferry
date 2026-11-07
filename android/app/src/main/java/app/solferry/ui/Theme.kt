package app.solferry.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import app.solferry.design.DarkColors
import app.solferry.design.IdentityColors
import app.solferry.design.LightColors
import app.solferry.design.RadiusTokens
import uniffi.solferry_ffi.TransferRoute

/** The design tokens the app reads directly, on top of Material's color scheme. */
data class Palette(
    val well: Color,
    val control: Color,
    val ink: Color,
    val inkMuted: Color,
    val primary: Color,
    val primaryFill: Color,
    val onPrimary: Color,
    val success: Color,
    val warning: Color,
    val danger: Color,
    val dangerWell: Color,
    val routes: Map<TransferRoute, Color>,
    val identity: List<Color>,
)

private val light = with(LightColors) {
    Palette(
        well, control, ink, inkMuted, primary, primaryFill, onPrimary, success, warning, danger, dangerWell,
        mapOf(
            TransferRoute.LOCAL to routeLocal, TransferRoute.OVERLAY to routeOverlay,
            TransferRoute.DIRECT to routeDirect, TransferRoute.RELAY to routeRelay,
        ),
        IdentityColors.light,
    )
}

private val dark = with(DarkColors) {
    Palette(
        well, control, ink, inkMuted, primary, primaryFill, onPrimary, success, warning, danger, dangerWell,
        mapOf(
            TransferRoute.LOCAL to routeLocal, TransferRoute.OVERLAY to routeOverlay,
            TransferRoute.DIRECT to routeDirect, TransferRoute.RELAY to routeRelay,
        ),
        IdentityColors.dark,
    )
}

val LocalPalette = staticCompositionLocalOf { light }

private fun corner(radius: Number) = RoundedCornerShape(radius.toFloat().dp)

@Composable
fun SolferryTheme(content: @Composable () -> Unit) {
    val night = isSystemInDarkTheme()
    val tokens = if (night) DarkColors.run {
        darkColorScheme(
            primary = primaryFill, onPrimary = onPrimary, background = page, onBackground = ink,
            surface = page, onSurface = ink, onSurfaceVariant = inkMuted, surfaceContainerLowest = well,
            surfaceContainerLow = well, surfaceContainer = raised, surfaceContainerHigh = raised,
            surfaceContainerHighest = control, secondaryContainer = control, onSecondaryContainer = ink,
            outline = line, outlineVariant = line, error = danger,
        )
    } else LightColors.run {
        lightColorScheme(
            primary = primaryFill, onPrimary = onPrimary, background = page, onBackground = ink,
            surface = page, onSurface = ink, onSurfaceVariant = inkMuted, surfaceContainerLowest = page,
            surfaceContainerLow = well, surfaceContainer = well, surfaceContainerHigh = raised,
            surfaceContainerHighest = control, secondaryContainer = control, onSecondaryContainer = ink,
            outline = line, outlineVariant = line, error = danger,
        )
    }
    val shapes = Shapes(
        extraSmall = corner(RadiusTokens.floor), small = corner(RadiusTokens.control),
        medium = corner(RadiusTokens.field), large = corner(RadiusTokens.well), extraLarge = corner(RadiusTokens.card),
    )
    CompositionLocalProvider(LocalPalette provides if (night) dark else light) {
        MaterialTheme(colorScheme = tokens, shapes = shapes, content = content)
    }
}
