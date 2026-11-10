import SwiftUI

struct HistoryScreen: View {
    @Environment(Engine.self) private var engine

    var body: some View {
        let history = engine.state.history
        if history.isEmpty {
            NoHistory()
        } else {
            Page(title: "History") {
                Card(glyph: .history, title: "History") {
                    ActionButton(label: "Clear", kind: .ghost, scale: .small) { engine.bridge.clearHistory() }
                } content: {
                    Well { ForEach(Array(history.enumerated()), id: \.offset) { Past(entry: $0.element) } }
                }
            }
        }
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

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            Tile(glyph: entry.sending ? .upload : .download, fill: palette.control, tone: palette.inkMuted)
            VStack(alignment: .leading, spacing: 0) {
                HStack(spacing: SpaceTokens.sm) {
                    Text(entry.label).lineLimit(1)
                    Spacer(minLength: 0)
                    Muted(entry.completed ? bytes(entry.bytes) : "Failed", color: entry.completed ? palette.inkMuted : palette.danger).fixedSize()
                }
                HStack(spacing: SpaceTokens.xs) {
                    Icon(platformGlyph(entry.platform), tint: palette.inkMuted, size: 14)
                    Muted(entry.name).lineLimit(1)
                    Spacer(minLength: 0)
                    Caption(ago(entry.finishedMs)).fixedSize()
                }
            }
        }
        .padding(SpaceTokens.sm)
    }
}
