import SwiftUI
import UIKit
import UserNotifications

/// What iOS has to allow Solferry. iOS asks for local network access by itself the first time the
/// app looks around and never tells the app the answer, so that row only leads to Settings.
struct PermissionsCard: View {
    @Environment(\.palette) private var palette
    @Environment(\.openURL) private var openURL
    @Environment(\.scenePhase) private var phase
    @State private var notifications: UNAuthorizationStatus?

    var body: some View {
        Card(glyph: .shield, title: "Permissions") {
            PaddedWell {
                PermissionRow(glyph: .notification, title: "Notifications", detail: "For incoming files, texts and pairing requests.") {
                    notificationStatus
                } action: {
                    notificationAction
                }
                PermissionRow(glyph: .nearby, title: "Local network", detail: "Finds your devices and sends to them directly.") {
                    EmptyView()
                } action: {
                    settingsButton(UIApplication.openSettingsURLString)
                }
            }
        }
        // Read again on every return, as after a trip to Settings.
        .task(id: phase) { if phase == .active { notifications = await Notifier.status() } }
    }

    @ViewBuilder private var notificationStatus: some View {
        switch notifications {
        case .authorized, .provisional, .ephemeral: StatusLine("Allowed", dot: palette.success)
        case .denied: StatusLine("Off", dot: palette.danger)
        case .notDetermined: StatusLine("Not asked yet", dot: palette.warning)
        default: EmptyView()
        }
    }

    @ViewBuilder private var notificationAction: some View {
        switch notifications {
        case .notDetermined:
            ActionButton(label: "Allow", kind: .primary, scale: .small) {
                Task { notifications = await Notifier.request() }
            }
        case .denied: settingsButton(UIApplication.openNotificationSettingsURLString)
        default: EmptyView()
        }
    }

    private func settingsButton(_ link: String) -> some View {
        ActionButton(label: "Open Settings", glyph: .external, scale: .small) {
            if let url = URL(string: link) { openURL(url) }
        }
    }
}

private struct PermissionRow<Status: View, Action: View>: View {
    @Environment(\.palette) private var palette
    let glyph: Glyph
    let title: LocalizedStringKey
    let detail: LocalizedStringKey
    @ViewBuilder let status: Status
    @ViewBuilder let action: Action

    var body: some View {
        HStack(alignment: .top, spacing: SpaceTokens.md) {
            Tile(glyph: glyph, fill: palette.control, tone: palette.inkMuted)
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                status
                Muted(detail)
                action
            }
            Spacer(minLength: 0)
        }
    }
}
