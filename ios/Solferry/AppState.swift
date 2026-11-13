import Foundation

private let maxToasts = 3

enum Phase {
    case waiting, active, reconnecting, done, failed
}

struct Transfer: Identifiable {
    let id: String
    let peerName: String
    let label: String
    let sending: Bool
    var transferred: UInt64 = 0
    var total: UInt64
    var phase = Phase.active
    var problem: Problem?

    var running: Bool { phase == .waiting || phase == .active || phase == .reconnecting }
    var fraction: Double { total == 0 ? 1 : Double(transferred) / Double(total) }
    var outcome: (tone: Tone, title: String) {
        if let problem { return (.failure, problem.title) }
        return (.success, sending ? String(localized: "Sent \(label)") : String(localized: "Received \(label)"))
    }}

struct PairingPrompt: Identifiable {
    let key: UInt64
    let peer: Identity
    let name: String
    let code: String

    var id: UInt64 { key }
}

/// A link a browser can open, with the files and texts browsers sent through it so far.
struct LinkState {
    let urls: [String]
    let receiving: Bool
    var received: [String] = []
    var texts: [String] = []
}

struct ReceivedText: Identifiable {
    let id: UInt64
    let name: String
    let text: String
    var copied: Bool
}

struct Me {
    let id: String
    let fingerprint: String
    let name: String
    let visibility: String
    let relaySpeedLimit: UInt64?
}

enum Tone {
    case success, failure
}

/// A short message under the transfer pill, gone after a few seconds.
struct Toast: Identifiable {
    let id: Int
    let tone: Tone
    let title: String
}

struct AppState {
    var me: Me?
    var startProblem: Problem?
    var startReason = ""
    var devices: [PairedDevice] = []
    var nearby: [Nearby] = []
    var reachable: Set<String>?
    var relays: [Relay] = []
    var blocked: [BlockedDevice] = []
    var transfers: [Transfer] = []
    var history: [HistoryEntry] = []
    /// Sends waiting for paired devices that weren't there.
    var queue: [QueueEntry] = []
    var offers: [Incoming] = []
    var pairings: [PairingPrompt] = []
    var pairingCode: String?
    var invite: String?
    var joining = false
    var joinError: String?
    var pairingAddress = false
    var addressError: String?
    var text: ReceivedText?
    var link: LinkState?
    var toasts: [Toast] = []
    var toastsMade = 0
    var pairedCount = 0
    /// Nil while files go to the Solferry folder in Files.
    var saveFolder: SaveFolder?

    /// Nearby devices not paired yet. Read against the paired list, which changes first.
    var strangers: [Nearby] { nearby.filter { near in !devices.contains { $0.identity.id == near.identity.id } } }
    var asking: Int { offers.count + pairings.count }
    var hasRunningTransfer: Bool { transfers.contains { $0.running } }

