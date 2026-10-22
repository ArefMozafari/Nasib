import SwiftUI

@main
struct NasibApp: App {
    static let mainWindowIdentifier = "main"

    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var engine = Engine()
    @StateObject private var updateChecker = UpdateChecker()

    var body: some Scene {
        Window("Nasib", id: Self.mainWindowIdentifier) {
            ContentView().environmentObject(engine).environmentObject(updateChecker)
        }
        .windowResizability(.contentSize)
        .defaultLaunchBehavior(.suppressed)

        MenuBarExtra {
            MenuBarMenu().environmentObject(engine).environmentObject(updateChecker)
        } label: {
            Image(nsImage: MenuBarGlyph.image(named: engine.state.glyphName))
        }
    }
}
