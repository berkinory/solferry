import SwiftUI

private let pulseSeconds = 2.8
private let breathSeconds = 3.6
private let radarSize = 172.0
private let coreSize = 52.0
private let orbits = [104.0, 168.0]

/// The kinds of device that could turn up, resting on the orbit where they would appear.
private let ghosts: [(glyph: Glyph, x: Double, y: Double, phase: Double)] = [
    (.apple, -65, -41, 0),
    (.windows, 66, -30, 0.25),
    (.android, -53, 56, 0.5),
    (.linux, 61, 51, 0.75),
]

private func cycle(_ date: Date, _ seconds: Double) -> Double {
    date.timeIntervalSinceReferenceDate.truncatingRemainder(dividingBy: seconds) / seconds
}

/// A ring leaving the center and fading on its way out.
private func pulse(_ context: GraphicsContext, _ size: CGSize, progress: Double, from: Double, tone: Color) {
    let radius = from + (min(size.width, size.height) / 2 - from) * progress
    let box = CGRect(x: size.width / 2 - radius, y: size.height / 2 - radius, width: radius * 2, height: radius * 2)
    context.stroke(Path(ellipseIn: box), with: .color(tone.opacity(0.7 * (1 - progress))), lineWidth: 1)
}

/// Runs at 20 fps: this stays on screen as long as the app is open.
private struct Radar: View {
    @Environment(\.palette) private var palette

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 20)) { timeline in
            let date = timeline.date
            ZStack {
                Canvas { context, size in
                    for orbit in orbits {
                        let box = CGRect(x: (size.width - orbit) / 2, y: (size.height - orbit) / 2, width: orbit, height: orbit)
                        context.stroke(Path(ellipseIn: box), with: .color(palette.line), lineWidth: 1)
                    }
                    let progress = cycle(date, pulseSeconds)
                    pulse(context, size, progress: progress, from: coreSize / 2, tone: palette.primary)
                    pulse(context, size, progress: (progress + 0.5).truncatingRemainder(dividingBy: 1), from: coreSize / 2, tone: palette.primary)
                }
                ForEach(ghosts, id: \.phase) { ghost in
                    Tile(glyph: ghost.glyph, fill: palette.control, tone: palette.inkMuted)
                        .offset(x: ghost.x, y: ghost.y)
                        .opacity(0.25 + 0.75 * (sin(2 * .pi * (cycle(date, breathSeconds) + ghost.phase)) * 0.5 + 0.5))
                }
                Icon(.nearby, tint: palette.inkMuted, size: 24)
                    .frame(width: coreSize, height: coreSize)
                    .background(palette.control, in: RoundedRectangle(cornerRadius: RadiusTokens.field, style: .continuous))
                    .overlay(RoundedRectangle(cornerRadius: RadiusTokens.field, style: .continuous).strokeBorder(palette.line))
            }
            .frame(width: radarSize, height: radarSize)
        }
        .accessibilityHidden(true)
    }
}

/// The whole home screen for someone with no devices yet.
struct RadarHero: View {
    var body: some View {
        CardFrame {
            Well(padding: SpaceTokens.lg, spacing: SpaceTokens.sm) {
                VStack(spacing: SpaceTokens.sm) {
                    Radar()
                    Text("Looking for nearby devices").font(.titleText).padding(.top, SpaceTokens.sm)
                    Muted("Open Solferry on another device on this network and it shows up here.")
                        .multilineTextAlignment(.center)
                    NavigationLink(value: Route.addDevice) {
                        HStack(spacing: 6) {
                            Icon(.qr, size: 18)
                            Text("Pair with code")
                        }
                    }
                    .buttonStyle(ActionStyle(kind: .secondary, iconOnly: false, scale: .small))
                    .padding(.top, SpaceTokens.xs)
                }
                .frame(maxWidth: .infinity)
            }
        }
    }
}

/// One quiet line for when nothing new is around but the search is still on.
struct Scanning: View {
    @Environment(\.palette) private var palette

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            TimelineView(.animation(minimumInterval: 1.0 / 20)) { timeline in
                ZStack {
                    Canvas { context, size in
                        pulse(context, size, progress: cycle(timeline.date, pulseSeconds), from: 4, tone: palette.primary)
                    }
                    Circle().fill(palette.primary).frame(width: 8, height: 8)
                }
                .frame(width: 38, height: 38)
            }
            Muted("Looking for new devices on this network")
        }
        .padding(SpaceTokens.sm)
    }
}
