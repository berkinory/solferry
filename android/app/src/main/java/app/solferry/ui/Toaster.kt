package app.solferry.ui

import androidx.annotation.DrawableRes
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
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
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
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
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.Phase
import app.solferry.R
import app.solferry.Toast
import app.solferry.Tone
import app.solferry.Transfer
import kotlinx.coroutines.delay
import uniffi.solferry_ffi.TransferRoute

private const val SUCCESS_STAYS_MS = 3000L
private const val ERROR_STAYS_MS = 5000L
private const val EXIT_MS = 220L

private val PillShape = RoundedCornerShape(20.dp)
private val enter: EnterTransition =
    slideInVertically(spring(dampingRatio = 0.82f, stiffness = 340f)) { -it / 3 } + fadeIn() + scaleIn(initialScale = 0.96f)
private val exit: ExitTransition =
    slideOutVertically(tween(EXIT_MS.toInt())) { -it / 4 } + fadeOut(tween(EXIT_MS.toInt())) + scaleOut(targetScale = 0.96f)

private fun routeName(route: TransferRoute) = when (route) {
    TransferRoute.LOCAL -> "local network"
    TransferRoute.OVERLAY -> "overlay network"
    TransferRoute.DIRECT -> "direct"
    TransferRoute.RELAY -> "relay"
}

private fun Transfer.motion(): String {
    val who = "${if (sending) "To" else "From"} $peerName"
    if (phase == Phase.Reconnecting) return "$who  ·  Reconnecting"
    if (bytesPerSecond <= 1) return "$who  ·  ${routeName(route)}"
    return "$who  ·  ${bytes(bytesPerSecond.toLong())}/s  ·  ${routeName(route)}"
}

/** Foreground fill and page-colored text, so the pill flips with the theme by itself. */
@Composable
private fun Shell(modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    val palette = LocalPalette.current
    Box(modifier.fillMaxWidth().shadow(10.dp, PillShape).clip(PillShape).background(palette.ink)) { content() }
}

