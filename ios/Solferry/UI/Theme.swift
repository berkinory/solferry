import SwiftUI

/// The design tokens the app reads directly.
struct Palette {
    let night: Bool
    let page, well, raised, control, line, ink, inkMuted: Color
    let primary, brand, primaryFill, primaryHover, primaryBorder, primaryDeep, onPrimary: Color
    let success, successFill, platformApple, platformAndroid, platformWindows, platformLinux, warning, danger, dangerFill, dangerWell: Color
    let routeLocal, routeOverlay, routeDirect, routeRelay: Color
    let identity: [Color]

    func route(_ route: TransferRoute) -> Color {
        switch route {
        case .local: routeLocal
        case .overlay: routeOverlay
        case .direct: routeDirect
        case .relay: routeRelay
        }
    }

    static let light = Palette(
        night: false,
        page: LightColors.page, well: LightColors.well, raised: LightColors.raised, control: LightColors.control,
        line: LightColors.line, ink: LightColors.ink, inkMuted: LightColors.inkMuted,
        primary: LightColors.primary, brand: LightColors.brand, primaryFill: LightColors.primaryFill, primaryHover: LightColors.primaryHover,
        primaryBorder: LightColors.primaryBorder, primaryDeep: LightColors.primaryDeep, onPrimary: LightColors.onPrimary,
        success: LightColors.success, successFill: LightColors.successFill, platformApple: LightColors.platformApple, platformAndroid: LightColors.platformAndroid, platformWindows: LightColors.platformWindows, platformLinux: LightColors.platformLinux, warning: LightColors.warning, danger: LightColors.danger,
        dangerFill: LightColors.dangerFill, dangerWell: LightColors.dangerWell,
        routeLocal: LightColors.routeLocal, routeOverlay: LightColors.routeOverlay,
        routeDirect: LightColors.routeDirect, routeRelay: LightColors.routeRelay,
        identity: IdentityColors.light
    )

    static let dark = Palette(
        night: true,
        page: DarkColors.page, well: DarkColors.well, raised: DarkColors.raised, control: DarkColors.control,
        line: DarkColors.line, ink: DarkColors.ink, inkMuted: DarkColors.inkMuted,
        primary: DarkColors.primary, brand: DarkColors.brand, primaryFill: DarkColors.primaryFill, primaryHover: DarkColors.primaryHover,
        primaryBorder: DarkColors.primaryBorder, primaryDeep: DarkColors.primaryDeep, onPrimary: DarkColors.onPrimary,
        success: DarkColors.success, successFill: DarkColors.successFill, platformApple: DarkColors.platformApple, platformAndroid: DarkColors.platformAndroid, platformWindows: DarkColors.platformWindows, platformLinux: DarkColors.platformLinux, warning: DarkColors.warning, danger: DarkColors.danger,
        dangerFill: DarkColors.dangerFill, dangerWell: DarkColors.dangerWell,
        routeLocal: DarkColors.routeLocal, routeOverlay: DarkColors.routeOverlay,
        routeDirect: DarkColors.routeDirect, routeRelay: DarkColors.routeRelay,
        identity: IdentityColors.dark
    )
}

extension EnvironmentValues {
    @Entry var palette = Palette.light
}

/// A phone reads one step larger than the desktop's dense scale.
private let phoneStep = 1.0

extension Font {
    private static func sans(_ size: Double, medium: Bool = false) -> Font {
        .custom(medium ? "InterTight-Medium" : "InterTight-Regular", size: size)
    }

    static let bodyText = sans(TypeTokens.body + phoneStep)
    static let titleText = sans(TypeTokens.body + phoneStep, medium: true)
    static let mutedText = sans(TypeTokens.label + phoneStep)
    static let buttonText = sans(TypeTokens.body, medium: true)
    static let cardTitle = sans(TypeTokens.label, medium: true)
    static let captionText = sans(TypeTokens.caption)
    static let digitText = sans(22, medium: true)
    static let launchText = sans(TypeTokens.body + 2, medium: true)
    static let tabText = sans(11, medium: true)
}

private struct Themed: ViewModifier {
    @Environment(\.colorScheme) private var scheme

    func body(content: Content) -> some View {
        let palette = scheme == .dark ? Palette.dark : Palette.light
        content
            .environment(\.palette, palette)
            .font(.bodyText)
            .foregroundStyle(palette.ink)
            .tint(palette.primary)
    }
}

extension View {
    func themed() -> some View {
        modifier(Themed())
    }
}
