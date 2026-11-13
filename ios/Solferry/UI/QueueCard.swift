import SwiftUI

/// Sends waiting for their devices, each with a way to take it back.
struct QueueCard: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let queue: [QueueEntry]

    var body: some View {
        Card(glyph: .send, title: "Waiting to send") {
            Count(value: queue.count)
        } content: {
            Well {
                ForEach(queue, id: \.id) { entry in
                    HStack(spacing: SpaceTokens.md) {
                        Tile(glyph: fileGlyph(entry.label), fill: palette.control, tone: palette.inkMuted)
                        VStack(alignment: .leading, spacing: 0) {
                            Text(verbatim: entry.label).lineLimit(1)
                            Muted("To \(entry.name)")
                        }
                        Spacer(minLength: 0)
                        ActionButton(glyph: .cancel, kind: .ghost, scale: .small, description: "Cancel send") {
                            engine.bridge.unqueue(id: entry.id)
                        }
                    }
                    .padding(SpaceTokens.sm)
                }
                Caption(String(localized: "Sent once the device is back while Solferry is running. Dropped after \(queueDays()) days."))
                    .lineLimit(nil)
                    .padding(.horizontal, SpaceTokens.sm)
                    .padding(.vertical, SpaceTokens.xs)
            }
        }
    }
}