@Composable
private fun Lines(title: String, detail: String, modifier: Modifier = Modifier) {
    val ground = LocalPalette.current.page
    Column(modifier) {
        Text(title, color = ground, fontSize = 13.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(detail, color = ground.copy(alpha = 0.62f), fontSize = 11.5.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/** A filled circle with a mark that lands with a small overshoot. */
@Composable
private fun Badge(tone: Tone) {
    val palette = LocalPalette.current
    val size = remember { Animatable(0.6f) }
    LaunchedEffect(Unit) { size.animateTo(1f, spring(dampingRatio = 0.42f, stiffness = 420f)) }
    Box(
        Modifier.size(18.dp).scale(size.value).background(if (tone == Tone.Success) palette.successFill else palette.dangerFill, CircleShape),
        contentAlignment = Alignment.Center,
    ) { Glyph(if (tone == Tone.Success) R.drawable.ic_check else R.drawable.ic_cancel, Color.White, 11) }
}

@Composable
private fun Ring(fraction: Float) {
    val ground = LocalPalette.current.page
    val shown by animateFloatAsState(fraction.coerceIn(0f, 1f), tween(240))
    CircularProgressIndicator(
        { shown }, Modifier.size(18.dp), color = ground, trackColor = ground.copy(alpha = 0.2f),
        strokeWidth = 2.5.dp, strokeCap = StrokeCap.Round, gapSize = 0.dp,
    )
}

@Composable
private fun Mark(@DrawableRes icon: Int, description: String, onClick: () -> Unit) {
    val ground = LocalPalette.current.page
    Box(Modifier.size(32.dp).clip(CircleShape).clickable(onClickLabel = description, onClick = onClick), contentAlignment = Alignment.Center) {
        Glyph(icon, ground.copy(alpha = 0.62f), 14)
    }
}

@Composable
private fun RunningRow(transfer: Transfer, engine: Engine) {
    val ground = LocalPalette.current.page
    Row(Modifier.fillMaxWidth().height(46.dp).padding(start = 14.dp, end = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Ring(transfer.fraction)
        Lines(transfer.label, transfer.motion(), Modifier.weight(1f))
        Text("${(transfer.fraction * 100).toInt()}%", color = ground.copy(alpha = 0.62f), fontSize = 12.sp)
        Mark(R.drawable.ic_cancel, "Cancel") { engine.bridge.cancel(transfer.id) }
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
        Column {
            AnimatedContent(
                face,
                transitionSpec = {
                    (slideInVertically(tween(320)) { it / 2 } + fadeIn(tween(240))) togetherWith (slideOutVertically(tween(120)) { -it / 2 } + fadeOut(tween(100)))
                },
                label = "pill",
            ) { shown ->
                when (shown) {
                    is PillFace.One -> running.find { it.id == shown.id }?.let { RunningRow(it, engine) } ?: Box(Modifier.height(46.dp))
                    is PillFace.Many -> Row(
                        Modifier.fillMaxWidth().height(46.dp).clickable { open = !open }.padding(horizontal = 14.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Ring(running.sumOf { it.transferred }.toFloat() / running.sumOf { it.total }.coerceAtLeast(1))
                        val out = running.count { it.sending }
                        val detail = when (out) {
                            0 -> "Receiving ${running.size}"
                            running.size -> "Sending ${running.size}"
                            else -> "Sending $out, receiving ${running.size - out}"
                        }
                        Lines("${running.size} transfers", detail, Modifier.weight(1f))
                        Glyph(R.drawable.ic_more, ground.copy(alpha = 0.62f), 15)
                    }
                    is PillFace.Ended -> transfers.find { it.id == shown.id }?.let { EndedRow(it) } ?: Box(Modifier.height(46.dp))
                }
            }
            AnimatedVisibility(open && running.size > 1, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
                Column { running.forEach { key(it.id) { RunningRow(it, engine) } } }
            }
        }
    }
}

@Composable
private fun EndedRow(transfer: Transfer) {
    val (tone, title, detail) = transfer.outcome
    Row(Modifier.fillMaxWidth().height(46.dp).padding(horizontal = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Badge(tone)
        Lines(title, detail, Modifier.weight(1f))
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
    LaunchedEffect(toast.key) {
        delay(if (toast.tone == Tone.Success) SUCCESS_STAYS_MS else ERROR_STAYS_MS)
        visible.targetState = false
    }
    AnimatedVisibility(visible, enter = expandVertically(spring(dampingRatio = 0.86f, stiffness = 340f)) + enter, exit = exit + shrinkVertically(tween(EXIT_MS.toInt()))) {
        Shell(Modifier.padding(top = 6.dp).shaking(toast.key.takeIf { toast.tone == Tone.Error })) {
            Row(Modifier.fillMaxWidth().height(46.dp).padding(start = 14.dp, end = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Badge(toast.tone)
                Lines(toast.title, toast.detail, Modifier.weight(1f))
                Mark(R.drawable.ic_cancel, "Dismiss") {
                    visible.targetState = false
                }
            }
        }
    }
    LaunchedEffect(visible.isIdle, visible.currentState) {
        if (visible.isIdle && !visible.currentState) engine.dismissToast(toast.key)
    }
}

/** One pill for what is moving over the brand, and short messages underneath it. */
@Composable
fun Toaster(state: AppState, engine: Engine, modifier: Modifier = Modifier) {
    var kept by remember { mutableStateOf(state.transfers) }
    if (state.transfers.isNotEmpty()) kept = state.transfers
    Column(modifier.statusBarsPadding().padding(top = 3.dp).padding(horizontal = 16.dp).widthIn(max = 440.dp).fillMaxWidth()) {
        AnimatedVisibility(state.transfers.isNotEmpty(), enter = enter, exit = exit) { Pill(kept, engine) }
        state.toasts.forEach { key(it.key) { ToastRow(it, engine) } }
    }
}
