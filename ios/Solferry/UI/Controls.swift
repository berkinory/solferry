import SwiftUI

enum ButtonKind {
    case primary, secondary, ghost, danger
}

enum ButtonScale {
    case small, medium

    var height: Double { self == .medium ? 44 : 36 }
}

/// The system's glass button styles, wearing the design system's colors and corners.
struct ActionButton: View {
    @Environment(\.palette) private var palette
    var label: String?
    var glyph: Glyph?
    var kind = ButtonKind.secondary
    var scale = ButtonScale.medium
    var description: String?
    var wide = false
    let action: () -> Void

    var body: some View {
        Group {
            switch kind {
            case .primary: button.buttonStyle(.glassProminent).tint(palette.primaryFill)
            case .danger: button.buttonStyle(.glassProminent).tint(palette.dangerFill)
            case .secondary: button.buttonStyle(.glass).foregroundStyle(palette.ink)
            case .ghost: button.buttonStyle(.borderless).tint(palette.inkMuted)
            }
        }
        .controlSize(scale == .medium ? .large : .regular)
        .buttonBorderShape(.roundedRectangle(radius: scale == .medium ? RadiusTokens.control : RadiusTokens.floor))
        .accessibilityLabel(description ?? label ?? "")
    }

    private var button: some View {
        Button(role: kind == .danger ? .destructive : nil, action: action) {
            HStack(spacing: 6) {
                if let glyph { Icon(glyph, tint: nil, size: 18) }
                if let label { Text(label).font(.buttonText).lineLimit(1) }
            }
            .frame(maxWidth: wide ? .infinity : nil, minHeight: label == nil ? 28 : nil)
            .frame(minWidth: label == nil ? 28 : nil)
        }
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
    let fraction: Double
    let tone: Color

    var body: some View {
        ProgressView(value: min(max(fraction, 0), 1)).tint(tone)
    }
}
