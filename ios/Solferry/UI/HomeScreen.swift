import SwiftUI

struct HomeScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let pick: (PairedDevice?) -> Void
    let text: (PairedDevice) -> Void

    var body: some View {
        let state = engine.state
        ScrollView {
            VStack(spacing: SpaceTokens.sm) {
                ForEach(state.offers, id: \.key) { OfferCard(incoming: $0) }
                if let shared = state.text { TextCard(shared: shared) }
                SendZone { pick(nil) }
                if !state.queue.isEmpty { QueueCard(queue: state.queue) }
                if !state.devices.isEmpty { DeviceCard(state: state, onPick: { pick($0) }, onText: text) }
                NearbyCard(nearby: state.strangers, alone: state.devices.isEmpty)
                BrowserCard()
            }
            .padding(.horizontal, SpaceTokens.md)
            .padding(.vertical, SpaceTokens.xs)
        }
        .scrollIndicators(.hidden)
        .refreshable {
            engine.bridge.checkPresence()
            try? await Task.sleep(for: .milliseconds(700))
        }
        .background(palette.page.ignoresSafeArea())
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            if #available(iOS 26, *) {
                ToolbarItem(placement: .principal) { Brand() }
                    .sharedBackgroundVisibility(.hidden)
            } else {
                ToolbarItem(placement: .principal) { Brand() }
            }
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
                            .glassFill(palette.primaryFill, in: fieldShape)
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
