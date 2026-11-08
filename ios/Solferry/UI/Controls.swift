import SwiftUI

enum ButtonKind {
    case primary, secondary, ghost, danger
}

enum ButtonScale {
    case small, medium

    var height: Double { self == .medium ? 44 : 36 }
}

struct ActionStyle: ButtonStyle {
    @Environment(\.palette) private var palette
    @Environment(\.isEnabled) private var enabled
    let kind: ButtonKind
    let iconOnly: Bool
    let scale: ButtonScale

    func makeBody(configuration: Configuration) -> some View {
        let look = look(pressed: configuration.isPressed)
        configuration.label
            .font(.buttonText)
            .foregroundStyle(look.text)
            .lineLimit(1)
            .padding(.horizontal, iconOnly ? 0 : 14)
            .frame(minWidth: iconOnly ? scale.height : nil, minHeight: scale.height)
            .background {
                ZStack {
                    controlShape.fill(look.fill)
                    if kind == .primary {
                        controlShape.fill(
                            Color.clear
                                .shadow(.inner(color: .white.opacity(0.22), radius: 0, y: 1))
                                .shadow(.inner(color: palette.primaryDeep.opacity(0.3), radius: 0, y: -1))
                        )
                    }
                    if let border = look.border { controlShape.strokeBorder(border) }
                }
            }
            .contentShape(controlShape)
            .opacity(enabled ? 1 : 0.5)
            .animation(.easeOut(duration: MotionTokens.quick / 1000), value: configuration.isPressed)
    }

    private func look(pressed: Bool) -> (fill: Color, text: Color, border: Color?) {
        switch kind {
        case .primary: (pressed ? palette.primaryHover : palette.primaryFill, palette.onPrimary, palette.primaryBorder)
        case .secondary: (pressed ? palette.raised : palette.page, palette.ink, palette.line)
        case .ghost: (pressed ? palette.raised : .clear, palette.inkMuted, nil)
        case .danger: (palette.dangerFill.opacity(pressed ? 0.9 : 1), palette.onPrimary, nil)
        }
    }
}

/// The app's one button, like the desktop's: a lit primary, a framed secondary, quiet ghosts.
struct ActionButton: View {
    var label: String?
    var glyph: Glyph?
    var kind = ButtonKind.secondary
    var scale = ButtonScale.medium
    var description: String?
    var wide = false
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                if let glyph { Icon(glyph, tint: nil, size: 18) }
                if let label { Text(label) }
            }
            .frame(maxWidth: wide ? .infinity : nil)
        }
        .buttonStyle(ActionStyle(kind: kind, iconOnly: label == nil, scale: scale))
        .accessibilityLabel(description ?? label ?? "")
    }
}

/// A one-line text field with an optional button inside its trailing edge.
struct Field<Trailing: View>: View {
    @Environment(\.palette) private var palette
    @FocusState private var focused: Bool
    @Binding var text: String
    let placeholder: String
    var onSubmit: () -> Void = {}
    @ViewBuilder var trailing: Trailing

    var body: some View {
        HStack(spacing: 0) {
            TextField("", text: $text, prompt: Text(placeholder).foregroundStyle(palette.inkMuted))
                .focused($focused)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .onSubmit(onSubmit)
            trailing
        }
        .padding(.leading, 12)
        .padding(.trailing, 4)
        .frame(minHeight: 44)
        .background(palette.page, in: controlShape)
        .overlay(controlShape.strokeBorder(focused ? palette.primary : palette.line))
        .animation(.easeOut(duration: MotionTokens.quick / 1000), value: focused)
    }
}

extension Field where Trailing == EmptyView {
    init(text: Binding<String>, placeholder: String, onSubmit: @escaping () -> Void = {}) {
        self.init(text: text, placeholder: placeholder, onSubmit: onSubmit, trailing: { EmptyView() })
    }
}

/// Apple's own paste control: iOS gives an app the clipboard only through a press on this.
struct ClipboardButton: View {
    @Environment(\.palette) private var palette
    let onText: (String) -> Void

    var body: some View {
        PasteButton(payloadType: String.self) { strings in
            guard let text = strings.first else { return }
            Task { @MainActor in onText(text) }
        }
        .labelStyle(.iconOnly)
        .buttonBorderShape(.roundedRectangle(radius: RadiusTokens.control))
        .tint(palette.raised)
    }
}

struct PasteInto: View {
    @Binding var text: String

    var body: some View {
        ClipboardButton { text = $0.trimmingCharacters(in: .whitespacesAndNewlines) }
    }
}

struct ProgressLine: View {
    @Environment(\.palette) private var palette
    let fraction: Double
    let tone: Color

    var body: some View {
        GeometryReader { space in
            ZStack(alignment: .leading) {
                Capsule().fill(palette.control)
                Capsule().fill(tone).frame(width: space.size.width * min(max(fraction, 0), 1))
            }
        }
        .frame(height: 6)
    }
}
