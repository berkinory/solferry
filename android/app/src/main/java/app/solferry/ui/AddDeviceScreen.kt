package app.solferry.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import app.solferry.AppState
import app.solferry.Engine
import app.solferry.R
import app.solferry.design.SpaceTokens
import uniffi.solferry_ffi.lanAddresses
import uniffi.solferry_ffi.qrCode

@Composable
fun AddDeviceSheet(
    state: AppState,
    engine: Engine,
    onClose: () -> Unit,
) = Sheet(stringResource(R.string.pair_code_title), { engine.clearPairingErrors(); onClose() }) {
    var code by remember { mutableStateOf("") }
    val join = { if (code.isNotBlank()) engine.join(code.trim()) }
    Card(R.drawable.ic_qr, stringResource(R.string.pair_show_title)) {
        PaddedWell {
            val invite = state.invite
            if (invite == null) {
                Muted(stringResource(R.string.pair_show_hint))
                Button({ engine.bridge.invite() }, Modifier.fillMaxWidth(), label = stringResource(R.string.pair_show_code), icon = R.drawable.ic_qr, kind = ButtonKind.Primary)
            } else {
                Invite(invite) { engine.copy(invite) }
            }
        }
    }
    Card(R.drawable.ic_key, stringResource(R.string.pair_enter_title)) {
        PaddedWell {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
                Field(
                    code, { code = it }, stringResource(R.string.pair_enter_placeholder), Modifier.weight(1f),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Go),
                    keyboardActions = KeyboardActions(onGo = { join() }),
                    trailing = {
                        Button(
                            { engine.pasted()?.let { code = it.trim() } }, icon = R.drawable.ic_clipboard,
                            kind = ButtonKind.Ghost, size = ButtonSize.Sm, description = stringResource(R.string.common_paste),
                        )
                    },
                )
                Button(join, label = stringResource(if (state.joining) R.string.common_pairing else R.string.common_pair), kind = ButtonKind.Primary, enabled = !state.joining)
            }
            state.joinError?.let { Muted(it, color = LocalPalette.current.danger) }
        }
    }
    AddressCard(state, engine)
}

/** For a device on another subnet, or on a network that keeps devices from finding each other. */
@Composable
private fun AddressCard(state: AppState, engine: Engine) = Card(R.drawable.ic_link, stringResource(R.string.pair_address_title)) {
    var address by remember { mutableStateOf("") }
    val own = remember { lanAddresses() }
    val pair = { if (address.isNotBlank()) engine.pairAddress(address.trim()) }
    PaddedWell {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
            Field(
                address, { address = it }, stringResource(R.string.pair_address_placeholder), Modifier.weight(1f),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Go),
                keyboardActions = KeyboardActions(onGo = { pair() }),
            )
            Button(pair, label = stringResource(if (state.pairingAddress) R.string.common_pairing else R.string.common_pair), kind = ButtonKind.Primary, enabled = !state.pairingAddress)
        }
        state.addressError?.let { Muted(it, color = LocalPalette.current.danger) }
        if (own.isNotEmpty()) Muted(stringResource(R.string.pair_own_address, own.joinToString(", ")))
    }
}

@Composable
private fun ColumnScope.Invite(invite: String, onCopy: () -> Unit) {
    var copied by remember(invite) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpaceTokens.sm.dp)) {
        Qr(invite)
        Muted(stringResource(R.string.pair_invite_expiry))
        Button(
            { onCopy(); copied = true }, label = stringResource(if (copied) R.string.common_copied else R.string.pair_copy_code),
            icon = if (copied) R.drawable.ic_check else R.drawable.ic_copy, size = ButtonSize.Sm,
        )
    }
}

/** Draws the code on a white plate so it scans in both themes. */
@Composable
fun Qr(text: String) {
    val matrix = remember(text) { qrCode(text) } ?: return
    val width = matrix.width.toInt()
    Canvas(Modifier.size(232.dp).background(Color.White, RoundedCornerShape(16.dp)).padding(16.dp)) {
        val cell = size.width / width
        matrix.dark.forEachIndexed { index, dark ->
            if (dark) {
                drawRect(Color.Black, Offset((index % width) * cell, (index / width) * cell), Size(cell + 0.5f, cell + 0.5f))
            }
        }
    }
}
