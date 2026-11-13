import UIKit
import UserNotifications

/// System notifications for things that need a person. On screen, the app itself shows them.
@MainActor
enum Notifier {
    private nonisolated static let offerCategory = "offer"
    private nonisolated static let acceptAction = "accept"
    private nonisolated static let declineAction = "decline"
    private static let responder = Responder()

    /// Gives offer notifications Accept and Decline, answered without opening the app. Only plain
    /// answers: always accepting a device is a choice to make in the app.
    static func listen(_ engine: Engine) {
        responder.engine = engine
        let center = UNUserNotificationCenter.current()
        center.delegate = responder
        let accept = UNNotificationAction(identifier: acceptAction, title: String(localized: "Accept"))
        let decline = UNNotificationAction(identifier: declineAction, title: String(localized: "Decline"), options: .destructive)
        center.setNotificationCategories([UNNotificationCategory(identifier: offerCategory, actions: [accept, decline], intentIdentifiers: [])])
    }

    nonisolated static func status() async -> UNAuthorizationStatus {
        await UNUserNotificationCenter.current().notificationSettings().authorizationStatus
    }

    /// Shows the system's prompt, which iOS does only once, and returns what it says now.
    nonisolated static func request() async -> UNAuthorizationStatus {
        _ = try? await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound, .badge])
        return await status()
    }

    static func offer(key: UInt64, name: String) {
        show(
            String(localized: "Incoming files"), String(localized: "\(name) wants to send you files"),
            id: offerId(key), category: offerCategory, info: ["key": String(key)]
        )
    }

    static func pairing(key: UInt64, name: String) {
        show(String(localized: "Pairing request"), String(localized: "\(name) wants to pair with this device"), id: pairingId(key))
    }

    static func text(from name: String, copied: Bool) {
        show(String(localized: "Text from \(name)"), copied ? String(localized: "Copied to your clipboard") : String(localized: "Open Solferry to copy it"))
    }

    static func received(label: String, name: String, folder: String) {
        show(String(localized: "Files received"), String(localized: "\(label) from \(name), in \(folder)"))
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

    private static func show(_ title: String, _ body: String, id: String = UUID().uuidString, category: String = "", info: [String: String] = [:]) {
        guard UIApplication.shared.applicationState != .active else { return }
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.categoryIdentifier = category
        content.userInfo = info
        content.sound = .default
        UNUserNotificationCenter.current().add(UNNotificationRequest(identifier: id, content: content, trigger: nil))
    }

    /// The center keeps its delegate weakly, so Notifier holds it.
    private final class Responder: NSObject, UNUserNotificationCenterDelegate {
        weak var engine: Engine?

        func userNotificationCenter(
            _ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse,
            withCompletionHandler completionHandler: @escaping () -> Void
        ) {
            let action = response.actionIdentifier
            let key = (response.notification.request.content.userInfo["key"] as? String).flatMap(UInt64.init)
            DispatchQueue.main.async { [weak self] in
                if let key, action == acceptAction || action == declineAction {
                    MainActor.assumeIsolated { self?.engine?.answerOffer(key, accept: action == acceptAction) }
                }
                completionHandler()
            }
        }
    }
}
