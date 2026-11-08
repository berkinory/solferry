package app.solferry

import uniffi.solferry_ffi.Identity
import uniffi.solferry_ffi.Incoming
import uniffi.solferry_ffi.Nearby
import uniffi.solferry_ffi.PairedDevice
import uniffi.solferry_ffi.Problem
import uniffi.solferry_ffi.TransferRoute
import uniffi.solferry_ffi.Update

private const val SPEED_SMOOTHING = 0.2
private const val MAX_TEXTS = 3

enum class Phase { Active, Reconnecting, Done, Failed }

data class Transfer(
    val id: String,
    val peerName: String,
    val label: String,
    val sending: Boolean,
    val route: TransferRoute,
    val transferred: Long = 0,
    val total: Long,
    val bytesPerSecond: Double = 0.0,
    val phase: Phase = Phase.Active,
    val problem: Problem? = null,
    val sampledAt: Long = 0,
    val sampledBytes: Long = 0,
) {
    val running get() = phase == Phase.Active || phase == Phase.Reconnecting
    val fraction get() = if (total == 0L) 1f else transferred.toFloat() / total
}

data class PairingPrompt(val key: ULong, val peer: Identity, val name: String, val code: String)

data class ReceivedText(val key: Long, val name: String, val text: String, val copied: Boolean)

data class Me(val id: String, val fingerprint: String, val name: String, val visibility: String)

data class AppState(
    val me: Me? = null,
    val startProblem: Problem? = null,
    val devices: List<PairedDevice> = emptyList(),
    val nearby: List<Nearby> = emptyList(),
    val reachable: Set<String>? = null,
    val transfers: List<Transfer> = emptyList(),
    val offers: List<Incoming> = emptyList(),
    val pairings: List<PairingPrompt> = emptyList(),
    val pairingCode: String? = null,
    val invite: String? = null,
    val pairingProblem: Problem? = null,
    val joining: Boolean = false,
    val texts: List<ReceivedText> = emptyList(),
    val problems: List<Pair<String, Problem>> = emptyList(),
    val pairedCount: Int = 0,
) {
    val strangers get() = nearby.filter { !it.known }

    fun apply(update: Update, now: Long): AppState = when (update) {
        is Update.Ready -> copy(me = Me(update.id, update.fingerprint, update.name, update.visibility), startProblem = null)
        is Update.StartFailed -> copy(startProblem = update.problem)
        is Update.Devices -> copy(devices = update.devices)
        is Update.Nearby -> copy(nearby = update.devices)
        is Update.Reachable -> copy(reachable = update.ids.toSet())
        is Update.Started -> copy(
            transfers = listOf(
                Transfer(update.id, update.peerName, update.label, update.sending, update.route, total = update.bytes.toLong()),
            ) + transfers.filter { it.id != update.id },
        )
        is Update.Progress -> transfer(update.id) { it.sample(update.transferred.toLong(), update.total.toLong(), now) }
        is Update.RouteChanged -> transfer(update.id) { it.copy(route = update.route) }
        is Update.Reconnecting -> transfer(update.id) { it.copy(phase = Phase.Reconnecting) }
        is Update.Completed -> transfer(update.id) { it.copy(phase = Phase.Done, transferred = it.total) }
        is Update.Failed -> transfer(update.id) { it.copy(phase = Phase.Failed, problem = update.problem) }
        is Update.Offer -> copy(offers = offers + update.incoming)
        is Update.OfferGone -> copy(offers = offers.filter { it.key != update.key })
        is Update.PairingRequest -> copy(pairings = pairings + PairingPrompt(update.key, update.peer, update.name, update.code))
        is Update.PairingGone -> copy(pairings = pairings.filter { it.key != update.key })
        is Update.PairingCode -> copy(pairingCode = update.code)
        is Update.Invite -> copy(invite = update.code)
        is Update.Paired -> copy(
            pairingCode = null, invite = null, pairingProblem = null, joining = false,
            devices = devices.filter { it.identity.id != update.device.identity.id } + update.device,
            pairedCount = pairedCount + 1,
            reachable = reachable?.plus(update.device.identity.id),
        )
        is Update.PairingFailed -> copy(pairingCode = null, joining = false, pairingProblem = update.problem)
        is Update.SendFailed -> copy(problems = problems + (update.name to update.problem))
        is Update.TextReceived -> copy(
            texts = (listOf(ReceivedText(now, update.name, update.text, update.autoCopy)) + texts).take(MAX_TEXTS),
        )
        is Update.TextShared -> update.problem?.let { copy(problems = problems + (update.name to it)) } ?: this
    }

    private fun transfer(id: String, change: (Transfer) -> Transfer) =
        copy(transfers = transfers.map { if (it.id == id) change(it) else it })
}

private fun Transfer.sample(bytes: Long, total: Long, now: Long): Transfer {
    val elapsed = (now - sampledAt) / 1e9
    val speed = if (sampledAt == 0L || elapsed <= 0) bytesPerSecond else {
        bytesPerSecond + ((bytes - sampledBytes) / elapsed - bytesPerSecond) * SPEED_SMOOTHING
    }
    return copy(transferred = bytes, total = total, bytesPerSecond = speed, sampledAt = now, sampledBytes = bytes, phase = Phase.Active)
}
