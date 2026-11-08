import SwiftUI

private let guide = URL(string: "https://github.com/berkinory/solferry/blob/main/docs/relay.md")!

private let reaches: [(reach: Reach, label: String, detail: String)] = [
    (.everyone, "Everyone", "Anyone on your network can see this device and ask to send."),
    (.paired, "Paired", "Others can see this device, but only paired devices can send."),
    (.hidden, "Hidden", "This device doesn't show up nearby. Paired devices can still send."),
]

private func index(of visibility: String) -> Int {
    switch visibility {
    case "KnownDevices": 1
    case "Hidden": 2
    default: 0
    }
}

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
            Card(glyph: .folder, title: "Save files to") {
                PaddedWell { Muted("The Solferry folder in the Files app") }
            }
            VisibilityCard(current: index(of: me?.visibility ?? ""))
            RelayCard()
            if !engine.state.blocked.isEmpty { BlockedCard(blocked: engine.state.blocked) }
            HStack {
                Mono("Solferry \(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "")")
                Spacer()
                Mono(me?.fingerprint ?? "")
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
    let current: Int

    var body: some View {
        Card(glyph: .view, title: "Who can see this device") {
            PaddedWell {
                HStack(spacing: SpaceTokens.xs) {
                    ForEach(reaches.indices, id: \.self) { position in
                        ActionButton(
                            label: reaches[position].label,
                            kind: position == current ? .primary : .ghost,
                            scale: .small,
                            wide: true
                        ) { engine.bridge.setReach(reach: reaches[position].reach) }
                        .accessibilityAddTraits(position == current ? .isSelected : [])
                    }
                }
                Muted(reaches[current].detail)
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
        Card(glyph: .relay, title: "Your relays") {
            ActionButton(label: "Set one up", glyph: .external, kind: .ghost, scale: .small) { openURL(guide) }
        } content: {
            PaddedWell {
                Muted("Reach devices on other networks through a relay you run.")
                ForEach(state.relays, id: \.link) { RelayRow(relay: $0) }
                if let problem = state.relayProblem { Muted(problem.next, color: palette.danger) }
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
    let blocked: [Identity]

    var body: some View {
        Card(glyph: .blocked, title: "Blocked") {
            Count(value: blocked.count)
        } content: {
            Well {
                ForEach(blocked, id: \.id) { device in
                    HStack(spacing: SpaceTokens.md) {
                        Tile(glyph: .blocked, fill: palette.control, tone: palette.inkMuted)
                        Mono(device.fingerprint)
                        Spacer()
                        ActionButton(label: "Unblock", scale: .small) { engine.bridge.unblock(peer: device.id) }
                    }
                    .padding(SpaceTokens.sm)
                }
            }
        }
    }
}
