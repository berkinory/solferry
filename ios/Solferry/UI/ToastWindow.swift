import SwiftUI
import UIKit

/// Sheets cover everything in the app's own window, so toasts live in a window above it.
/// Only the area the toasts take up catches touches.
final class ToastWindow: UIWindow {
    private static var shown: ToastWindow?
    fileprivate var taken = CGRect.zero

    override func point(inside point: CGPoint, with event: UIEvent?) -> Bool {
        taken.contains(point)
    }

    @MainActor static func show(_ engine: Engine) {
        guard shown == nil, let scene = UIApplication.shared.connectedScenes.compactMap({ $0 as? UIWindowScene }).first else { return }
        let window = ToastWindow(windowScene: scene)
        let host = UIHostingController(rootView: Hosted(window: window).environment(engine).themed())
        host.view.backgroundColor = .clear
        window.rootViewController = host
        window.windowLevel = .normal + 1
        window.isHidden = false
        shown = window
    }
}

private struct Hosted: View {
    let window: ToastWindow

    var body: some View {
        Toaster()
            .onGeometryChange(for: CGRect.self) { $0.frame(in: .global) } action: { window.taken = $0 }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
    }
}
