import SwiftUI

/// Light, dark or whatever the system uses, kept on this device.
enum Appearance: String, CaseIterable {
    case system, light, dark

    static let key = "appearance"

    var scheme: ColorScheme? {
        switch self {
        case .system: nil
        case .light: .light
        case .dark: .dark
        }
    }

    var label: LocalizedStringKey {
        switch self {
        case .system: "System"
        case .light: "Light"
        case .dark: "Dark"
        }
    }
}

struct AppearanceCard: View {
    @AppStorage(Appearance.key) private var chosen = Appearance.system

    var body: some View {
        Card(glyph: .appearance, title: "Appearance") {
            PaddedWell {
                HStack(spacing: SpaceTokens.xs) {
                    ForEach(Appearance.allCases, id: \.self) { appearance in
                        ActionButton(label: appearance.label, kind: chosen == appearance ? .secondary : .ghost, scale: .small) {
                            chosen = appearance
                        }
                    }
                }
            }
        }
    }
}
