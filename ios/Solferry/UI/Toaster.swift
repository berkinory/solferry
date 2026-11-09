import SwiftUI

private let successSeconds = 3.0
private let failureSeconds = 5.0
private let pillShape = RoundedRectangle(cornerRadius: 20, style: .continuous)
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

private func routeName(_ route: TransferRoute) -> String {
    switch route {
    case .local: "local network"
    case .overlay: "overlay network"
    case .direct: "direct"
    case .relay: "relay"
    }
}

private func motion(_ transfer: Transfer) -> String {
    let who = "\(transfer.sending ? "To" : "From") \(transfer.peerName)"
    if transfer.phase == .reconnecting { return "\(who)  ·  Reconnecting" }
    if transfer.bytesPerSecond <= 1 { return "\(who)  ·  \(routeName(transfer.route))" }
    return "\(who)  ·  \(bytes(UInt64(transfer.bytesPerSecond)))/s  ·  \(routeName(transfer.route))"
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
        .frame(maxWidth: 440)
        .animation(spring, value: state.toasts.map(\.id))
        .animation(spring, value: state.transfers.isEmpty)
    }
}

/// The design system's foreground as the fill, so the pill flips with the theme by itself.
private struct Shell<Content: View>: View {
    @Environment(\.palette) private var palette
    @ViewBuilder let content: Content

    var body: some View {
        content
            .foregroundStyle(palette.page)
            .glassFill(palette.ink, in: pillShape, interactive: true)
            .shadow(color: .black.opacity(0.25), radius: 12, y: 5)
    }
}

private struct Lines: View {
    @Environment(\.palette) private var palette
    let title: String
    let detail: String

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(title).font(.system(size: 13, weight: .medium)).lineLimit(1)
            Text(detail).font(.system(size: 11.5)).foregroundStyle(palette.page.opacity(0.62)).lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .transition(.asymmetric(insertion: .move(edge: .bottom).combined(with: .opacity), removal: .move(edge: .top).combined(with: .opacity)))
    }
}

/// A filled circle with a mark that lands with a small overshoot.
private struct Badge: View {
    @Environment(\.palette) private var palette
    @State private var landed = false
    let tone: Tone

    var body: some View {
        Icon(tone == .success ? .check : .cancel, tint: .white, size: 11)
            .frame(width: 18, height: 18)
            .background(tone == .success ? palette.successFill : palette.dangerFill, in: Circle())
            .scaleEffect(landed ? 1 : 0.6)
            .onAppear { withAnimation(.spring(response: 0.34, dampingFraction: 0.42)) { landed = true } }
    }
}

private struct Ring: View {
    @Environment(\.palette) private var palette
    let fraction: Double

    var body: some View {
        ZStack {
            Circle().stroke(palette.page.opacity(0.2), lineWidth: 2.5)
            Circle()
                .trim(from: 0, to: min(max(fraction, 0), 1))
                .stroke(palette.page, style: StrokeStyle(lineWidth: 2.5, lineCap: .round))
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
            Icon(.cancel, tint: palette.page.opacity(0.62), size: 14).frame(width: 32, height: 32).contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(label)
    }
}

private struct RunningRow: View {
    @Environment(Engine.self) private var engine
    @Environment(\.palette) private var palette
    let transfer: Transfer

    var body: some View {
        HStack(spacing: SpaceTokens.md) {
            Ring(fraction: transfer.fraction)
            Lines(title: transfer.label, detail: motion(transfer))
            Text("\(Int(transfer.fraction * 100))%")
                .font(.system(size: 12))
                .foregroundStyle(palette.page.opacity(0.62))
                .contentTransition(.numericText())
            Dismiss(label: "Cancel") { engine.bridge.cancel(transfer: transfer.id) }
        }
        .padding(.leading, 14)
        .padding(.trailing, 6)
        .frame(height: 44)
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
            VStack(spacing: 0) {
                face
                if open, running.count > 1 {
                    ForEach(running) { RunningRow(transfer: $0).transition(.opacity) }
                }
            }
        }
        .clipShape(pillShape)
        .animation(spring, value: running.count)
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
        let out = running.filter(\.sending).count
        let detail = out == 0 ? "Receiving \(running.count)" : out == running.count ? "Sending \(running.count)" : "Sending \(out), receiving \(running.count - out)"
        return HStack(spacing: SpaceTokens.md) {
            Ring(fraction: Double(moved) / Double(total))
            Lines(title: "\(running.count) transfers", detail: detail)
            Icon(.more, tint: palette.page.opacity(0.62), size: 15).padding(.trailing, 8)
        }
        .padding(.leading, 14)
        .padding(.trailing, 6)
        .frame(height: 44)
        .contentShape(Rectangle())
        .onTapGesture { open.toggle() }
    }

    private func endedRow(_ ended: Transfer) -> some View {
        let outcome = ended.outcome
        return HStack(spacing: SpaceTokens.md) {
            Badge(tone: outcome.tone)
            Lines(title: outcome.title, detail: outcome.detail)
        }
        .padding(.horizontal, 14)
        .frame(height: 44)
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
            HStack(spacing: SpaceTokens.md) {
                Badge(tone: toast.tone)
                Lines(title: toast.title, detail: toast.detail)
                Dismiss(label: "Dismiss") { engine.dismissToast(toast.id) }
            }
            .padding(.leading, 14)
            .padding(.trailing, 6)
            .frame(height: 44)
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
