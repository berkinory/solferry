package app.solferry

import uniffi.solferry_ffi.BlockedDevice
import uniffi.solferry_ffi.HistoryEntry
import uniffi.solferry_ffi.Identity
import uniffi.solferry_ffi.Incoming
import uniffi.solferry_ffi.Nearby
import uniffi.solferry_ffi.PairedDevice
import uniffi.solferry_ffi.Problem
import uniffi.solferry_ffi.Relay
import uniffi.solferry_ffi.Update

private const val MAX_TOASTS = 3

enum class Phase { Waiting, Active, Reconnecting, Done, Failed }

enum class Tone { Success, Error }

/** A short message under the transfer pill, gone after a few seconds. */
data class Toast(val key: Long, val tone: Tone, val title: String)

data class Transfer(
    val id: String,
    val peerName: String,
    val label: String,
    val sending: Boolean,
    val transferred: Long = 0,
    val total: Long,
    val phase: Phase = Phase.Active,
    val problem: Problem? = null,
) {
    val running get() = phase == Phase.Waiting || phase == Phase.Active || phase == Phase.Reconnecting
    val fraction get() = if (total == 0L) 1f else transferred.toFloat() / total
    val outcome: Pair<Tone, String> get() {
        problem?.let { return Tone.Error to it.title }
        return Tone.Success to "${if (sending) "Sent" else "Received"} $label"
    }
}

/** A link a browser can open. `received` lists what browsers sent through it so far. */
data class LinkState(val urls: List<String>, val receiving: Boolean, val received: List<String> = emptyList())

data class PickedFile(val uri: android.net.Uri, val name: String, val size: Long?)

data class PairingPrompt(val key: ULong, val peer: Identity, val name: String, val code: String)

data class ReceivedText(val key: Long, val name: String, val text: String, val copied: Boolean)

data class Me(val id: String, val fingerprint: String, val name: String, val visibility: String, val checkUpdates: Boolean? = null)

data class AvailableUpdate(val version: String, val page: String)

data class AppState(
    val me: Me? = null,
    val startProblem: Problem? = null,
    val startReason: String = "",
    val devices: List<PairedDevice> = emptyList(),
    val nearby: List<Nearby> = emptyList(),
    val reachable: Set<String>? = null,
    val relays: List<Relay> = emptyList(),
    val blocked: List<BlockedDevice> = emptyList(),
    val transfers: List<Transfer> = emptyList(),
    val history: List<HistoryEntry> = emptyList(),
    val offers: List<Incoming> = emptyList(),
    val pairings: List<PairingPrompt> = emptyList(),
    val pairingCode: String? = null,
    val invite: String? = null,
    val joining: Boolean = false,
    val joinError: String? = null,
    val update: AvailableUpdate? = null,
    val text: ReceivedText? = null,
    val toasts: List<Toast> = emptyList(),
    val link: LinkState? = null,
    val toastsMade: Long = 0,
    val standby: Boolean = false,
    val pairedCount: Int = 0,
) {
    val strangers get() = nearby.filter { !it.known }

    fun apply(update: Update, now: Long): AppState = when (update) {
        is Update.Ready -> copy(me = Me(update.id, update.fingerprint, update.name, update.visibility, update.checkUpdates), startProblem = null)
        is Update.StartFailed -> copy(startProblem = update.problem, startReason = update.reason)
        is Update.Devices -> copy(devices = update.devices)
        is Update.History -> copy(history = update.entries)
        is Update.Nearby -> copy(nearby = update.devices)
        is Update.Reachable -> copy(reachable = update.ids.toSet())
        is Update.Relays -> copy(relays = update.relays)
        is Update.RelayFailed -> toast(Tone.Error, update.problem.title)
        is Update.Blocked -> copy(blocked = update.devices)
        is Update.UpdateAvailable -> toast(Tone.Success, "Solferry ${update.version} is available").copy(update = AvailableUpdate(update.version, update.page))
        is Update.Waiting -> begin(Transfer(update.id, update.peerName, update.label, true, total = update.bytes.toLong(), phase = Phase.Waiting))
        is Update.Started -> begin(Transfer(update.id, update.peerName, update.label, update.sending, total = update.bytes.toLong()))
        is Update.Progress -> transfer(update.id) { it.copy(transferred = update.transferred.toLong(), total = update.total.toLong(), phase = Phase.Active) }
        is Update.RouteChanged -> this
        is Update.Reconnecting -> transfer(update.id) { it.copy(phase = Phase.Reconnecting) }
        is Update.Completed -> transfer(update.id) { it.copy(phase = Phase.Done, transferred = it.total) }.settle(update.id)
        is Update.Cancelled -> copy(transfers = transfers.filter { it.id != update.id })
        is Update.Failed ->
            if (transfers.find { it.id == update.id }?.sending == true) copy(transfers = transfers.filter { it.id != update.id })
            else transfer(update.id) { it.copy(phase = Phase.Failed, problem = update.problem) }.settle(update.id)
        is Update.Offer -> copy(offers = offers + update.incoming)
        is Update.OfferGone -> copy(offers = offers.filter { it.key != update.key })
        is Update.PairingRequest -> copy(pairings = pairings + PairingPrompt(update.key, update.peer, update.name, update.code))
        is Update.PairingGone -> copy(pairings = pairings.filter { it.key != update.key })
        is Update.PairingCode -> copy(pairingCode = update.code)
        is Update.Invite -> copy(invite = update.code)
        is Update.Paired -> toast(Tone.Success, "Paired with ${update.device.name}").copy(
            pairingCode = null, invite = null, joining = false, joinError = null,
            devices = devices.filter { it.identity.id != update.device.identity.id } + update.device,
            pairedCount = pairedCount + 1,
            reachable = reachable?.plus(update.device.identity.id),
        )
        is Update.PairingFailed -> (if (joining) copy(joinError = update.problem.title) else toast(Tone.Error, update.problem.title)).copy(pairingCode = null, joining = false)
        is Update.SendFailed -> toast(Tone.Error, update.problem.title)
        is Update.TextReceived -> copy(
            text = ReceivedText(now, update.name, update.text, update.autoCopy),
        )
        is Update.TextShared -> update.problem?.let { toast(Tone.Error, it.title) } ?: this
        is Update.LinkReady -> copy(link = LinkState(update.urls, update.receiving))
        is Update.LinkStopped -> copy(link = null)
        is Update.LinkFailed -> toast(Tone.Error, update.problem.title).copy(link = null)
        is Update.LinkReceived -> toast(Tone.Success, "Received ${update.name}").copy(link = link?.let { it.copy(received = it.received + update.name) })
    }

    fun toast(tone: Tone, title: String) =
        copy(toasts = (toasts + Toast(toastsMade + 1, tone, title)).takeLast(MAX_TOASTS), toastsMade = toastsMade + 1)

    /** The last transfer to end keeps the pill for a moment. One ending beside others moves to a toast underneath. */
    private fun settle(id: String): AppState {
        val ended = transfers.find { it.id == id } ?: return this
        if (transfers.none { it.running }) return this
        val (tone, title) = ended.outcome
        return copy(transfers = transfers - ended).toast(tone, title)
    }

    private fun begin(transfer: Transfer) = copy(transfers = listOf(transfer) + transfers.filter { it.id != transfer.id && it.running })

    private fun transfer(id: String, change: (Transfer) -> Transfer) =
        copy(transfers = transfers.map { if (it.id == id) change(it) else it })
}
