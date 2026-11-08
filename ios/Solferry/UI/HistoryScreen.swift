import SwiftUI

struct HistoryScreen: View {
    @Environment(Engine.self) private var engine

    var body: some View {
        let history = engine.state.history
        Page(title: "History") {
            if history.isEmpty {
                Card(glyph: .history, title: "History") {
                    PaddedWell {
                        VStack(spacing: 2) {
                            Text("No transfers yet")
                            Muted("Files you send or receive are listed here.")
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, SpaceTokens.lg)
                    }
                }
            } else {
                Card(glyph: .history, title: "History") {
                    ActionButton(label: "Clear", kind: .ghost, scale: .small) { engine.bridge.clearHistory() }
                } content: {
                    Well { ForEach(Array(history.enumerated()), id: \.offset) { Past(entry: $0.element) } }
                }
            }
        }
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
