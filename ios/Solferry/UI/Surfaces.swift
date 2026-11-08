import SwiftUI

let cardShape = RoundedRectangle(cornerRadius: RadiusTokens.card, style: .continuous)
let wellShape = RoundedRectangle(cornerRadius: RadiusTokens.well, style: .continuous)
let controlShape = RoundedRectangle(cornerRadius: RadiusTokens.control, style: .continuous)
let fieldShape = RoundedRectangle(cornerRadius: RadiusTokens.field, style: .continuous)

/// One line and 3pt of padding, so what it holds sits exactly 4pt inside it.
struct CardFrame<Content: View>: View {
    @Environment(\.palette) private var palette
    var line: Color?
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 0) { content }
            .padding(4)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(palette.page, in: cardShape)
            .overlay(cardShape.strokeBorder(line ?? palette.line))
            .shadow(color: .black.opacity(0.06), radius: 2, y: 1)
    }
}

/// Set into the frame by shade and a soft inner shadow, with no line of its own.
struct Well<Content: View>: View {
    @Environment(\.palette) private var palette
    var ground: Color?
    var padding = SpaceTokens.sm
    var spacing = SpaceTokens.xs
    var action: (() -> Void)?
    @ViewBuilder let content: Content

    var body: some View {
        if let action {
            Button(action: action) { face(pressed: false) }
                .buttonStyle(WellPress(ground: ground ?? palette.well, raised: palette.raised, night: palette.night, padding: padding))
        } else {
            face(pressed: false)
                .padding(padding)
                .background { WellGround(fill: ground ?? palette.well, night: palette.night) }
        }
    }

    private func face(pressed _: Bool) -> some View {
        VStack(alignment: .leading, spacing: spacing) { content }
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private struct WellGround: View {
    let fill: Color
    let night: Bool

    var body: some View {
        wellShape
            .fill(fill.shadow(.inner(color: .black.opacity(night ? 0.45 : 0.05), radius: 2, y: 1)))
            .overlay(wellShape.strokeBorder(night ? Color.white.opacity(0.03) : Color.black.opacity(0.035)))
    }
}

private struct WellPress: ButtonStyle {
    let ground: Color
    let raised: Color
    let night: Bool
    let padding: Double

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .padding(padding)
            .background { WellGround(fill: configuration.isPressed ? raised : ground, night: night) }
            .contentShape(wellShape)
    }
}

/// A well with room for text, spaced for reading rather than for rows.
struct PaddedWell<Content: View>: View {
    var ground: Color?
    @ViewBuilder let content: Content

    var body: some View {
        Well(ground: ground, padding: SpaceTokens.lg, spacing: SpaceTokens.sm) { content }
    }
}

/// A frame with an icon and title over the well that holds its rows.
struct Card<Aside: View, Content: View>: View {
    @Environment(\.palette) private var palette
    let glyph: Glyph
    let title: String
    var tone: Color?
    var line: Color?
    @ViewBuilder var aside: Aside
    @ViewBuilder let content: Content

    var body: some View {
        CardFrame(line: line) {
            HStack(spacing: SpaceTokens.sm) {
                Icon(glyph, tint: tone ?? palette.inkMuted, size: 16)
                Text(title)
                    .font(.cardTitle)
                    .foregroundStyle(palette.ink.opacity(0.8))
                    .lineLimit(1)
                Spacer(minLength: 0)
                aside
            }
            .frame(minHeight: 36)
            .padding(.horizontal, SpaceTokens.lg)
            content
        }
    }
}

extension Card where Aside == EmptyView {
    init(glyph: Glyph, title: String, tone: Color? = nil, line: Color? = nil, @ViewBuilder content: () -> Content) {
        self.init(glyph: glyph, title: title, tone: tone, line: line, aside: { EmptyView() }, content: content)
    }
}

struct Count: View {
    let value: Int

    var body: some View {
        Caption(String(value))
    }
}
