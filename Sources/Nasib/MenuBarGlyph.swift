import AppKit

/// Loads the menu bar glyphs rendered into the bundle by build.sh, as template images so macOS tints them.
enum MenuBarGlyph {
    private static let fallbackSymbolName = "antenna.radiowaves.left.and.right"

    static func image(named name: String) -> NSImage {
        let image = NSImage(named: name) ?? NSImage(systemSymbolName: fallbackSymbolName, accessibilityDescription: nil)!
        image.isTemplate = true
        return image
    }
}
