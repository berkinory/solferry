import SwiftUI

private let successSeconds = 3.0
private let failureSeconds = 5.0
private let pillShape = RoundedRectangle(cornerRadius: 20, style: .continuous)
private let reconnecting = Color(red: 0.961, green: 0.62, blue: 0.043)
private let spring = Animation.spring(response: 0.38, dampingFraction: 0.82)

/// In from above: a little smaller, blurred, and clear by the time it lands.
private struct Landing: Transition {
    func body(content: Content, phase: TransitionPhase) -> some View {
        content
            .opacity(phase.isIdentity ? 1 : 0)
            .scaleEffect(phase.isIdentity ? 1 : 0.96)
            .offset(y: phase.isIdentity ? 0 : -12)
            .blur(radius: phase.isIdentity ? 0 : 8)
    }
}

/// One pill for what is moving over the brand, and short messages underneath it.
struct Toaster: View {
    @Environment(Engine.self) private var engine

    var body: some View {
        let state = engine.state
        VStack(spacing: 6) {
            if !state.transfers.isEmpty { TransferPill(transfers: state.transfers).transition(Landing()) }
            ForEach(state.toasts) { ToastRow(toast: $0).transition(Landing()) }
        }
        .padding(.horizontal, SpaceTokens.lg)
        .padding(.top, 2)
        .animation(spring, value: state.toasts.map(\.id))
        .animation(spring, value: state.transfers.isEmpty)
    }
}

/// The design system's foreground as the fill, sized by what it says, so it flips with the theme.
private struct Shell<Content: View>: View {
    @Environment(\.palette) private var palette
    @ViewBuilder let content: Content

    var body: some View {
        content
            .foregroundStyle(palette.page)
            .background(palette.ink, in: pillShape)
            .shadow(color: .black.opacity(0.28), radius: 14, y: 6)
    }
}

/// One line that comes up from below whenever it changes.
private struct Label: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.system(size: 14, weight: .medium))
            .lineLimit(1)
            .truncationMode(.middle)
            .id(text)
            .transition(.asymmetric(insertion: .move(edge: .bottom).combined(with: .opacity), removal: .move(edge: .top).combined(with: .opacity)))
    }
}

/// The row every toast is built from: 16 points in, wider on the side with no button.
private struct PillRow<Content: View>: View {
    var closes = false
    @ViewBuilder let content: Content

    var body: some View {
        HStack(spacing: SpaceTokens.md) { content }
            .padding(.leading, 16)
            .padding(.trailing, closes ? 8 : 20)
            .frame(height: 44)
    }
}

/// A filled circle with a mark that lands with a small overshoot.
private struct Badge: View {
    @Environment(\.palette) private var palette
    @State private var landed = false
    let tone: Tone

    var body: some View {
        Icon(tone == .success ? .check : .cancel, tint: .white, size: 10)
            .frame(width: 16, height: 16)
            .background(tone == .success ? palette.successFill : palette.dangerFill, in: Circle())
            .scaleEffect(landed ? 1 : 0.6)
            .onAppear { withAnimation(.spring(response: 0.34, dampingFraction: 0.42)) { landed = true } }
    }
}

private struct Ring: View {
    let fraction: Double
    let tone: Color

    var body: some View {
        ZStack {
            Circle().stroke(tone.opacity(0.25), lineWidth: 2)
            Circle()
                .trim(from: 0, to: min(max(fraction, 0), 1))
                .stroke(tone, style: StrokeStyle(lineWidth: 2, lineCap: .round))
                .rotationEffect(.degrees(-90))
                .animation(.easeOut(duration: 0.24), value: fraction)
        }
        .frame(width: 16, height: 16)
    }
}

private struct Dismiss: View {
    @Environment(\.palette) private var palette
    let label: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Icon(.cancel, tint: palette.page.opacity(0.76), size: 13).frame(width: 28, height: 28).contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(label)
    }
}

private struct RunningRow: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let transfer: Transfer
    var wide = false

    var body: some View {
        let waiting = transfer.phase == .reconnecting
        let line = waiting ? "Reconnecting to \(transfer.peerName)" : "\(transfer.sending ? "Sending" : "Receiving") \(transfer.label)"
        PillRow(closes: true) {
            Ring(fraction: transfer.fraction, tone: waiting ? reconnecting : palette.page)
            Label(text: line)
            if wide { Spacer(minLength: 0) }
            Percent(fraction: transfer.fraction)
            Dismiss(label: "Cancel") { engine.bridge.cancel(transfer: transfer.id) }
        }
    }
}

