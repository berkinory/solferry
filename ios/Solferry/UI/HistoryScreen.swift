import QuickLook
import SwiftUI

struct HistoryScreen: View {
    @Environment(Engine.self) private var engine
    @State private var preview: URL?
    /// A file in a picked folder is only readable while the app holds access to it, so the preview keeps it open.
    @State private var scoped: URL?

    var body: some View {
        let history = engine.state.history
        if history.isEmpty {
            NoHistory()
        } else {
            Page(title: "History") {
                Card(glyph: .history, title: "History") {
                    ActionButton(label: "Clear", kind: .ghost, scale: .small) { engine.bridge.clearHistory() }
                } content: {
                    Well { ForEach(Array(history.enumerated()), id: \.offset) { Past(entry: $0.element, onOpen: opener(for: $0.element)) } }
                }
            }
            .quickLookPreview($preview)
            .onChange(of: preview) { _, shown in
                if shown == nil { release() }
            }
        }
    }

    /// Only received files can be opened: a phone sends what apps hand it, which has no place of its own to go back to.
    private func opener(for entry: HistoryEntry) -> (() -> Void)? {
        guard !entry.sending, entry.completed, entry.path != nil else { return nil }
        return {
            guard let url = engine.savedFile(of: entry) else { return }
            release()
            if url.startAccessingSecurityScopedResource() { scoped = url }
            preview = url
        }
    }

    private func release() {
        scoped?.stopAccessingSecurityScopedResource()
        scoped = nil
    }
}

private struct NoHistory: View {
    @Environment(\.palette) private var palette

    var body: some View {
        VStack(spacing: SpaceTokens.xs) {
            Icon(.history, tint: palette.inkMuted, size: 24)
                .frame(width: 56, height: 56)
                .background(palette.control, in: fieldShape)
                .overlay(fieldShape.strokeBorder(palette.line))
                .padding(.bottom, SpaceTokens.md)
            Text("No transfers yet").font(.titleText)
            Muted("Files you send or receive show up here.")
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(palette.page.ignoresSafeArea())
        .navigationTitle("History")
        .navigationBarTitleDisplayMode(.inline)
    }
}

private struct Past: View {
    @Environment(\.palette) private var palette
    let entry: HistoryEntry
    let onOpen: (() -> Void)?

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            Tile(glyph: entry.sending ? .upload : .download, fill: palette.control, tone: palette.inkMuted)
            VStack(alignment: .leading, spacing: 0) {
                HStack(spacing: SpaceTokens.sm) {
                    Text(entry.label).lineLimit(1)
                    Spacer(minLength: 0)
                    Muted(verbatim: entry.completed ? bytes(entry.bytes) : String(localized: "Failed"), color: entry.completed ? palette.inkMuted : palette.danger).fixedSize()
                }
                HStack(spacing: SpaceTokens.xs) {
                    Icon(platformGlyph(entry.platform), tint: palette.inkMuted, size: 14)
                    Muted(verbatim: entry.name).lineLimit(1)
                    Spacer(minLength: 0)
                    Caption(ago(entry.finishedMs)).fixedSize()
                }
            }
        }
        .padding(SpaceTokens.sm)
        .contentShape(Rectangle())
        .onTapGesture { onOpen?() }
        .accessibilityAddTraits(onOpen == nil ? [] : .isButton)
    }
}
