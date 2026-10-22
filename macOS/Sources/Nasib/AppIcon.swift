import AppKit

/// The app's own icon at a given point size, as the system renders it (Liquid Glass on macOS 26).
enum AppIcon {
    static func image(points: CGFloat) -> NSImage {
        let icon = NSApp.applicationIconImage.copy() as! NSImage
        icon.size = NSSize(width: points, height: points)
        return icon
    }

    /// Opens the standard About window, which shows the icon, version and copyright from Info.plist.
    static func showAboutPanel() {
        NSApp.activate(ignoringOtherApps: true)
        NSApp.orderFrontStandardAboutPanel(nil)
    }
}
