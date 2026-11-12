import SwiftUI

/// A link for a browser on the same network. Closing the sheet stops it.
struct LinkScreen: View {
    @Environment(Engine.self) private var engine
    @State private var copied = false
    let link: LinkState

    var body: some View {
        let url = link.urls.first ?? ""
        SheetPage(title: link.receiving ? "Receive via link" : "Share via link", close: { engine.bridge.stopLink() }) {
            VStack(spacing: SpaceTokens.sm) {
                QrCode(text: url)
                Text(url).font(.mutedText).multilineTextAlignment(.center).textSelection(.enabled)
                Muted(link.receiving
                    ? "Open it in a browser on the same network to send files or text here."
                    : "Open it in a browser on the same network to download the files.")
                    .multilineTextAlignment(.center)
                ActionButton(label: copied ? "Copied" : "Copy link", glyph: copied ? .check : .copy, scale: .small) {
                    engine.copy(url)
                    copied = true
                }
            }
            .frame(maxWidth: .infinity)
            if !link.received.isEmpty || !link.texts.isEmpty {
                Card(glyph: .download, title: "Received") {
                    Well {
                        ForEach(Array(link.texts.enumerated()), id: \.offset) { _, text in
                            HStack(alignment: .top, spacing: SpaceTokens.sm) {
                                Text(verbatim: text).lineLimit(3).frame(maxWidth: .infinity, alignment: .leading)
                                ActionButton(label: "Copy", glyph: .copy, scale: .small) { engine.copy(text) }
                            }
                            .padding(SpaceTokens.xs)
                        }
                        ForEach(Array(link.received.enumerated()), id: \.offset) { Muted(verbatim: $0.element).padding(SpaceTokens.xs) }
                    }
                }
            }
            ActionButton(label: "Stop", kind: .danger) { engine.bridge.stopLink() }
        }
    }
}

struct BrowserCard: View {
    @Environment(Engine.self) private var engine

    var body: some View {
        Card(glyph: .link, title: "No app on the other device") {
            PaddedWell {
                Muted("A browser on the same network can send you files. To send files to one, pick them and choose Share via link.")
                ActionButton(label: "Receive via link", glyph: .download, wide: true) { engine.bridge.receiveLink() }
            }
        }
    }
}
