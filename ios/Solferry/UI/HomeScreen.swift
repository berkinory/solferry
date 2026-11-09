import SwiftUI

struct HomeScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let pick: (PairedDevice?) -> Void

    var body: some View {
        let state = engine.state
        ScrollView {
            VStack(spacing: SpaceTokens.sm) {
                if let problem = state.startProblem { ProblemCard(problem: problem) }
                ForEach(state.problems) { item in
                    ProblemCard(problem: item.problem, about: item.name) { engine.dismissProblem(item.id) }
                }
                ForEach(state.pairings) { PairingCard(prompt: $0) }
                if let code = state.pairingCode { CompareCard(code: code) }
                ForEach(state.offers, id: \.key) { OfferCard(incoming: $0) }
                ForEach(state.texts) { TextCard(shared: $0) }
                SendZone { pick(nil) }
                if !state.devices.isEmpty { DeviceCard(state: state) { pick($0) } }
                NearbyCard(nearby: state.strangers, alone: state.devices.isEmpty)
                if !state.transfers.isEmpty { TransferCard(transfers: state.transfers) }
            }
            .padding(.horizontal, SpaceTokens.md)
            .padding(.vertical, SpaceTokens.xs)
        }
        .refreshable {
            engine.bridge.checkPresence()
            try? await Task.sleep(for: .milliseconds(700))
        }
        .background(palette.page.ignoresSafeArea())
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) { Brand() }
                .sharedBackgroundVisibility(.hidden)
        }
    }
}

private struct Brand: View {
    @Environment(\.palette) private var palette

    var body: some View {
        HStack(spacing: 6) {
            Icon(.mark, tint: palette.brand, size: 20)
            Text("Solferry").font(.titleText)
        }
    }
}

private struct SendZone: View {
    @Environment(\.palette) private var palette
    let action: () -> Void

    var body: some View {
        CardFrame {
            Well(padding: 14, action: action) {
                VStack(spacing: SpaceTokens.sm) {
                    ZStack {
                        FanTile(glyph: .typeImage, x: -38)
                        FanTile(glyph: .typePdf, x: 38)
                        Icon(.send, tint: palette.onPrimary, size: 24)
                            .frame(width: 56, height: 56)
                            .glassEffect(.regular.tint(palette.primaryFill), in: fieldShape)
                    }
                    .frame(height: 64)
                    VStack(spacing: 0) {
                        Text("Send files").font(.titleText)
                        Muted("Tap to choose what to send")
                    }
                }
                .frame(maxWidth: .infinity)
                .frame(height: 120)
            }
        }
    }
}

private struct FanTile: View {
    @Environment(\.palette) private var palette
    let glyph: Glyph
    let x: Double

    var body: some View {
        Icon(glyph, tint: palette.inkMuted, size: 22)
            .frame(width: 56, height: 56)
            .background(palette.control, in: fieldShape)
            .overlay(fieldShape.strokeBorder(palette.line))
            .rotationEffect(.degrees(x < 0 ? -11 : 11))
            .offset(x: x)
    }
}
