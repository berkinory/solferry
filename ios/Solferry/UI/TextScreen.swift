import SwiftUI

/// Text to send to one device. iOS hands over the clipboard only through its paste control.
struct TextScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @Environment(\.dismiss) private var dismiss
    @FocusState private var focused: Bool
    @State private var text = ""
    let device: PairedDevice

    var body: some View {
        SheetPage(title: "Send text", close: { dismiss() }) {
            HStack {
                Muted("To \(device.name)")
                Spacer()
                ClipboardButton { text = $0 }
            }
            TextEditor(text: $text)
                .focused($focused)
                .scrollContentBackground(.hidden)
                .padding(SpaceTokens.xs)
                .frame(minHeight: 120, maxHeight: 280)
                .background(palette.page, in: controlShape)
                .overlay(controlShape.strokeBorder(focused ? palette.primary : palette.line))
            ActionButton(label: "Send", glyph: .send, kind: .primary, wide: true) {
                engine.bridge.sendText(peer: device.identity.id, text: text)
                dismiss()
            }
            .disabled(text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
        }
    }
}
