import SwiftUI

/// The first screen on a fresh install: name this device, see what iOS allows, then go home.
struct OnboardingScreen: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @State private var name = ""
    @State private var height = 0.0
    let done: () -> Void

    var body: some View {
        let current = engine.state.me?.name
        let trimmed = name.trimmingCharacters(in: .whitespaces)
        ScrollView {
            VStack(spacing: SpaceTokens.sm) {
                Icon(.mark, tint: palette.brand, size: 56)
                Text("Welcome to Solferry").font(.launchText).multilineTextAlignment(.center)
                Muted("Send files between your devices. No account, nothing in the cloud.")
                    .multilineTextAlignment(.center)
                    .padding(.bottom, SpaceTokens.md)
                Card(glyph: .computer, title: "Your device's name") {
                    PaddedWell {
                        Field(text: $name, placeholder: "Name") { start(trimmed, current) }
                        Muted("Nearby devices see this name.")
                    }
                }
                PermissionsCard()
                ActionButton(label: "Get started", kind: .primary, wide: true) { start(trimmed, current) }
                    .disabled(trimmed.isEmpty)
                    .padding(.top, SpaceTokens.xs)
            }
            .padding(.horizontal, SpaceTokens.md)
            .padding(.vertical, SpaceTokens.xl)
            .frame(minHeight: height)
        }
        .onGeometryChange(for: Double.self) { $0.size.height } action: { height = $0 }
        .scrollBounceBehavior(.basedOnSize)
        .scrollDismissesKeyboard(.interactively)
        .background(palette.page.ignoresSafeArea())
        .onChange(of: current, initial: true) { _, saved in if name.isEmpty { name = saved ?? "" } }
    }

    private func start(_ trimmed: String, _ current: String?) {
        guard !trimmed.isEmpty else { return }
        if trimmed != current { engine.bridge.rename(name: trimmed) }
        done()
    }
}
