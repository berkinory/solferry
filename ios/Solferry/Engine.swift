import Foundation
import Observation
import UIKit

/// Hands the core's updates to the main thread in the order they happened.
private final class BridgeEvents: Listener, @unchecked Sendable {
    weak var engine: Engine?

    func update(update: Update) {
        DispatchQueue.main.async { [weak self] in
            MainActor.assumeIsolated { self?.engine?.receive(update) }
        }
    }
}

@MainActor
@Observable
final class Engine {
    private(set) var state = AppState()
    @ObservationIgnored private(set) var bridge: Solferry!
    @ObservationIgnored private let keepalive = Keepalive()
    @ObservationIgnored private var active = true

    init() {
        let events = BridgeEvents()
        events.engine = self
        bridge = Solferry(
            dataDir: Engine.dataDirectory().path,
            downloadDir: URL.documentsDirectory.path,
            deviceName: UIDevice.current.name,
            listener: events
        )
        Notifier.requestPermission()
    }

    /// The device key lives here, so a restored backup can't make a second phone with this identity.
    private static func dataDirectory() -> URL {
        var directory = URL.applicationSupportDirectory.appending(path: "Solferry", directoryHint: .isDirectory)
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        var values = URLResourceValues()
        values.isExcludedFromBackup = true
        try? directory.setResourceValues(values)
        return directory
    }

    fileprivate func receive(_ update: Update) {
        state.apply(update, now: DispatchTime.now().uptimeNanoseconds)
        react(to: update)
        holdForTransfers()
    }

    private func react(to update: Update) {
        switch update {
        case let .textReceived(name, text, autoCopy):
            if autoCopy { copy(text) }
            Notifier.text(from: name, copied: autoCopy)
        case let .offer(incoming): Notifier.offer(key: incoming.key, name: incoming.name)
        case let .offerGone(key): Notifier.settleOffer(key)
        case let .pairingRequest(key, _, name, _): Notifier.pairing(key: key, name: name)
        case let .pairingGone(key): Notifier.settlePairing(key)
        case let .completed(id):
            if let done = state.transfers.first(where: { $0.id == id }), !done.sending {
                Notifier.received(label: done.label, name: done.peerName)
            }
        default: break
        }
    }

    func sceneChanged(active: Bool) {
        self.active = active
        holdForTransfers()
    }

    private func holdForTransfers() {
        keepalive.hold(!active && state.hasRunningTransfer)
    }

    func copy(_ text: String) {
        UIPasteboard.general.string = text
    }

    func send(_ urls: [URL], to peers: [String]) {
        let bridge = bridge!
        Task.detached {
            for peer in peers {
                let files = urls.compactMap(Outbox.open)
                if !files.isEmpty { bridge.sendFiles(peer: peer, files: files) }
            }
            Outbox.discard(urls)
        }
    }

    func clearPairingErrors() {
        state.joinError = nil
        state.addressError = nil
    }

    func join(_ code: String) {
        state.joining = true
        state.joinError = nil
        bridge.join(code: code)
    }

    func pairAddress(_ address: String) {
        state.pairingAddress = true
        state.addressError = nil
        bridge.pairAddress(address: address)
    }

    func answerOffer(_ key: UInt64, accept: Bool) {
        settleOffer(key)
        bridge.answerOffer(key: key, accept: accept)
    }

    func acceptAlways(_ key: UInt64) {
        settleOffer(key)
        bridge.acceptAlways(key: key)
    }

    func blockOffer(_ key: UInt64) {
        settleOffer(key)
        bridge.blockOffer(key: key)
    }

    private func settleOffer(_ key: UInt64) {
        state.offers.removeAll { $0.key == key }
        Notifier.settleOffer(key)
    }

    func answerPairing(_ key: UInt64, accept: Bool) {
        state.pairings.removeAll { $0.key == key }
        Notifier.settlePairing(key)
        bridge.answerPairing(key: key, accept: accept)
    }

    func dismissToast(_ id: Int) {
        state.toasts.removeAll { $0.id == id }
    }

    /// Closing the pairing screen says no to whoever asked, or just stops showing our own code.
    func closePairing() {
        if let asked = state.pairings.first { answerPairing(asked.key, accept: false) } else { state.pairingCode = nil }
    }

    func dismissText(_ id: UInt64) {
        if state.text?.id == id { state.text = nil }
    }

    func markCopied(_ id: UInt64) {
        if state.text?.id == id { state.text?.copied = true }
    }

    func clearTransfer(_ id: String) {
        state.transfers.removeAll { $0.id == id }
    }
}

/// iOS suspends an app soon after it leaves the screen. This asks for the short time it allows,
/// so a transfer in progress can finish or reach a point it can resume from.
@MainActor
private final class Keepalive {
    private var task = UIBackgroundTaskIdentifier.invalid

    func hold(_ wanted: Bool) {
        if !wanted {
            release()
        } else if task == .invalid {
            task = UIApplication.shared.beginBackgroundTask(withName: "Solferry transfers") { [weak self] in
                MainActor.assumeIsolated { self?.release() }
            }
        }
    }

    private func release() {
        guard task != .invalid else { return }
        UIApplication.shared.endBackgroundTask(task)
        task = .invalid
    }
}