    mutating func apply(_ update: Update, now: UInt64) {
        switch update {
        case let .ready(id, fingerprint, name, _, visibility, _, relaySpeedLimit):
            me = Me(id: id, fingerprint: fingerprint, name: name, visibility: visibility, relaySpeedLimit: relaySpeedLimit)
            startProblem = nil
        case let .startFailed(problem, reason):
            startProblem = problem
            startReason = reason
        case let .devices(list): devices = list
        case let .history(entries): history = entries
        case let .nearby(list): nearby = list
        case let .reachable(ids): reachable = Set(ids)
        case let .present(id): reachable = (reachable ?? []).union([id])
        case let .absent(id): reachable?.remove(id)
        case let .relays(list):
            relays = list
        case let .relayFailed(problem): toast(.failure, problem.title)
        case let .blocked(list): blocked = list
        case .updateAvailable: break
        case let .waiting(id, peerName, label, bytes):
            begin(Transfer(id: id, peerName: peerName, label: label, sending: true, total: bytes, phase: .waiting))
        case let .started(id, peerName, label, sending, _, bytes):
            begin(Transfer(id: id, peerName: peerName, label: label, sending: sending, total: bytes))
        case let .progress(id, transferred, total):
            change(id) {
                $0.transferred = transferred
                $0.total = total
                $0.phase = .active
            }
        case .routeChanged: break
        case let .reconnecting(id): change(id) { $0.phase = .reconnecting }
        case let .completed(id):
            change(id) {
                $0.phase = .done
                $0.transferred = $0.total
            }
            settle(id)
        case let .cancelled(id): transfers.removeAll { $0.id == id }
        case let .failed(id, problem):
            if transfers.first(where: { $0.id == id })?.sending == true {
                transfers.removeAll { $0.id == id }
            } else {
                change(id) {
                    $0.phase = .failed
                    $0.problem = problem
                }
                settle(id)
            }
        case let .offer(incoming): offers.append(incoming)
        case let .offerGone(key): offers.removeAll { $0.key == key }
        case let .pairingRequest(key, peer, name, code):
            pairings.append(PairingPrompt(key: key, peer: peer, name: name, code: code))
        case let .pairingGone(key): pairings.removeAll { $0.key == key }
        case let .pairingCode(code): pairingCode = code
        case let .invite(code): invite = code
        case let .paired(device):
            toast(.success, String(localized: "Paired with \(device.name)"))
            pairingCode = nil
            invite = nil
            joining = false
            pairingAddress = false
            addressError = nil
            devices.removeAll { $0.identity.id == device.identity.id }
            devices.append(device)
            pairedCount += 1
            reachable?.insert(device.identity.id)
        case let .addressFailed(problem):
            addressError = problem.title
            pairingAddress = false
        case let .pairingFailed(problem):
            if joining {
                joinError = problem.title
            } else if pairingAddress {
                addressError = problem.title
            } else {
                toast(.failure, problem.title)
            }
            pairingCode = nil
            joining = false
            pairingAddress = false
        case let .sendFailed(_, problem):
            toast(.failure, problem.title)
        case let .textReceived(name, text, autoCopy):
            self.text = ReceivedText(id: now, name: name, text: text, copied: autoCopy)
        case let .textShared(_, problem):
            if let problem { toast(.failure, problem.title) }
        case let .linkReady(urls, receiving): link = LinkState(urls: urls, receiving: receiving)
        case .linkStopped: link = nil
        case let .linkFailed(problem):
            link = nil
            toast(.failure, problem.title)
        case let .linkReceived(name, _):
            link?.received.append(name)
            toast(.success, String(localized: "Received \(name)"))
        case let .linkText(text):
            link?.texts.append(text)
        case let .queue(entries): queue = entries
        case let .queued(name): toast(.success, String(localized: "\(name) is offline. Sending when it's back"))
        case let .queueSettled(_, name, label, _, result):
            switch result {
            case .delivered: toast(.success, String(localized: "Delivered \(label) to \(name)"))
            case let .failed(problem): toast(.failure, problem.title)
            case .dropped: break
            }
        }
    }

    mutating func toast(_ tone: Tone, _ title: String) {
        toastsMade += 1
        toasts.append(Toast(id: toastsMade, tone: tone, title: title))
        toasts = Array(toasts.suffix(maxToasts))
    }

    /// The last transfer to end keeps the pill for a moment. One ending beside others moves to a toast underneath.
    private mutating func settle(_ id: String) {
        guard transfers.contains(where: \.running), let index = transfers.firstIndex(where: { $0.id == id }) else { return }
        let ended = transfers.remove(at: index).outcome
        toast(ended.tone, ended.title)
    }

    private mutating func begin(_ transfer: Transfer) {
        transfers.removeAll { $0.id == transfer.id || !$0.running }
        transfers.insert(transfer, at: 0)
    }

    private mutating func change(_ id: String, _ edit: (inout Transfer) -> Void) {
        if let index = transfers.firstIndex(where: { $0.id == id }) { edit(&transfers[index]) }
    }
}
