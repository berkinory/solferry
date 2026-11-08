package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.withFrameMillis
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.solferry.R
import app.solferry.design.RadiusTokens
import app.solferry.design.SpaceTokens
import kotlinx.coroutines.delay
import kotlin.math.PI
import kotlin.math.sin

private const val PULSE_MS = 2800
private const val TICK_MS = 50L
private const val SIZE = 208
private const val CORE = 64
private val orbits = listOf(124f, 200f)

private data class Ghost(@DrawableRes val icon: Int, val x: Int, val y: Int, val phase: Float)

/** The kinds of device that could turn up, resting on the orbit where they would appear. */
private val ghosts = listOf(
    Ghost(R.drawable.ic_apple, -78, -50, 0f),
    Ghost(R.drawable.ic_windows, 80, -36, 0.25f),
    Ghost(R.drawable.ic_android, -64, 68, 0.5f),
    Ghost(R.drawable.ic_linux, 74, 62, 0.75f),
)

/** Runs at 20 fps. These stay on screen as long as the app is open, and a full rate cost a third of a core. */
@Composable
fun rememberCycle(millis: Int): State<Float> = produceState(0f, millis) {
    while (true) {
        delay(TICK_MS)
        value = withFrameMillis { (it % millis).toFloat() / millis }
    }
}

/** A ring leaving the center and fading on its way out. */
private fun DrawScope.pulse(progress: Float, from: Float, tone: Color) {
    val radius = from + (size.minDimension / 2 - from) * progress
    drawCircle(tone.copy(alpha = 0.7f * (1 - progress)), radius, style = Stroke(1.dp.toPx()))
}

@Composable
private fun Radar() {
    val palette = LocalPalette.current
    val cycle by rememberCycle(PULSE_MS)
    val breath by rememberCycle(PULSE_MS + 800)
    Box(Modifier.size(SIZE.dp), contentAlignment = Alignment.Center) {
        Canvas(Modifier.fillMaxSize()) {
            orbits.forEach { drawCircle(palette.line, it.dp.toPx() / 2, style = Stroke(1.dp.toPx())) }
            val from = CORE.dp.toPx() / 2
            pulse(cycle, from, palette.primary)
            pulse((cycle + 0.5f) % 1f, from, palette.primary)
        }
        ghosts.forEach { ghost ->
            val glow = Modifier.graphicsLayer {
                alpha = 0.25f + 0.75f * (sin(2 * PI.toFloat() * (breath + ghost.phase)) * 0.5f + 0.5f)
            }
            Box(Modifier.offset(ghost.x.dp, ghost.y.dp).then(glow)) {
                Tile(ghost.icon, palette.control, palette.inkMuted)
            }
        }
        Box(
            Modifier.size(CORE.dp).background(palette.primaryFill, RoundedCornerShape(RadiusTokens.field.dp))
                .border(1.dp, palette.primaryBorder, RoundedCornerShape(RadiusTokens.field.dp)),
            contentAlignment = Alignment.Center,
        ) { Glyph(R.drawable.ic_nearby, palette.onPrimary, 28) }
    }
}

/** The whole home screen for someone with no devices yet. */
@Composable
fun RadarHero(onUseCode: () -> Unit) = Frame {
    Well(padding = 24.dp, spacing = SpaceTokens.sm.dp) {
        Column(
            Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp),
        ) {
            Radar()
            Text("Looking for nearby devices", Modifier.padding(top = SpaceTokens.sm.dp), style = MaterialTheme.typography.titleMedium)
            Muted("Open Solferry on another device on this network and it shows up here.", textAlign = TextAlign.Center)
            Button(onUseCode, Modifier.padding(top = SpaceTokens.xs.dp), label = "Pair with code", icon = R.drawable.ic_qr, size = ButtonSize.Sm)
        }
    }
}

/** One quiet line for when nothing new is around but the search is still on. */
@Composable
fun Scanning() {
    val palette = LocalPalette.current
    val cycle by rememberCycle(PULSE_MS)
    Row(
        Modifier.fillMaxWidth().padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(Modifier.size(38.dp), contentAlignment = Alignment.Center) {
            Canvas(Modifier.fillMaxSize()) { pulse(cycle, 4.dp.toPx(), palette.primary) }
            Box(Modifier.size(8.dp).background(palette.primary, CircleShape))
        }
        Muted("Looking for new devices on this network")
    }
}