private struct Percent: View {
    @Environment(\.palette) private var palette
    let fraction: Double

    var body: some View {
        Text("\(Int(fraction * 100))%")
            .font(.system(size: 13))
            .foregroundStyle(palette.page.opacity(0.76))
            .contentTransition(.numericText())
    }
}

private struct TransferPill: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    @State private var open = false
    @State private var shakes = 0
    let transfers: [Transfer]

    private var running: [Transfer] { transfers.filter(\.running) }

    var body: some View {
        Shell {
            VStack(alignment: .leading, spacing: 0) {
                face
                if open, running.count > 1 {
                    ForEach(running) { RunningRow(transfer: $0, wide: true).transition(.opacity) }
                }
            }
            .frame(maxWidth: open && running.count > 1 ? 360 : nil, alignment: .leading)
        }
        .clipShape(pillShape)
        .animation(spring, value: running.count)
        .animation(spring, value: running.first?.phase)
        .animation(spring, value: open)
        .keyframeAnimator(initialValue: 0.0, trigger: shakes) { view, shift in
            view.offset(x: shift)
        } keyframes: { _ in
            KeyframeTrack {
                LinearKeyframe(-3, duration: 0.07)
                LinearKeyframe(3, duration: 0.07)
                LinearKeyframe(-3, duration: 0.07)
                LinearKeyframe(0, duration: 0.07)
            }
        }
    }

    @ViewBuilder private var face: some View {
        if running.count > 1 {
            many
        } else if let only = running.first {
            RunningRow(transfer: only).id(only.id)
        } else if let ended = transfers.first {
            endedRow(ended).id(ended.id)
        }
    }

    private var many: some View {
        let moved = running.reduce(UInt64(0)) { $0 + $1.transferred }
        let total = max(running.reduce(UInt64(0)) { $0 + $1.total }, 1)
        let fraction = Double(moved) / Double(total)
        return PillRow {
            Ring(fraction: fraction, tone: palette.page)
            Label(text: "\(running.count) transfers")
            if open { Spacer(minLength: 0) }
            Percent(fraction: fraction)
            Icon(open ? .chevronUp : .chevronDown, tint: palette.page.opacity(0.76), size: 15)
        }
        .contentShape(Rectangle())
        .onTapGesture { open.toggle() }
    }

    private func endedRow(_ ended: Transfer) -> some View {
        let outcome = ended.outcome
        return PillRow {
            Badge(tone: outcome.tone)
            Label(text: outcome.title)
        }
        .sensoryFeedback(outcome.tone == .success ? .success : .error, trigger: ended.id)
        .task(id: ended.id) {
            if outcome.tone == .failure { shakes += 1 }
            try? await Task.sleep(for: .seconds(outcome.tone == .success ? successSeconds : failureSeconds))
            engine.clearTransfer(ended.id)
        }
    }
}

private struct ToastRow: View {
    @Environment(Engine.self) private var engine
    @State private var shakes = 0
    let toast: Toast

    var body: some View {
        Shell {
            PillRow(closes: toast.tone == .failure) {
                Badge(tone: toast.tone)
                Label(text: toast.title)
                if toast.tone == .failure { Dismiss(label: "Dismiss") { engine.dismissToast(toast.id) } }
            }
        }
        .keyframeAnimator(initialValue: 0.0, trigger: shakes) { view, shift in
            view.offset(x: shift)
        } keyframes: { _ in
            KeyframeTrack {
                LinearKeyframe(-3, duration: 0.07)
                LinearKeyframe(3, duration: 0.07)
                LinearKeyframe(-3, duration: 0.07)
                LinearKeyframe(0, duration: 0.07)
            }
        }
        .task(id: toast.id) {
            if toast.tone == .failure { shakes += 1 }
            try? await Task.sleep(for: .seconds(toast.tone == .success ? successSeconds : failureSeconds))
            engine.dismissToast(toast.id)
        }
    }
}
