import SwiftUI

private struct Target: Identifiable {
    let identity: Identity
    let name: String
    let detail: String

    var id: String { identity.id }
}

/// Everything about one send on its own screen: what goes, and who gets it.
struct SendScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @Environment(\.dismiss) private var dismiss
    @State private var chosen: Set<String> = []
    @State private var browsing = false
    @Binding var files: [URL]

    var body: some View {
        NavigationStack {
            Page(title: "Send") {
                Card(glyph: .file, title: "Files") {
                    ActionButton(label: "Add", glyph: .add, kind: .ghost, scale: .small) { browsing = true }
                } content: {
                    Well(spacing: 0) {
                        ForEach(files, id: \.self) { file in
                            FileRow(file: file) {
                                files.removeAll { $0 == file }
                                if files.isEmpty { dismiss() }
                            }
                        }
                    }
                }
                Card(glyph: .devices, title: "Send to") {
                    Well {
                        if targets.isEmpty { Scanning() }
                        ForEach(targets) { target in
                            TargetRow(target: target, chosen: chosen.contains(target.id)) {
                                if chosen.remove(target.id) == nil { chosen.insert(target.id) }
                            }
                        }
                    }
                }
                ActionButton(label: chosen.count > 1 ? "Send to \(chosen.count) devices" : "Send", glyph: .send, kind: .primary, wide: true) {
                    engine.send(files, to: Array(chosen))
                    files = []
                    dismiss()
                }
                .disabled(chosen.isEmpty)
            }
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button { files = []; dismiss() } label: { Icon(.cancel, size: 20) }
                        .accessibilityLabel("Cancel")
                }
            }
            .fileImporter(isPresented: $browsing, allowedContentTypes: [.item], allowsMultipleSelection: true) { result in
                if case let .success(urls) = result { files.append(contentsOf: urls.filter { !files.contains($0) }) }
            }
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
        let others = state.strangers.map { Target(identity: $0.identity, name: $0.name ?? "Unnamed device", detail: $0.overlay ? "Overlay, not paired" : "Not paired") }
        return mine + others
    }

    private func detail(near: Nearby?, online: Bool?) -> String {
        if let near { return near.overlay ? "On your overlay network" : "On this network" }
        guard let online else { return "Paired" }
        return online ? "Online" : "Offline"
    }
}

private struct FileRow: View {
    @Environment(\.palette) private var palette
    let file: URL
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: SpaceTokens.sm) {
            Icon(.file, tint: palette.inkMuted, size: 18)
            Text(file.lastPathComponent).lineLimit(1).truncationMode(.middle)
            Spacer(minLength: 0)
            if let size { Mono(bytes(size)) }
            ActionButton(glyph: .cancel, kind: .ghost, scale: .small, description: "Remove \(file.lastPathComponent)", action: onRemove)
        }
        .padding(.leading, SpaceTokens.sm)
        .frame(minHeight: 44)
    }

    private var size: UInt64? {
        let scoped = file.startAccessingSecurityScopedResource()
        defer { if scoped { file.stopAccessingSecurityScopedResource() } }
        return (try? file.resourceValues(forKeys: [.fileSizeKey]).fileSize).flatMap { $0 }.map(UInt64.init)
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
                DeviceTile(identity: target.identity)
            }
            VStack(alignment: .leading, spacing: 0) {
                Text(target.name).lineLimit(1)
                StatusLine(target.detail)
            }
            Spacer(minLength: 0)
        }
        .padding(SpaceTokens.sm)
        .background(chosen ? palette.raised : .clear, in: controlShape)
        .contentShape(controlShape)
        .onTapGesture(perform: onToggle)
        .accessibilityAddTraits(chosen ? [.isButton, .isSelected] : .isButton)
    }
}
