import SwiftUI

struct HomeScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @Binding var staged: [URL]
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
                if !staged.isEmpty {
                    StagedCard(count: staged.count) { staged = [] }
                } else if !state.devices.isEmpty {
                    SendZone { pick(nil) }
                }
                if state.devices.isEmpty, state.strangers.isEmpty { RadarHero() }
                if !state.devices.isEmpty {
                    DeviceCard(state: state, picking: !staged.isEmpty) { device in
                        if staged.isEmpty {
                            pick(device)
                        } else {
                            engine.send(staged, to: device.identity.id)
                            staged = []
                        }
                    }
                }
                if !state.devices.isEmpty || !state.strangers.isEmpty { NearbyCard(nearby: state.strangers) }
                if !state.transfers.isEmpty { TransferCard(transfers: state.transfers) }
            }
            .padding(.horizontal, SpaceTokens.md)
            .padding(.vertical, SpaceTokens.xs)
        }
        .background(palette.page.ignoresSafeArea())
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                HStack(spacing: 6) {
                    Text("Solferry").font(.titleText)
                    Muted(state.me?.name ?? "").lineLimit(1)
                }
            }
            ToolbarItem(placement: .topBarTrailing) {
                NavigationLink(value: Route.settings) { Icon(.settings, size: 20) }
                    .accessibilityLabel("Settings")
            }
        }
    }
}

private struct SendZone: View {
    @Environment(\.palette) private var palette
    let action: () -> Void

    var body: some View {
        CardFrame {
            Well(padding: 14, action: action) {
                HStack(spacing: SpaceTokens.md) {
                    Icon(.send, tint: palette.onPrimary, size: 20)
                        .frame(width: 44, height: 44)
                        .background(palette.primaryFill, in: controlShape)
                        .overlay(controlShape.strokeBorder(palette.primaryBorder))
                    VStack(alignment: .leading, spacing: 0) {
                        Text("Send files")
                        Muted("Choose what to send, then tap a device")
                    }
                }
            }
        }
    }
}

private struct StagedCard: View {
    let count: Int
    let onClear: () -> Void

    var body: some View {
        Card(glyph: .upload, title: "Ready to send") {
            Count(value: count)
        } content: {
            PaddedWell {
                HStack {
                    Muted(count == 1 ? "Tap a device to send this file." : "Tap a device to send these files.")
                    Spacer()
                    ActionButton(label: "Cancel", kind: .ghost, scale: .small, action: onClear)
                }
            }
        }
    }
}
