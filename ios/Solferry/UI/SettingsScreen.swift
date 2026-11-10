import SwiftUI

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
            RelayCard()
            Card(glyph: .folder, title: "Save files to") {
                PaddedWell { Muted("The Solferry folder in the Files app") }
            }
            VisibilityCard(found: me?.visibility != "Hidden")
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

private struct RelayRow: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let relay: Relay

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            Tile(glyph: .relay, fill: palette.control, tone: palette.inkMuted)
            VStack(alignment: .leading, spacing: 0) {
                Text(relay.url).lineLimit(1)
                status
            }
            Spacer(minLength: 0)
            ActionButton(glyph: .delete, kind: .ghost, scale: .small, description: "Remove \(relay.url)") {
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
        Card(glyph: .relay, title: "Relay") {
            ActionButton(label: "Setup Guide", glyph: .external, kind: .ghost, scale: .small) { openURL(guide) }
        } content: {
            PaddedWell {
                Muted("Reach devices on other networks through a relay you run.")
                ForEach(state.relays, id: \.link) { RelayRow(relay: $0) }
                HStack(spacing: SpaceTokens.sm) {
                    Field(text: $link, placeholder: "https://relay.example.com#token", onSubmit: add) { PasteInto(text: $link) }
                    ActionButton(label: "Add", action: add)
                }
            }
        }
        .onChange(of: state.relays.count) { link = "" }
    }

    private func add() {
        let trimmed = link.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty { engine.bridge.addRelay(link: trimmed) }
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
