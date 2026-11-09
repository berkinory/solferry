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
            ZStack {
                FanTile(glyph: .upload, x: -38)
                FanTile(glyph: .download, x: 38)
                Icon(.history, tint: palette.ink, size: 24)
                    .frame(width: 56, height: 56)
                    .background(palette.raised, in: fieldShape)
                    .overlay(fieldShape.strokeBorder(palette.line))
            }
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
                Text(entry.label).lineLimit(1)
                Muted(
                    "\(entry.sending ? "To" : "From") \(entry.name)  ·  \(entry.completed ? bytes(entry.bytes) : "Failed")  ·  \(ago(entry.finishedMs))",
                    color: entry.completed ? nil : palette.danger
                )
                .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(SpaceTokens.sm)
    }
}
