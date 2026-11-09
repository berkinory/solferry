import SwiftUI

private struct Digits: View {
    @Environment(\.palette) private var palette
    let code: String

    var body: some View {
        let digits = Array(code.filter(\.isNumber))
        HStack(spacing: 4) {
            ForEach(digits.indices, id: \.self) { index in
                if index == digits.count / 2 { Spacer().frame(width: 10) }
                Text(String(digits[index]))
                    .font(.digitText)
                    .frame(width: 38, height: 48)
                    .background(palette.page, in: controlShape)
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, SpaceTokens.xs)
        .accessibilityElement()
        .accessibilityLabel(code)
    }
}

private struct Asker: View {
    let peer: Identity
    let title: String
    let detail: String

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            DeviceTile(identity: peer)
            VStack(alignment: .leading, spacing: 0) {
                Text(title).lineLimit(1)
                Caption(detail)
            }
        }
    }
}

/// Pairing takes over the screen the way sending does.
struct PairingScreen: View {
    @Environment(Engine.self) private var engine

    var body: some View {
        let state = engine.state
        if let asked = state.pairings.first {
            SheetPage(title: "Pairing request", close: engine.closePairing) { RequestBody(prompt: asked) }
        } else if let code = state.pairingCode {
            SheetPage(title: "Compare codes", close: engine.closePairing) { CompareBody(code: code) }
        }
    }
}

private struct RequestBody: View {
    @Environment(Engine.self) private var engine
    let prompt: PairingPrompt

    var body: some View {
        VStack(spacing: SpaceTokens.xs) {
            DeviceTile(identity: prompt.peer)
            Text("\(prompt.name) wants to pair").lineLimit(1)
            Caption(prompt.peer.fingerprint)
        }
        CardFrame { Well(padding: 12) { Digits(code: prompt.code) } }
        Muted("Accept only if the other screen shows the same code.").multilineTextAlignment(.center)
        HStack(spacing: SpaceTokens.sm) {
            ActionButton(label: "Decline") { engine.answerPairing(prompt.key, accept: false) }
            ActionButton(label: "Same code", glyph: .check, kind: .primary, wide: true) { engine.answerPairing(prompt.key, accept: true) }
        }
        .padding(.top, SpaceTokens.sm)
    }
}

private struct CompareBody: View {
    let code: String

    var body: some View {
        CardFrame { Well(padding: 12) { Digits(code: code) } }
        Muted("Check that the other device shows this code, then accept there.").multilineTextAlignment(.center)
    }
}

struct OfferCard: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @State private var blocking = false
    let incoming: Incoming

    var body: some View {
        let files = incoming.files == 1 ? "1 file" : "\(incoming.files) files"
        Card(glyph: .download, title: "Incoming files", tone: palette.primary) {
            PaddedWell {
                Asker(
                    peer: incoming.sender,
                    title: "\(incoming.name) wants to send",
                    detail: "\(files), \(bytes(incoming.bytes))  ·  \(incoming.sender.fingerprint)"
                )
                Text(incoming.firstPath).lineLimit(1).truncationMode(.middle).foregroundStyle(palette.inkMuted)
                if !incoming.paired { Muted("Not paired. Accept only if you expected this.", color: palette.warning) }
                ViewThatFits(in: .horizontal) {
                    answers
                    VStack(spacing: SpaceTokens.sm) { answers }
                }
            }
        }
        .confirm(
            "Block \(incoming.name)?",
            isPresented: $blocking,
            message: "It can't send files or ask to pair. You can unblock it later.",
            action: "Block"
        ) { engine.blockOffer(incoming.key) }
    }

    private var answers: some View {
        HStack(spacing: SpaceTokens.sm) {
            ActionButton(label: "Accept", kind: .primary, wide: true) { engine.answerOffer(incoming.key, accept: true) }
            if incoming.paired { ActionButton(label: "Always accept") { engine.acceptAlways(incoming.key) } }
            ActionButton(label: "Decline") { engine.answerOffer(incoming.key, accept: false) }
            if !incoming.paired { ActionButton(label: "Block", kind: .ghost) { blocking = true } }
        }
    }
}

struct TextCard: View {
    @Environment(Engine.self) private var engine
    let shared: ReceivedText

    var body: some View {
        Card(glyph: .clipboard, title: "Text from \(shared.name)") {
            PaddedWell {
                Text(shared.text).lineLimit(4).textSelection(.enabled)
                HStack(spacing: SpaceTokens.sm) {
                    ActionButton(label: shared.copied ? "Copied" : "Copy", glyph: shared.copied ? .check : .copy, scale: .small) {
                        engine.copy(shared.text)
                        engine.markCopied(shared.id)
                    }
                    ActionButton(label: "Dismiss", kind: .ghost, scale: .small) { engine.dismissText(shared.id) }
                }
            }
        }
    }
}
