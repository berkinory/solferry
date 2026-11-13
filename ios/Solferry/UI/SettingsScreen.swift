import SwiftUI
import UniformTypeIdentifiers

private let guide = URL(string: "https://github.com/berkinory/solferry/blob/main/docs/relay.md")!

struct SettingsScreen: View {
    @Environment(Engine.self) private var engine
    @State private var name = ""

    var body: some View {
        let me = engine.state.me
        let changed = !name.trimmingCharacters(in: .whitespaces).isEmpty && name != me?.name
        Page(title: "Settings") {
            Card(glyph: .computer, title: "Device name") {
                PaddedWell {
                    HStack(spacing: SpaceTokens.sm) {
                        Field(text: $name, placeholder: "Name") { save(changed) }
                        ActionButton(label: "Save") { save(changed) }.disabled(!changed)
                    }
                }
            }
            PermissionsCard()
            RelayCard()
            SaveFolderCard()
            VisibilityCard(found: me?.visibility != "Hidden")
            HeicCard()
            AppearanceCard()
            if !engine.state.blocked.isEmpty { BlockedCard(blocked: engine.state.blocked) }
            HStack {
                Caption("Solferry \(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "")")
                Spacer()
                Caption(me?.fingerprint ?? "")
            }
            .padding(.horizontal, SpaceTokens.md)
            .padding(.vertical, SpaceTokens.sm)
        }
        .onChange(of: me?.name, initial: true) { _, saved in name = saved ?? "" }
    }

    private func save(_ changed: Bool) {
        if changed { engine.bridge.rename(name: name.trimmingCharacters(in: .whitespaces)) }
    }
}

private struct SaveFolderCard: View {
    @Environment(Engine.self) private var engine
    @State private var picking = false

    var body: some View {
        let folder = engine.state.saveFolder
        Card(glyph: .folder, title: "Save files to") {
            PaddedWell {
                HStack(spacing: SpaceTokens.sm) {
                    Muted(verbatim: folder?.name ?? String(localized: "The Solferry folder in the Files app"))
                    Spacer(minLength: 0)
                    if folder != nil {
                        ActionButton(label: "Reset", kind: .ghost, scale: .small) { engine.resetFolder() }
                    }
                    ActionButton(label: "Change", glyph: .folder, scale: .small) { picking = true }
                }
            }
        }
        .fileImporter(isPresented: $picking, allowedContentTypes: [.folder]) { result in
            if case let .success(url) = result { engine.chooseFolder(url) }
        }
    }
}

private struct VisibilityCard: View {
    @Environment(Engine.self) private var engine
    let found: Bool

    var body: some View {
        Card(glyph: .view, title: "Let others find me") {
            PaddedWell {
                Toggle(isOn: Binding(get: { found }, set: { engine.bridge.setReach(reach: $0 ? .everyone : .hidden) })) {
                    Muted(found ? "Devices on your network can see this one and ask to send." : "Nobody new can find this device. Paired devices can still reach it.")
                }
            }
        }
    }
}

private struct HeicCard: View {
    @AppStorage(Heic.key) private var convert = true

    var body: some View {
        Card(glyph: .typeImage, title: "Convert HEIC photos to JPEG") {
            PaddedWell {
                Toggle(isOn: $convert) {
                    Muted("For Linux and Windows devices, which can't open HEIC. Converted on this device with no visible quality loss.")
                }
            }
        }
    }
}

