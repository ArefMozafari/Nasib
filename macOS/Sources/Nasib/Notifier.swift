import UserNotifications

/// Posts the app's own notifications, so they carry the Nasib name and icon.
enum Notifier {
    /// The notification's user info key for a page that clicking it opens.
    static let pageURLKey = "pageURL"

    static func requestPermission() {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }
    }

    static func post(title: String, body: String, opening pageURL: URL? = nil) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        if let pageURL {
            content.userInfo = [pageURLKey: pageURL.absoluteString]
        }
        let request = UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: nil)
        UNUserNotificationCenter.current().add(request)
    }
}
