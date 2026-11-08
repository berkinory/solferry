import SwiftUI

enum Glyph: String {
    case alert, android, apple, blocked, cancel, check, clipboard, computer, copy, delete, devices, download
    case external, folder, key, linux, more, nearby, qr, relay, send, settings, shield, transfers, upload, view, windows
}

func platformGlyph(_ platform: DevicePlatform?) -> Glyph {
    switch platform {
    case .macos, .ios: .apple
    case .windows: .windows
    case .linux: .linux
    case .android: .android
    case nil: .computer
    }
}

/// `tint` of nil keeps the color of the text around it, as buttons need.
struct Icon: View {
    let glyph: Glyph
    let tint: Color?
    let size: Double

    init(_ glyph: Glyph, tint: Color? = nil, size: Double = 20) {
        self.glyph = glyph
        self.tint = tint
        self.size = size
    }

    var body: some View {
        let image = Image(glyph.rawValue).renderingMode(.template).resizable().frame(width: size, height: size)
        if let tint { image.foregroundStyle(tint) } else { image }
    }
}

struct Tile: View {
    let glyph: Glyph
    let fill: Color
    let tone: Color

    var body: some View {
        Icon(glyph, tint: tone, size: 18)
            .frame(width: 38, height: 38)
            .background(fill, in: RoundedRectangle(cornerRadius: RadiusTokens.floor, style: .continuous))
    }
}

/// A device's tile: its platform logo in the color its key gives it, so copied names still differ.
struct DeviceTile: View {
    @Environment(\.palette) private var palette
    let identity: Identity
    var platform: DevicePlatform?

    var body: some View {
        let tone = palette.identity[Int(identity.tone) % palette.identity.count]
        Tile(glyph: platformGlyph(identity.platform ?? platform), fill: tone.opacity(0.16), tone: tone)
    }
}

struct Mono: View {
    @Environment(\.palette) private var palette
    let text: String
    var color: Color?

    init(_ text: String, color: Color? = nil) {
        self.text = text
        self.color = color
    }

    var body: some View {
        Text(text).font(.monoText).foregroundStyle(color ?? palette.inkMuted).lineLimit(1)
    }
}

struct Muted: View {
    @Environment(\.palette) private var palette
    let text: String
    var color: Color?

    init(_ text: String, color: Color? = nil) {
        self.text = text
        self.color = color
    }

    var body: some View {
        Text(text).font(.mutedText).foregroundStyle(color ?? palette.inkMuted)
    }
}

/// Muted text after a colored dot, for anything that has a state.
struct StatusLine: View {
    @Environment(\.palette) private var palette
    let text: String
    var dot: Color?

    init(_ text: String, dot: Color? = nil) {
        self.text = text
        self.dot = dot
    }

    var body: some View {
        HStack(spacing: 6) {
            if let dot { Circle().fill(dot).frame(width: 6, height: 6) }
            Text(text).font(.mutedText).foregroundStyle(palette.inkMuted).lineLimit(1)
        }
    }
}

/// A pushed screen: a title in the bar and cards that scroll clear of the keyboard.
struct Page<Content: View>: View {
    @Environment(\.palette) private var palette
    let title: String
    @ViewBuilder let content: Content

    var body: some View {
        ScrollView {
            VStack(spacing: SpaceTokens.sm) { content }
                .padding(.horizontal, SpaceTokens.md)
                .padding(.vertical, SpaceTokens.xs)
        }
        .scrollDismissesKeyboard(.interactively)
        .background(palette.page.ignoresSafeArea())
        .navigationTitle(title)
        .navigationBarTitleDisplayMode(.inline)
    }
}

extension View {
    func confirm(
        _ title: String,
        isPresented: Binding<Bool>,
        message: String,
        action: String,
        onConfirm: @escaping () -> Void
    ) -> some View {
        alert(title, isPresented: isPresented) {
            Button(action, role: .destructive, action: onConfirm)
            Button("Cancel", role: .cancel) {}
        } message: {
            Text(message)
        }
    }
}

func bytes(_ value: UInt64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"]
    var amount = Double(value)
    var unit = 0
    while amount >= 1024, unit < units.count - 1 {
        amount /= 1024
        unit += 1
    }
    return unit == 0 ? "\(value) B" : String(format: "%.1f %@", amount, units[unit])
}
