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
    let label: String
    @ViewBuilder let trailing: Trailing

    var body: some View {
        HStack {
            Muted(label)
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
                    DeviceTile(identity: device.identity, platform: near?.identity.platform)
                    VStack(alignment: .leading, spacing: 0) {
                        Text(device.name).lineLimit(1)
                        status
                    }
                    Spacer(minLength: 0)
                }
                .contentShape(Rectangle())
                .onTapGesture { onPick(device) }
                .accessibilityAddTraits(.isButton)
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

    @ViewBuilder private var status: some View {
        if near != nil || online == true {
            StatusLine("Online", dot: palette.success)
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
            Detail(label: lastTransfer.map { "Last transfer \(ago($0))" } ?? "Paired \(ago(device.pairedAtMs))") {
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
    let nearby: [Nearby]

    var body: some View {
        Card(glyph: .nearby, title: "Nearby") {
            NavigationLink(value: Route.addDevice) {
                HStack(spacing: 6) {
                    Icon(.qr, size: 16)
                    Text("Use a code").font(.buttonText)
                }
                .padding(.horizontal, SpaceTokens.sm)
                .frame(minHeight: 36)
            }
            .foregroundStyle(.secondary)
        } content: {
            Well {
                if nearby.isEmpty { Scanning() }
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

private struct TransferRow: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let transfer: Transfer

    var body: some View {
        RowBody {
            Tile(
                glyph: transfer.sending ? .upload : .download,
                fill: palette.control,
                tone: transfer.running ? palette.route(transfer.route) : palette.inkMuted
            )
            VStack(alignment: .leading, spacing: 2) {
                Text(transfer.problem?.title ?? transfer.label).lineLimit(1)
                StatusLine("\(transfer.sending ? "To" : "From") \(transfer.peerName)\(transfer.running ? ", \(routeName(transfer.route))" : "")")
                outcome
            }
            Spacer(minLength: 0)
            ActionButton(glyph: .cancel, kind: .ghost, scale: .small, description: transfer.running ? "Cancel" : "Clear") {
                if transfer.running { engine.bridge.cancel(transfer: transfer.id) } else { engine.clearTransfer(transfer.id) }
            }
        }
    }

    @ViewBuilder private var outcome: some View {
        switch transfer.phase {
        case .active, .reconnecting:
            let reconnecting = transfer.phase == .reconnecting
            ProgressLine(fraction: transfer.fraction, tone: reconnecting ? palette.warning : palette.primary)
                .padding(.top, SpaceTokens.xs)
            HStack {
                Caption("\(bytes(transfer.transferred)) / \(bytes(transfer.total))")
                Spacer()
                Caption(reconnecting ? "reconnecting" : "\(bytes(UInt64(max(transfer.bytesPerSecond, 0))))/s")
            }
        case .done: Caption(bytes(transfer.total), color: palette.success)
        case .failed: Muted(transfer.problem?.next ?? "", color: palette.danger)
        }
    }
}

struct TransferCard: View {
    let transfers: [Transfer]

    var body: some View {
        Card(glyph: .transfers, title: "Transfers") {
            Count(value: transfers.count)
        } content: {
            Well { ForEach(transfers) { TransferRow(transfer: $0) } }
        }
    }
}
