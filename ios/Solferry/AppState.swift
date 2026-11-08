import Foundation

private let speedSmoothing = 0.2
private let maxTexts = 3

enum Phase {
    case active, reconnecting, done, failed
}

struct Transfer: Identifiable {
    let id: String
    let peerName: String
    let label: String
    let sending: Bool
    var route: TransferRoute
    var transferred: UInt64 = 0
    var total: UInt64
    var bytesPerSecond = 0.0
    var phase = Phase.active
    var problem: Problem?
    var sampledAt: UInt64 = 0
    var sampledBytes: UInt64 = 0

    var running: Bool { phase == .active || phase == .reconnecting }
    var fraction: Double { total == 0 ? 1 : Double(transferred) / Double(total) }

    mutating func sample(_ bytes: UInt64, total: UInt64, now: UInt64) {
        let elapsed = Double(now - sampledAt) / 1e9
        if sampledAt != 0, elapsed > 0 {
            let moved = Double(bytes) - Double(sampledBytes)
            bytesPerSecond += (moved / elapsed - bytesPerSecond) * speedSmoothing
        }
        transferred = bytes
        self.total = total
        sampledAt = now
        sampledBytes = bytes
        phase = .active
    }
}

struct PairingPrompt: Identifiable {
    let key: UInt64
    let peer: Identity
    let name: String
    let code: String

    var id: UInt64 { key }
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
}

struct NamedProblem: Identifiable {
    let id = UUID()
    let name: String
    let problem: Problem
}

struct AppState {
    var me: Me?
    var startProblem: Problem?
    var devices: [PairedDevice] = []
    var nearby: [Nearby] = []
    var reachable: Set<String>?
    var relays: [Relay] = []
    var relayProblem: Problem?
    var blocked: [Identity] = []
    var transfers: [Transfer] = []
    var history: [HistoryEntry] = []
    var offers: [Incoming] = []
    var pairings: [PairingPrompt] = []
    var pairingCode: String?
    var invite: String?
    var pairingProblem: Problem?
    var joining = false
    var texts: [ReceivedText] = []
    var problems: [NamedProblem] = []
    var pairedCount = 0

    var strangers: [Nearby] { nearby.filter { !$0.known } }
    var asking: Int { offers.count + pairings.count }
    var hasRunningTransfer: Bool { transfers.contains { $0.running } }

    mutating func apply(_ update: Update, now: UInt64) {
        switch update {
        case let .ready(id, fingerprint, name, _, visibility):
            me = Me(id: id, fingerprint: fingerprint, name: name, visibility: visibility)
            startProblem = nil
        case let .startFailed(problem): startProblem = problem
        case let .devices(list): devices = list
        case let .history(entries): history = entries
        case let .nearby(list): nearby = list
        case let .reachable(ids): reachable = Set(ids)
        case let .relays(list):
            relays = list
            relayProblem = nil
        case let .relayFailed(problem): relayProblem = problem
        case let .blocked(list): blocked = list
        case let .started(id, peerName, label, sending, route, bytes):
            transfers.removeAll { $0.id == id }
            let started = Transfer(id: id, peerName: peerName, label: label, sending: sending, route: route, total: bytes)
            transfers.insert(started, at: 0)
        case let .progress(id, transferred, total):
            change(id) { $0.sample(transferred, total: total, now: now) }
        case let .routeChanged(id, route): change(id) { $0.route = route }
        case let .reconnecting(id): change(id) { $0.phase = .reconnecting }
        case let .completed(id):
            change(id) {
                $0.phase = .done
                $0.transferred = $0.total
            }
        case let .failed(id, problem):
            change(id) {
                $0.phase = .failed
                $0.problem = problem
            }
        case let .offer(incoming): offers.append(incoming)
        case let .offerGone(key): offers.removeAll { $0.key == key }
        case let .pairingRequest(key, peer, name, code):
            pairings.append(PairingPrompt(key: key, peer: peer, name: name, code: code))
        case let .pairingGone(key): pairings.removeAll { $0.key == key }
        case let .pairingCode(code): pairingCode = code
        case let .invite(code): invite = code
        case let .paired(device):
            pairingCode = nil
            invite = nil
            pairingProblem = nil
            joining = false
            devices.removeAll { $0.identity.id == device.identity.id }
            devices.append(device)
            pairedCount += 1
            reachable?.insert(device.identity.id)
        case let .pairingFailed(problem):
            pairingCode = nil
            joining = false
            pairingProblem = problem
        case let .sendFailed(name, problem):
            problems.append(NamedProblem(name: name, problem: problem))
        case let .textReceived(name, text, autoCopy):
            texts.insert(ReceivedText(id: now, name: name, text: text, copied: autoCopy), at: 0)
            texts = Array(texts.prefix(maxTexts))
        case let .textShared(name, problem):
            if let problem { problems.append(NamedProblem(name: name, problem: problem)) }
        }
    }

    private mutating func change(_ id: String, _ edit: (inout Transfer) -> Void) {
        if let index = transfers.firstIndex(where: { $0.id == id }) { edit(&transfers[index]) }
    }
}
