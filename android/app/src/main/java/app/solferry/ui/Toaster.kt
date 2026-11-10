package app.solferry.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntRect
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupPositionProvider
import androidx.compose.ui.window.PopupProperties
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.Phase
import app.solferry.R
import app.solferry.Toast
import app.solferry.Tone
import app.solferry.Transfer
import uniffi.solferry_ffi.Problem
import kotlinx.coroutines.delay

private const val SUCCESS_STAYS_MS = 3000L
private const val ERROR_STAYS_MS = 5000L
private const val EXIT_MS = 220L

private val PillShape = RoundedCornerShape(20.dp)
private val RECONNECTING = Color(0xFFF59E0B)
private val enter: EnterTransition =
    slideInVertically(spring(dampingRatio = 0.82f, stiffness = 340f)) { -it / 3 } + fadeIn() + scaleIn(initialScale = 0.96f)
private val exit: ExitTransition =
    slideOutVertically(tween(EXIT_MS.toInt())) { -it / 4 } + fadeOut(tween(EXIT_MS.toInt())) + scaleOut(targetScale = 0.96f)

/** Foreground fill and page-colored text, sized by what it says, so it flips with the theme. */
@Composable
private fun Shell(modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    val palette = LocalPalette.current
    Box(
        modifier.widthIn(max = 380.dp).shadow(14.dp, PillShape, spotColor = Color.Black.copy(alpha = 0.3f)).clip(PillShape)
            .background(palette.ink).animateContentSize(spring(dampingRatio = 0.86f, stiffness = 340f)),
    ) { content() }
}

/** One line that comes up from below whenever it changes. */
@Composable
private fun RowScope.Label(text: String, wide: Boolean = false) {
    val ground = LocalPalette.current.page
    AnimatedContent(
        text, Modifier.weight(1f, fill = wide),
        transitionSpec = {
            (slideInVertically(tween(420)) { it } + fadeIn(tween(320))) togetherWith (slideOutVertically(tween(160)) { -it } + fadeOut(tween(120)))
        },
        label = "toast label",
    ) { shown ->
        Text(shown, color = ground, fontSize = 14.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.MiddleEllipsis)
    }
}

