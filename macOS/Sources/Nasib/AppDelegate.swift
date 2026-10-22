import AppKit
import UserNotifications

final class AppDelegate: NSObject, NSApplicationDelegate, UNUserNotificationCenterDelegate {
    /// Hands over to the instance that is already running and quits this one.
    private func terminateIfAnotherInstanceIsRunning() {
        guard let bundleIdentifier = Bundle.main.bundleIdentifier else { return }
        let currentProcessIdentifier = ProcessInfo.processInfo.processIdentifier
        let others = NSRunningApplication.runningApplications(withBundleIdentifier: bundleIdentifier)
            .filter { $0.processIdentifier != currentProcessIdentifier }
        guard let other = others.first else { return }
        other.activate()
        NSApp.terminate(nil)
    }

    // The delegate is set before launch finishes, so a click that launched the app is delivered too.
    func applicationWillFinishLaunching(_ notification: Notification) {
        UNUserNotificationCenter.current().delegate = self
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        terminateIfAnotherInstanceIsRunning()
        NSApp.setActivationPolicy(.accessory)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }

    /// Opens the page a notification points at, such as a new release.
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        if let link = response.notification.request.content.userInfo[Notifier.pageURLKey] as? String,
           let pageURL = URL(string: link) {
            NSWorkspace.shared.open(pageURL)
        }
        completionHandler()
    }
}
