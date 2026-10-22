import SwiftUI

@main
struct NasibApp: App {
    static let mainWindowIdentifier = "main"

    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var engine = Engine()

    var body: some Scene {
        Window("Nasib", id: Self.mainWindowIdentifier) {
            ContentView().environmentObject(engine)
        }
        .windowResizability(.contentSize)
        .defaultLaunchBehavior(.suppressed)

        MenuBarExtra {
            MenuBarMenu().environmentObject(engine)
        } label: {
            Image(systemName: engine.state.symbolName)
        }
    }
}