@Composable
private fun PillRow(onClick: (() -> Unit)? = null, closes: Boolean, wide: Boolean = false, content: @Composable RowScope.() -> Unit) {
    val press = if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier
    Row(
        press.then(if (wide) Modifier.fillMaxWidth() else Modifier).height(44.dp).padding(start = 16.dp, end = if (closes) 8.dp else 20.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        content = content,
    )
}

/** A filled circle with a mark that lands with a small overshoot. */
@Composable
private fun Badge(tone: Tone) {
    val palette = LocalPalette.current
    val size = remember { Animatable(0.6f) }
    LaunchedEffect(Unit) { size.animateTo(1f, spring(dampingRatio = 0.42f, stiffness = 420f)) }
    Box(
        Modifier.size(16.dp).scale(size.value).background(if (tone == Tone.Success) palette.successFill else palette.dangerFill, CircleShape),
        contentAlignment = Alignment.Center,
    ) { Glyph(if (tone == Tone.Success) R.drawable.ic_check else R.drawable.ic_cancel, Color.White, 10) }
}

@Composable
private fun Ring(fraction: Float, tone: Color) {
    val shown by animateFloatAsState(fraction.coerceIn(0f, 1f), tween(240))
    val color by animateColorAsState(tone)
    CircularProgressIndicator(
        { shown }, Modifier.size(16.dp), color = color, trackColor = color.copy(alpha = 0.25f),
        strokeWidth = 2.dp, strokeCap = StrokeCap.Round, gapSize = 0.dp,
    )
}

@Composable
private fun Close(description: String, onClick: () -> Unit) {
    val ground = LocalPalette.current.page
    Box(Modifier.size(28.dp).clip(RoundedCornerShape(6.dp)).clickable(onClickLabel = description, onClick = onClick), contentAlignment = Alignment.Center) {
        Glyph(R.drawable.ic_cancel, ground.copy(alpha = 0.76f), 13)
    }
}

@Composable
private fun Percent(fraction: Float) =
    Text("${(fraction * 100).toInt()}%", color = LocalPalette.current.page.copy(alpha = 0.76f), fontSize = 13.sp)

@Composable
private fun RunningRow(transfer: Transfer, engine: Engine, wide: Boolean = false) {
    val waiting = transfer.phase == Phase.Reconnecting
    val line = when {
        transfer.phase == Phase.Waiting -> "Waiting for ${transfer.peerName}"
        waiting -> "Reconnecting to ${transfer.peerName}"
        else -> "${if (transfer.sending) "Sending" else "Receiving"} ${transfer.label}"
    }
    PillRow(closes = true, wide = wide) {
        Ring(transfer.fraction, if (waiting) RECONNECTING else LocalPalette.current.page)
        Label(line, wide)
        if (transfer.phase != Phase.Waiting) Percent(transfer.fraction)
        Close("Cancel") { engine.bridge.cancel(transfer.id) }
    }
}

private sealed interface PillFace {
    data class One(val id: String) : PillFace
    data class Many(val count: Int) : PillFace
    data class Ended(val id: String) : PillFace
}

@Composable
private fun Pill(transfers: List<Transfer>, engine: Engine) {
    val ground = LocalPalette.current.page
    val running = transfers.filter { it.running }
    var open by remember { mutableStateOf(false) }
    val face = when {
        running.size > 1 -> PillFace.Many(running.size)
        running.size == 1 -> PillFace.One(running[0].id)
        else -> PillFace.Ended(transfers.first().id)
    }
    if (face is PillFace.Ended) {
        val ended = transfers.first()
        LaunchedEffect(face) {
            delay(if (ended.problem == null) SUCCESS_STAYS_MS else ERROR_STAYS_MS)
            engine.clear(ended.id)
        }
    }
    Shell(Modifier.shaking(transfers.firstOrNull()?.takeIf { it.phase == Phase.Failed }?.id)) {
        val spread = open && running.size > 1
        Column(if (spread) Modifier.width(320.dp) else Modifier.width(IntrinsicSize.Max)) {
            when (face) {
                is PillFace.One -> running.firstOrNull()?.let { RunningRow(it, engine) }
                is PillFace.Many -> PillRow({ open = !open }, closes = false, wide = spread) {
                    val fraction = running.sumOf { it.transferred }.toFloat() / running.sumOf { it.total }.coerceAtLeast(1)
                    Ring(fraction, ground)
                    Label("${running.size} transfers", spread)
                    Percent(fraction)
                    Glyph(if (open) R.drawable.ic_chevron_up else R.drawable.ic_chevron_down, ground.copy(alpha = 0.76f), 15)
                }
                is PillFace.Ended -> transfers.firstOrNull()?.let {
                    val (tone, title) = it.outcome
                    PillRow(closes = false) {
                        Badge(tone)
                        Label(title)
                    }
                }
            }
            AnimatedVisibility(open && running.size > 1, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
                Column { running.forEach { key(it.id) { RunningRow(it, engine, wide = true) } } }
            }
        }
    }
}

@Composable
private fun Modifier.shaking(id: Any?): Modifier {
    val shift = remember { Animatable(0f) }
    LaunchedEffect(id) {
        if (id != null) {
            shift.animateTo(0f, keyframes { durationMillis = 280; -3f at 70; 3f at 140; -3f at 210 })
        }
    }
    return graphicsLayer { translationX = shift.value * density }
}

@Composable
private fun ToastRow(toast: Toast, engine: Engine) {
    val visible = remember { MutableTransitionState(false).apply { targetState = true } }
    val failed = toast.tone == Tone.Error
    LaunchedEffect(toast.key) {
        delay(if (failed) ERROR_STAYS_MS else SUCCESS_STAYS_MS)
        visible.targetState = false
    }
    LaunchedEffect(visible.isIdle, visible.currentState) {
        if (visible.isIdle && !visible.currentState) engine.dismissToast(toast.key)
    }
    AnimatedVisibility(visible, enter = expandVertically(spring(dampingRatio = 0.86f, stiffness = 340f)) + enter, exit = exit + shrinkVertically(tween(EXIT_MS.toInt()))) {
        Shell(Modifier.padding(top = 6.dp).shaking(toast.key.takeIf { failed })) {
            PillRow(closes = failed) {
                Badge(toast.tone)
                Label(toast.title)
                if (failed) Close("Dismiss") { visible.targetState = false }
            }
        }
    }
}

/** Stays until the app can network again, and opens to say why. */
@Composable
private fun FatalRow(problem: Problem, reason: String) {
    val ground = LocalPalette.current.page
    var opened by remember { mutableStateOf(false) }
    Shell {
        Column(if (opened) Modifier.width(320.dp) else Modifier.width(IntrinsicSize.Max)) {
            PillRow({ opened = !opened }, closes = false, wide = opened) {
                Badge(Tone.Error)
                Label(problem.title, opened)
                Glyph(if (opened) R.drawable.ic_chevron_up else R.drawable.ic_chevron_down, ground.copy(alpha = 0.76f), 15)
            }
            AnimatedVisibility(opened, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
                Column(Modifier.padding(start = 18.dp, end = 18.dp, bottom = 14.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(problem.next, color = ground.copy(alpha = 0.76f), fontSize = 13.sp)
                    Text(reason, color = ground.copy(alpha = 0.76f), fontSize = 13.sp)
                }
            }
        }
    }
}

/** A sheet is its own window, so over one the toasts float in a popup at the top of the screen. */
@Composable
fun ToasterOverSheet(state: AppState, engine: Engine, top: Int) {
    val position = remember(top) {
        object : PopupPositionProvider {
            override fun calculatePosition(anchorBounds: IntRect, windowSize: IntSize, layoutDirection: LayoutDirection, popupContentSize: IntSize) =
                IntOffset((windowSize.width - popupContentSize.width) / 2, top)
        }
    }
    Popup(popupPositionProvider = position, properties = PopupProperties(focusable = false, clippingEnabled = false)) {
        Toaster(state, engine, Modifier.padding(bottom = 24.dp), inSheet = true)
    }
}

/** One pill for what is moving over the brand, and short messages underneath it. */
@Composable
fun Toaster(state: AppState, engine: Engine, modifier: Modifier = Modifier, inSheet: Boolean = false) {
    var kept by remember { mutableStateOf(state.transfers) }
    if (state.transfers.isNotEmpty()) kept = state.transfers
    val place = if (inSheet) modifier else modifier.statusBarsPadding()
    Column(place.padding(top = 4.dp).padding(horizontal = 16.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        state.startProblem?.let { FatalRow(it, state.startReason) }
        AnimatedVisibility(state.transfers.isNotEmpty(), enter = enter, exit = exit) { Pill(kept, engine) }
        state.toasts.forEach { key(it.key) { ToastRow(it, engine) } }
    }
}
