import SwiftUI

private struct Target: Identifiable {
    let identity: Identity
    let name: String
    let detail: String

    var id: String { identity.id }
}

private enum Launch {
    case idle, flying, sending, sent
}

/// Everything about one send on its own screen: what goes, and who gets it.
struct SendScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @Environment(\.dismiss) private var dismiss
    @State private var chosen: Set<String> = []
    @State private var appeared = false
    @Binding var files: [URL]

    var body: some View {
        NavigationStack {
            Page(title: "Send") {
                VStack(spacing: 0) {
                    ForEach(Array(files.enumerated()), id: \.element) { index, file in
                        FileRow(file: file) {
                            files.removeAll { $0 == file }
                            if files.isEmpty { dismiss() }
                        }
                        .opacity(appeared ? 1 : 0)
                        .offset(x: appeared ? 0 : -14)
                        .animation(.easeOut(duration: 0.3).delay(Double(index) * 0.045), value: appeared)
                    }
                }
                Connector(live: !chosen.isEmpty)
                CardFrame {
                    Well {
                        if targets.isEmpty { Scanning() }
                        ForEach(targets) { target in
                            TargetRow(target: target, chosen: chosen.contains(target.id)) {
                                if chosen.remove(target.id) == nil { chosen.insert(target.id) }
                            }
                        }
                    }
                }
                LaunchButton(chosen: chosen.count) {
                    engine.send(files, to: Array(chosen))
                } onDone: {
                    files = []
                    dismiss()
                }
                .padding(.top, SpaceTokens.sm)
            }
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button { files = []; dismiss() } label: { Icon(.cancel, size: 20) }
                        .accessibilityLabel("Cancel")
                }
            }
            .onAppear { appeared = true }
        }
    }

    private var targets: [Target] {
        let state = engine.state
        let mine = state.devices.map { device in
            let near = state.nearby.first { $0.identity.id == device.identity.id }
            var identity = device.identity
            identity.platform = identity.platform ?? near?.identity.platform
            return Target(identity: identity, name: device.name, detail: detail(near: near, online: state.reachable?.contains(device.identity.id)))
        }
        return mine + state.strangers.map { Target(identity: $0.identity, name: $0.name ?? "Unnamed device", detail: "Not paired") }
    }

    private func detail(near: Nearby?, online: Bool?) -> String {
        if near != nil || online == true { return "Online" }
        return online == false ? "Offline" : "Paired"
    }
}

private struct FileRow: View {
    @Environment(\.palette) private var palette
    let file: URL
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            Icon(fileGlyph(file.lastPathComponent), tint: palette.inkMuted, size: 22)
            Text(file.lastPathComponent).lineLimit(1).truncationMode(.middle)
            Spacer(minLength: 0)
            if let size { Muted(bytes(size)) }
            Button(action: onRemove) { Icon(.delete, tint: palette.danger, size: 18).frame(width: 36, height: 36).contentShape(Rectangle()) }
                .buttonStyle(.plain)
                .accessibilityLabel("Remove \(file.lastPathComponent)")
        }
        .padding(.leading, SpaceTokens.xs)
        .frame(minHeight: 48)
    }

    private var size: UInt64? {
        let scoped = file.startAccessingSecurityScopedResource()
        defer { if scoped { file.stopAccessingSecurityScopedResource() } }
        return (try? file.resourceValues(forKeys: [.fileSizeKey]).fileSize).flatMap { $0 }.map(UInt64.init)
    }
}

/// Dots running from the files down to the devices, lit once there is somewhere to go.
private struct Connector: View {
    @Environment(\.palette) private var palette
    let live: Bool
    private let dots = 5

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 20)) { timeline in
            let cycle = timeline.date.timeIntervalSinceReferenceDate.truncatingRemainder(dividingBy: 1.1) / 1.1
            VStack(spacing: 5) {
                ForEach(0 ..< dots, id: \.self) { index in
                    let wave = (cycle - Double(index) / Double(dots) + 1).truncatingRemainder(dividingBy: 1)
                    Circle()
                        .fill(live ? palette.primary : palette.inkMuted)
                        .frame(width: 5, height: 5)
                        .opacity(0.15 + 0.85 * (1 - wave) * (1 - wave))
                }
            }
        }
        .padding(.vertical, SpaceTokens.xs)
        .accessibilityHidden(true)
    }
}

private struct LaunchButton: View {
    @Environment(\.palette) private var palette
    @State private var launch = Launch.idle
    let chosen: Int
    let onLaunch: () -> Void
    let onDone: () -> Void

    private var label: String {
        switch launch {
        case .sent: "Sent"
        case .sending: "Sending"
        case .idle, .flying: chosen > 1 ? "Send to \(chosen) devices" : "Send"
        }
    }

    var body: some View {
        Button(action: start) {
            HStack(spacing: SpaceTokens.sm) {
                switch launch {
                case .sent:
                    Icon(.check, tint: palette.onPrimary, size: 22).transition(.scale.combined(with: .opacity))
                case .sending:
                    ProgressView().tint(palette.onPrimary).transition(.scale.combined(with: .opacity))
                case .idle, .flying:
                    Icon(.cursor, tint: palette.onPrimary, size: 22)
                        .rotationEffect(.degrees(90))
                        .offset(x: launch == .flying ? 36 : 0, y: launch == .flying ? -36 : 0)
                        .opacity(launch == .flying ? 0 : 1)
                }
                Text(label).font(.launchText).foregroundStyle(palette.onPrimary).contentTransition(.opacity)
            }
            .frame(width: launch == .sent ? 148 : 210, height: 56)
            .background(launch == .sent ? palette.successFill : palette.primaryFill, in: RoundedRectangle(cornerRadius: RadiusTokens.field, style: .continuous))
            .clipShape(RoundedRectangle(cornerRadius: RadiusTokens.field, style: .continuous))
        }
        .buttonStyle(.plain)
        .disabled(chosen == 0 || launch != .idle)
        .opacity(chosen > 0 || launch == .sent ? 1 : 0.5)
        .sensoryFeedback(.impact(weight: .medium), trigger: launch == .flying)
        .sensoryFeedback(.success, trigger: launch == .sent)
    }

    private func start() {
        onLaunch()
        withAnimation(.easeIn(duration: 0.36)) { launch = .flying }
        Task {
            try? await Task.sleep(for: .milliseconds(380))
            withAnimation(.snappy(duration: 0.25)) { launch = .sending }
            try? await Task.sleep(for: .milliseconds(900))
            withAnimation(.snappy(duration: 0.3)) { launch = .sent }
            try? await Task.sleep(for: .milliseconds(1000))
            onDone()
        }
    }
}

private struct TargetRow: View {
    @Environment(\.palette) private var palette
    let target: Target
    let chosen: Bool
    let onToggle: () -> Void

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            if chosen {
                Tile(glyph: .check, fill: palette.primaryFill, tone: palette.onPrimary)
            } else {
                DeviceTile(identity: target.identity, online: target.detail != "Offline" && target.detail != "Paired")
            }
            VStack(alignment: .leading, spacing: 0) {
                Text(target.name).lineLimit(1)
                StatusLine(target.detail, dot: target.detail == "Online" ? palette.successFill : nil)
            }
            Spacer(minLength: 0)
        }
        .padding(SpaceTokens.sm)
        .background(chosen ? palette.raised : .clear, in: controlShape)
        .contentShape(controlShape)
        .onTapGesture(perform: onToggle)
        .accessibilityAddTraits(chosen ? [.isButton, .isSelected] : .isButton)
        .sensoryFeedback(.selection, trigger: chosen)
    }
}