private struct RelayRow: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let relay: Relay
    @State private var copied = false

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            Tile(glyph: .relay, fill: palette.control, tone: palette.inkMuted)
            VStack(alignment: .leading, spacing: 0) {
                Text(verbatim: relay.shown).lineLimit(1)
                status
            }
            Spacer(minLength: 0)
            ActionButton(glyph: copied ? .check : .copy, kind: .ghost, scale: .small, description: copied ? "Copied" : "Copy link") {
                engine.copy(relay.link)
                copied = true
            }
            ActionButton(glyph: .delete, kind: .ghost, scale: .small, description: "Remove \(relay.shown)") {
                engine.bridge.removeRelay(link: relay.link)
            }
        }
    }

    @ViewBuilder private var status: some View {
        switch relay.status {
        case .checking: StatusLine("Checking")
        case .connected: StatusLine("Connected", dot: palette.success)
        case .udpBlocked: StatusLine("Connected, UDP 7842 is blocked", dot: palette.warning)
        case .refused: StatusLine("Refused, check the link", dot: palette.danger)
        case .unreachable: StatusLine("Can't reach the server", dot: palette.danger)
        }
    }
}

private struct RelayCard: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @Environment(\.openURL) private var openURL
    @State private var link = ""

    var body: some View {
        let state = engine.state
        Card(glyph: .relay, title: "Relay server") {
            ActionButton(label: "Setup Guide", glyph: .external, kind: .ghost, scale: .small) { openURL(guide) }
        } content: {
            PaddedWell {
                Muted("Reach devices on other networks through a relay server you run.")
                ForEach(state.relays, id: \.link) { RelayRow(relay: $0) }
                HStack(spacing: SpaceTokens.sm) {
                    Field(text: $link, placeholder: "https://relay.example.com#token", onSubmit: add) { PasteInto(text: $link) }
                    ActionButton(label: "Add", action: add)
                }
                RelaySpeedLimit(current: state.me?.relaySpeedLimit)
            }
        }
        .onChange(of: state.relays.count) { link = "" }
    }

    private func add() {
        let trimmed = link.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty { engine.bridge.addRelay(link: trimmed) }
    }
}

/// How fast files may go through a relay, typed in MB/s, 0 for no limit. Saving restarts the node, so it waits for Save.
private struct RelaySpeedLimit: View {
    @Environment(Engine.self) private var engine
    let current: UInt64?
    @State private var typed = ""
    private let mib: UInt64 = 1024 * 1024

    private var saved: String { String((current ?? 0) / mib) }

    var body: some View {
        VStack(alignment: .leading, spacing: SpaceTokens.xs) {
            Text("Relay speed limit")
            Muted("Only applies while files go through the relay. Direct connections are never limited.")
            Muted(verbatim: "0 = " + String(localized: "No limit"))
            HStack(spacing: SpaceTokens.sm) {
                Field(text: $typed, placeholder: "0", onSubmit: commit) {
                    if typed != saved {
                        ActionButton(label: "Save", kind: .primary, scale: .small, action: commit)
                    }
                }
                .keyboardType(.numberPad)
                .frame(width: 180)
                Muted(verbatim: "MB/s")
            }
        }
        .padding(.top, SpaceTokens.sm)
        .onChange(of: current, initial: true) { typed = saved }
        .onChange(of: typed) { _, value in
            let digits = String(value.filter(\.isNumber).prefix(6))
            if digits != value { typed = digits }
        }
    }

    private func commit() {
        guard let megabytes = UInt64(typed) else {
            typed = saved
            return
        }
        let limit = megabytes == 0 ? nil : megabytes * mib
        if limit != current { engine.bridge.setRelaySpeedLimit(limit: limit) }
    }
}

private struct BlockedCard: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let blocked: [BlockedDevice]

    var body: some View {
        Card(glyph: .blocked, title: "Blocked") {
            Count(value: blocked.count)
        } content: {
            Well {
                ForEach(blocked, id: \.identity.id) { device in
                    HStack(spacing: SpaceTokens.md) {
                        Tile(glyph: .blocked, fill: palette.control, tone: palette.inkMuted)
                        VStack(alignment: .leading, spacing: 0) {
                            Text(device.name ?? device.identity.fingerprint).lineLimit(1)
                            if device.name != nil { Caption(device.identity.fingerprint) }
                        }
                        Spacer()
                        ActionButton(label: "Unblock", scale: .small) { engine.bridge.unblock(peer: device.identity.id) }
                    }
                    .padding(SpaceTokens.sm)
                }
            }
        }
    }
}
