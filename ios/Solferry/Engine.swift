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

    func reopen(reference: String) -> Int32? {
        QueueFiles.reopen(reference)
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
        setLocale(locale: Bundle.main.preferredLocalizations.first ?? "en")
        bridge = Solferry(
            dataDir: Engine.dataDirectory().path,
            downloadDir: Exporter.staging.path,
            deviceName: UIDevice.current.name,
            listener: events
        )
        Notifier.listen(self)
        let folder = destination()
        Task.detached { [weak self] in
            let saved = Exporter.recover(to: folder)
            await self?.exported(saved, by: String(localized: "Received files"), to: folder)
        }
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
        case let .linkReceived(name, _):
            let folder = destination()
            Task.detached { [weak self] in
                let saved = Exporter.export(to: folder)
                await self?.exported(saved, by: name, to: folder)
            }
        case let .queueSettled(id, name, label, references, result):
            let others = Set(state.queue.filter { $0.id != id }.flatMap(\.references))
            for reference in references where !others.contains(reference) { QueueFiles.release(reference) }
            switch result {
            case .delivered: Notifier.queueDelivered(label: label, name: name)
            case let .failed(problem): Notifier.queueFailed(reason: problem.title)
            case .dropped: break
            }
        case let .completed(id):
            if let done = state.transfers.first(where: { $0.id == id }), !done.sending {
                let (folder, name, label) = (destination(), done.peerName, done.label)
                Task.detached { [weak self] in
                    let saved = Exporter.export(to: folder)
                    await self?.exported(saved, by: name, to: folder, label: label)
                }
            }
        default: break
        }
    }

    private func exported(_ saved: Bool, by name: String, to folder: SaveFolder?, label: String? = nil) {
        let place = folder?.name ?? String(localized: "Files")
        if !saved {
            state.toast(.failure, String(localized: "\(name): Couldn't save to \(place)"))
        } else if let label {
            Notifier.received(label: label, name: name, folder: place)
        }
    }

    func chooseFolder(_ picked: URL) {
        if let folder = SaveFolder.keep(picked) { state.saveFolder = folder } else { state.toast(.failure, String(localized: "Couldn't use that folder")) }
    }

    func resetFolder() {
        SaveFolder.forget()
        state.saveFolder = nil
    }

    /// The picked folder if it still works. Once it doesn't, files go to the Solferry folder and the user hears about it once.
    private func destination() -> SaveFolder? {
        guard let name = SaveFolder.pickedName else { return nil }
        state.saveFolder = SaveFolder.resolve()
        if state.saveFolder == nil {
            SaveFolder.forget()
            state.toast(.failure, String(localized: "Lost access to \(name), saving to the Solferry folder"))
        }
        return state.saveFolder
    }

    func sceneChanged(active: Bool) {
        self.active = active
        holdForTransfers()
    }

    /// iOS suspends a backgrounded app within seconds, so paired devices hear it's going instead
    /// of waiting for their probes to fail. Its next presence check brings it back.
    func leaveUnlessStaying() {
        if !state.hasRunningTransfer { bridge.leave() }
    }

    private func holdForTransfers() {
        keepalive.hold(!active && state.hasRunningTransfer)
    }

    func copy(_ text: String) {
        UIPasteboard.general.string = text
    }

    func shareLink(_ urls: [URL]) {
        let bridge = bridge!
        Task.detached {
            let files = urls.compactMap(Outbox.open)
            if !files.isEmpty { bridge.shareLink(files: files) }
            Outbox.discard(urls)
        }
    }

    /// A paired device known to be away gets the files once it's back, instead of a send that can't connect.
    func send(_ urls: [URL], to peers: [String]) {
        let bridge = bridge!
        let away = Set(peers.filter { peer in
            state.devices.contains { $0.identity.id == peer } && state.reachable.map { !$0.contains(peer) } == true
        })
        let wantJpeg = Heic.enabled ? Set(peers.filter { !away.contains($0) && Heic.wanted(platform(of: $0)) }) : []
        Task.detached { [weak self] in
            let jpegs = wantJpeg.isEmpty ? [:] : Heic.jpegCopies(of: urls)
            for peer in peers {
                if away.contains(peer) {
                    let kept = urls.compactMap(QueueFiles.keep)
                    if kept.count == urls.count {
                        bridge.queueFiles(peer: peer, files: kept)
                    } else {
                        kept.forEach { QueueFiles.release($0.reference) }
                        await self?.toastFailure(String(localized: "Couldn't open that file, try sharing it again"))
                    }
                    continue
                }
                let chosen = wantJpeg.contains(peer) ? urls.map { jpegs[$0] ?? $0 } : urls
                let files = chosen.compactMap(Outbox.open)
                if !files.isEmpty { bridge.sendFiles(peer: peer, files: files) }
            }
            Outbox.discard(urls + jpegs.values)
        }
    }

    private func platform(of peer: String) -> DevicePlatform? {
        let identity = state.devices.first { $0.identity.id == peer }?.identity ?? state.nearby.first { $0.identity.id == peer }?.identity
        return identity?.platform
    }

    private func toastFailure(_ title: String) {
        state.toast(.failure, title)
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
