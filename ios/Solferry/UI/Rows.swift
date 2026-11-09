import SwiftUI

private func routeName(_ route: TransferRoute) -> String {
    switch route {
    case .local: "local network"
    case .overlay: "overlay network"
    case .direct: "direct"
    case .relay: "relay"
    }
}

private struct RowBody<Content: View>: View {
    @Environment(\.palette) private var palette
    var highlighted = false
    @ViewBuilder let content: Content

    var body: some View {
        HStack(spacing: SpaceTokens.md) { content }
            .padding(SpaceTokens.sm)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(highlighted ? palette.raised : .clear, in: controlShape)
    }
}

private struct Detail<Trailing: View>: View {
    @Environment(\.palette) private var palette
    let label: String
    var moment: String?
    @ViewBuilder let trailing: Trailing

    var body: some View {
        HStack(spacing: SpaceTokens.xs) {
            Muted(label)
            if let moment { Muted(moment, color: palette.inkMuted.opacity(0.6)) }
            Spacer()
            trailing
        }
        .frame(minHeight: 46)
        .padding(.leading, 58)
        .padding(.trailing, SpaceTokens.sm)
    }
}

private struct DeviceRow: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @State private var open = false
    @State private var forgetting = false
    let device: PairedDevice
    let near: Nearby?
    let online: Bool?
    let lastTransfer: UInt64?
    let onPick: (PairedDevice) -> Void

    var body: some View {
        VStack(spacing: 0) {
            RowBody(highlighted: open) {
                // Only this part picks the device: a press on iOS's paste control would reach it too.
                HStack(spacing: SpaceTokens.md) {
                    DeviceTile(identity: device.identity, platform: near?.identity.platform, online: near != nil || online == true)
                    VStack(alignment: .leading, spacing: 0) {
                        Text(device.name).lineLimit(1)
                        status
                    }
                    Spacer(minLength: 0)
                }
                .contentShape(Rectangle())
                .onTapGesture { onPick(device) }
                .accessibilityAddTraits(.isButton)
                .contextMenu { menu }
                HStack(spacing: 2) {
                    ClipboardButton { engine.bridge.sendText(peer: device.identity.id, text: $0) }
                    ActionButton(glyph: .more, kind: .ghost, scale: .small, description: "Options for \(device.name)") {
                        withAnimation(.easeOut(duration: MotionTokens.standard / 1000)) { open.toggle() }
                    }
                }
            }
            if open { details }
        }
        .confirm(
            "Forget \(device.name)?",
            isPresented: $forgetting,
            message: "It goes back to being an unpaired device until you pair again.",
            action: "Forget"
        ) { engine.bridge.forget(peer: device.identity.id) }
    }

    @ViewBuilder private var menu: some View {
        Button { onPick(device) } label: { Label { Text("Send files") } icon: { Image(decorative: Glyph.send.rawValue) } }
        Toggle(
            "Receive without asking",
            isOn: Binding(
                get: { device.autoAccept },
                set: { engine.bridge.setAutoAccept(peer: device.identity.id, enabled: $0) }
            )
        )
        Divider()
        Button(role: .destructive) { forgetting = true } label: { Label { Text("Forget") } icon: { Image(decorative: Glyph.delete.rawValue) } }
    }

    @ViewBuilder private var status: some View {
        if near != nil || online == true {
            StatusLine("Online", dot: palette.successFill)
        } else if online == false {
            StatusLine("Offline", dot: palette.inkMuted.opacity(0.5))
        } else {
            StatusLine("Paired")
        }
    }

    private var details: some View {
        VStack(spacing: 0) {
            Detail(label: "Receive without asking") {
                Toggle(
                    "Receive without asking",
                    isOn: Binding(
                        get: { device.autoAccept },
                        set: { engine.bridge.setAutoAccept(peer: device.identity.id, enabled: $0) }
                    )
                )
                .labelsHidden()
            }
            Detail(label: lastTransfer == nil ? "Paired" : "Last transfer", moment: ago(lastTransfer ?? device.pairedAtMs)) {
                ActionButton(label: "Forget", glyph: .delete, kind: .danger, scale: .small) { forgetting = true }
            }
        }
        .transition(.opacity.combined(with: .move(edge: .top)))
    }
}

struct DeviceCard: View {
    let state: AppState
    let onPick: (PairedDevice) -> Void

    var body: some View {
        Card(glyph: .devices, title: "Devices") {
            Count(value: state.devices.count)
        } content: {
            Well {
                ForEach(state.devices, id: \.identity.id) { device in
                    DeviceRow(
                        device: device,
                        near: state.nearby.first { $0.identity.id == device.identity.id },
                        online: state.reachable?.contains(device.identity.id),
                        lastTransfer: state.history.first { $0.peer == device.identity.id && $0.completed }?.finishedMs,
                        onPick: onPick
                    )
                }
            }
        }
    }
}

struct NearbyCard: View {
    @Environment(Engine.self) private var engine
    @Environment(\.addDevice) private var addDevice
    let nearby: [Nearby]
    let alone: Bool

    var body: some View {
        Card(glyph: .nearby, title: "Nearby") {
            ActionButton(label: "Use a code", glyph: .qr, kind: .ghost, scale: .small, action: addDevice)
        } content: {
            Well {
                if nearby.isEmpty { if alone { Searching() } else { Scanning() } }
                ForEach(nearby, id: \.identity.id) { device in
                    RowBody {
                        DeviceTile(identity: device.identity)
                        VStack(alignment: .leading, spacing: 0) {
                            Text(device.name ?? "Unnamed device").lineLimit(1)
                            StatusLine("Not paired")
                        }
                        Spacer(minLength: 0)
                        ActionButton(label: "Pair", glyph: .link, scale: .small) { engine.bridge.pairNearby(peer: device.identity.id) }
                    }
                }
            }
        }
    }
}
