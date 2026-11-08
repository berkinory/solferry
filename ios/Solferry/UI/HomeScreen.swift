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
                if state.devices.isEmpty, state.strangers.isEmpty { RadarHero() }
                if !state.devices.isEmpty { DeviceCard(state: state) { pick($0) } }
                if !state.devices.isEmpty || !state.strangers.isEmpty { NearbyCard(nearby: state.strangers) }
                if !state.transfers.isEmpty { TransferCard(transfers: state.transfers) }
            }
            .padding(.horizontal, SpaceTokens.md)
            .padding(.vertical, SpaceTokens.xs)
        }
        .background(palette.page.ignoresSafeArea())
        .toolbar(.hidden, for: .navigationBar)
        .safeAreaInset(edge: .top, spacing: 0) { HomeBar() }
    }
}

private struct HomeBar: View {
    @Environment(\.palette) private var palette

    var body: some View {
        HStack(spacing: 6) {
            Icon(.mark, tint: palette.brand, size: 20)
            Text("Solferry").font(.titleText)
        }
        .frame(maxWidth: .infinity)
        .frame(height: 44)
        .background(palette.page)
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
                        FanTile(glyph: .typeImage, x: -46)
                        FanTile(glyph: .typePdf, x: 46)
                        Icon(.send, tint: palette.onPrimary, size: 24)
                            .frame(width: 56, height: 56)
                            .background(palette.primaryFill, in: RoundedRectangle(cornerRadius: RadiusTokens.field, style: .continuous))
                            .overlay(RoundedRectangle(cornerRadius: RadiusTokens.field, style: .continuous).strokeBorder(palette.primaryBorder))
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
        Icon(glyph, tint: palette.inkMuted, size: 20)
            .frame(width: 44, height: 44)
            .background(palette.control, in: controlShape)
            .overlay(controlShape.strokeBorder(palette.line))
            .offset(x: x, y: 6)
    }
}
