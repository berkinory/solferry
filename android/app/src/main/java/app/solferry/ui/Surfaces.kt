package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ripple
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.draw.innerShadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import app.solferry.design.RadiusTokens
import app.solferry.design.SpaceTokens

val CardShape = RoundedCornerShape(RadiusTokens.card.dp)
val WellShape = RoundedCornerShape(RadiusTokens.well.dp)
val ControlShape = RoundedCornerShape(RadiusTokens.control.dp)
val FieldShape = RoundedCornerShape(RadiusTokens.field.dp)
val FloorShape = RoundedCornerShape(RadiusTokens.floor.dp)

/** One line and 3dp of padding, so what it holds sits exactly 4dp inside it. */
@Composable
fun Frame(modifier: Modifier = Modifier, line: Color = LocalPalette.current.line, content: @Composable ColumnScope.() -> Unit) = Column(
    modifier.fillMaxWidth()
        .dropShadow(CardShape, Shadow(radius = 2.dp, color = Color.Black.copy(alpha = 0.06f), offset = DpOffset(0.dp, 1.dp)))
        .background(LocalPalette.current.page, CardShape)
        .border(1.dp, line, CardShape)
        .padding(4.dp),
    content = content,
)

/** Set into the frame by shade and a soft inner shadow, with no line of its own. */
@Composable
fun Well(
    modifier: Modifier = Modifier,
    ground: Color = LocalPalette.current.well,
    padding: Dp = SpaceTokens.sm.dp,
    spacing: Dp = SpaceTokens.xs.dp,
    onClick: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val palette = LocalPalette.current
    val (depth, rim) = if (palette.night) {
        Color.Black.copy(alpha = 0.45f) to Color.White.copy(alpha = 0.03f)
    } else {
        Color.Black.copy(alpha = 0.05f) to Color.Black.copy(alpha = 0.035f)
    }
    val press = remember { MutableInteractionSource() }
    val pressed by press.collectIsPressedAsState()
    val tap = if (onClick != null) Modifier.clip(WellShape).clickable(press, ripple(), onClick = onClick) else Modifier
    Column(
        modifier.fillMaxWidth()
            .background(if (pressed) palette.raised else ground, WellShape)
            .innerShadow(WellShape, Shadow(radius = 2.dp, color = depth, offset = DpOffset(0.dp, 1.dp)))
            .innerShadow(WellShape, Shadow(radius = 0.dp, color = rim, spread = 1.dp))
            .then(tap)
            .padding(padding),
        verticalArrangement = Arrangement.spacedBy(spacing),
        content = content,
    )
}

/** A well with room for text, spaced for reading rather than for rows. */
@Composable
fun PaddedWell(ground: Color = LocalPalette.current.well, content: @Composable ColumnScope.() -> Unit) =
    Well(ground = ground, padding = SpaceTokens.lg.dp, spacing = SpaceTokens.sm.dp, content = content)

/** A frame with an icon and title over the well that holds its rows. */
@Composable
fun Card(
    @DrawableRes icon: Int,
    title: String,
    tone: Color = LocalPalette.current.inkMuted,
    line: Color = LocalPalette.current.line,
    aside: (@Composable RowScope.() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) = Frame(line = line) {
    Row(
        Modifier.fillMaxWidth().height(36.dp).padding(start = SpaceTokens.lg.dp, end = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Glyph(icon, tone, 16)
        Spacer(Modifier.width(SpaceTokens.sm.dp))
        Text(
            title, Modifier.weight(1f), style = MaterialTheme.typography.labelMedium, maxLines = 1,
            color = LocalPalette.current.ink.copy(alpha = 0.8f),
        )
        aside?.invoke(this)
    }
    content()
}

@Composable
fun Count(value: Int) = Caption(value.toString(), Modifier.padding(end = 14.dp))
