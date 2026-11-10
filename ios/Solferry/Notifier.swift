import UIKit
import UserNotifications

/// System notifications for things that need a person. On screen, the app itself shows them.
@MainActor
enum Notifier {
    static func requestPermission() {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound, .badge]) { _, _ in }
    }

    static func offer(key: UInt64, name: String) {
        show("Incoming files", "\(name) wants to send you files", id: offerId(key))
    }

    static func pairing(key: UInt64, name: String) {
        show("Pairing request", "\(name) wants to pair with this device", id: pairingId(key))
    }

    static func text(from name: String, copied: Bool) {
        show("Text from \(name)", copied ? "Copied to your clipboard" : "Open Solferry to copy it")
    }

    static func received(label: String, name: String, folder: String) {
        show("Files received", "\(label) from \(name), in \(folder)")
    }

    static func settleOffer(_ key: UInt64) {
        take(offerId(key))
    }

    static func settlePairing(_ key: UInt64) {
        take(pairingId(key))
    }

    private static func offerId(_ key: UInt64) -> String { "offer-\(key)" }

    private static func pairingId(_ key: UInt64) -> String { "pairing-\(key)" }

    private static func take(_ id: String) {
        let center = UNUserNotificationCenter.current()
        center.removeDeliveredNotifications(withIdentifiers: [id])
        center.removePendingNotificationRequests(withIdentifiers: [id])
    }

    private static func show(_ title: String, _ body: String, id: String = UUID().uuidString) {
        guard UIApplication.shared.applicationState != .active else { return }
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default
        UNUserNotificationCenter.current().add(UNNotificationRequest(identifier: id, content: content, trigger: nil))
    }
}
