import SwiftUI

struct AddDeviceScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.dismiss) private var dismiss
    @Environment(\.palette) private var palette
    @State private var code = ""

    var body: some View {
        let state = engine.state
        SheetPage(title: "Pair with a code", close: { dismiss() }) {
            Card(glyph: .qr, title: "Show a code") {
                PaddedWell {
                    if let invite = state.invite {
                        Invite(code: invite)
                    } else {
                        Muted("For a device on another network, or when this one can't be found nearby. Show a code here and enter it there.")
                        ActionButton(label: "Show code", glyph: .qr, kind: .primary, wide: true) { engine.bridge.invite() }
                    }
                }
            }
            Card(glyph: .key, title: "Enter a code") {
                PaddedWell {
                    HStack(spacing: SpaceTokens.sm) {
                        Field(text: $code, placeholder: "Paste a sf1 code", onSubmit: join) { PasteInto(text: $code) }
                        ActionButton(label: state.joining ? "Pairing" : "Pair", kind: .primary, action: join)
                            .disabled(state.joining)
                    }
                    if let error = state.joinError { Muted(error, color: palette.danger) }
                }
            }
            AddressCard()
        }
        .onDisappear { engine.clearPairingErrors() }
    }

    private func join() {
        let trimmed = code.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty { engine.join(trimmed) }
    }
}

/// For a device on another subnet, or on a network that keeps devices from finding each other.
private struct AddressCard: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @State private var address = ""
    @State private var own = lanAddresses()

    var body: some View {
        let state = engine.state
        Card(glyph: .link, title: "Enter an IP address") {
            PaddedWell {
                HStack(spacing: SpaceTokens.sm) {
                    Field(text: $address, placeholder: "192.168.1.20", onSubmit: pair)
                        .keyboardType(.numbersAndPunctuation)
                    ActionButton(label: state.pairingAddress ? "Pairing" : "Pair", kind: .primary, action: pair)
                        .disabled(state.pairingAddress)
                }
                if let error = state.addressError { Muted(error, color: palette.danger) }
                if !own.isEmpty { Muted("This device is at \(own.joined(separator: ", ")).") }
            }
        }
    }

    private func pair() {
        let trimmed = address.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty { engine.pairAddress(trimmed) }
    }
}

private struct Invite: View {
    @Environment(Engine.self) private var engine
    @State private var copied = false
    let code: String

    var body: some View {
        VStack(spacing: SpaceTokens.sm) {
            QrCode(text: code)
            Muted("Works once, for 10 minutes.")
            ActionButton(label: copied ? "Copied" : "Copy code", glyph: copied ? .check : .copy, scale: .small) {
                engine.copy(code)
                copied = true
            }
        }
        .frame(maxWidth: .infinity)
    }
}

/// Draws the code on a white plate so it scans in both themes.
struct QrCode: View {
    let text: String

    var body: some View {
        if let matrix = qrCode(text: text) {
            Canvas { context, size in
                let width = Int(matrix.width)
                let cell = size.width / CGFloat(width)
                for (index, dark) in matrix.dark.enumerated() where dark {
                    let box = CGRect(x: CGFloat(index % width) * cell, y: CGFloat(index / width) * cell, width: cell + 0.5, height: cell + 0.5)
                    context.fill(Path(box), with: .color(.black))
                }
            }
            .frame(width: 200, height: 200)
            .padding(16)
            .background(.white, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
            .accessibilityLabel("QR code")
        }
    }
}
