import SwiftUI

/// Sizing and spacing tokens for the main window and the menu.
enum Layout {
    static let windowWidth: CGFloat = 360
    /// Settings is wider, so the two location lists sit side by side instead of stacking.
    static let settingsWindowWidth: CGFloat = 620
    static let sectionSpacing: CGFloat = 16
    static let rowSpacing: CGFloat = 8
    /// The space inside a Settings group box, between its edge and the section's content.
    static let groupPadding: CGFloat = 8
    static let padding: CGFloat = 20
    static let statusDotSize: CGFloat = 10
    static let logHeight: CGFloat = 140
    /// Space around each log line, so its stripe reads as a row.
    static let logLinePadding: CGFloat = 4
    static let logLineVerticalPadding: CGFloat = 2
    static let locationListsHeight: CGFloat = 300
    /// The search field above each location list, and the line below it: the ranking switch on one side,
    /// the count on the other.
    static let columnControlHeight: CGFloat = 22
    static let protocolListHeight: CGFloat = 190
    static let headerIconSize: CGFloat = 32
    static let menuIconSize: CGFloat = 16
}

/// App name and icon at the top of the Status tab, with an update button at the trailing edge when a
/// newer release exists, so nothing below shifts when it appears.
struct AppHeader: View {
    @EnvironmentObject private var updateChecker: UpdateChecker

    var body: some View {
        HStack(spacing: Layout.rowSpacing) {
            HStack(spacing: Layout.rowSpacing) {
                Image(nsImage: AppIcon.image(points: Layout.headerIconSize))
                Text("Nasib").font(.title2.bold())
            }
            .accessibilityElement(children: .combine)
            Spacer()
            if let release = updateChecker.available {
                Button("Update to \(release.version)") { updateChecker.openReleasePage() }
                    .buttonStyle(.borderedProminent)
                    .help(Tooltip.update)
            }
        }
    }
}

extension Engine.State {
    var title: String {
        switch self {
        case .unknown: "Checking…"
        case .down: "Not connected"
        case let .searching(location, protocolName): "Trying \(location) / \(protocolName)"
        case let .connected(location, protocolName): "\(location) / \(protocolName)"
        }
    }

    var color: Color {
        switch self {
        case .unknown: .secondary
        case .down: .red
        case .searching: .orange
        case .connected: .green
        }
    }

    /// Name of the glyph PNG pair in the bundle; unknown shows as searching until the first status read lands.
    var glyphName: String {
        switch self {
        case .unknown, .searching: "MenuBarSearching"
        case .down: "MenuBarDown"
        case .connected: "MenuBarConnected"
        }
    }
}

struct StatusHeader: View {
    @EnvironmentObject private var engine: Engine

    var body: some View {
        HStack(spacing: Layout.rowSpacing) {
            Circle()
                .fill(engine.state.color)
                .frame(width: Layout.statusDotSize, height: Layout.statusDotSize)
            Text(engine.state.title)
                .font(.headline)
                .lineLimit(1)
            Spacer()
            if engine.isBusy {
                ProgressView().controlSize(.small)
            }
        }
    }
}

struct ActionRow: View {
    @EnvironmentObject private var engine: Engine

    var body: some View {
        HStack(spacing: Layout.rowSpacing) {
            if engine.isSearching {
                Button("Stop") { engine.stop() }
                    .help(Tooltip.stopFind)
            } else {
                Button("Find Now") { engine.findNow() }
                    .disabled(engine.isBusy || engine.engineMissing)
                    .help(Tooltip.findNow)
            }
            if engine.isSurveying {
                Button("Stop Survey") { engine.stop() }
                    .help(Tooltip.stopSurvey)
            } else {
                Button("Start Survey") { engine.startSurvey() }
                    .disabled(engine.isBusy || engine.engineMissing)
                    .help(Tooltip.startSurvey)
            }
            Spacer()
            Button("Show Log") { NSWorkspace.shared.open(Engine.logFile) }
                .help(Tooltip.showLog)
        }
    }
}

struct SettingsRows: View {
    @EnvironmentObject private var engine: Engine
    @State private var launchAtLogin = LoginItem.isEnabled
    @State private var loginItemError: String?

    private func setLaunchAtLogin(_ enabled: Bool) {
        do {
            try LoginItem.set(enabled: enabled)
            loginItemError = nil
        } catch {
            loginItemError = error.localizedDescription
            launchAtLogin = LoginItem.isEnabled
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.rowSpacing) {
            Toggle("Auto-reconnect when the tunnel drops", isOn: $engine.autoReconnect)
                .help(Tooltip.autoReconnect)
            Toggle("Launch at login", isOn: $launchAtLogin)
                .onChange(of: launchAtLogin) { _, enabled in setLaunchAtLogin(enabled) }
                .help(Tooltip.launchAtLogin)
            if let loginItemError {
                Text(loginItemError).font(.caption).foregroundStyle(.red)
            }
        }
    }
}

extension OutputTone {
    var color: Color {
        switch self {
        case .primary: .primary
        case .success: .green
        case .muted: .secondary
        case .warning: .orange
        case .error: .red
        case .accent: .accentColor
        case .detail: Color(nsColor: .tertiaryLabelColor)
        case .phase: .teal
        }
    }
}

struct RecentOutput: View {
    @EnvironmentObject private var engine: Engine

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(Array(engine.recentLines.enumerated()), id: \.offset) { index, line in
                        Text(line)
                            .font(.caption.monospaced())
                            .foregroundStyle(EngineOutputParser.parse(line: line).tone.color)
                            .padding(.horizontal, Layout.logLinePadding)
                            .padding(.vertical, Layout.logLineVerticalPadding)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            // macOS's own alternating row color, as in Finder lists, so light and dark mode both fit.
                            .background(
                                Engine.isStriped(index: index, droppedLineCount: engine.droppedLineCount)
                                    ? Color(nsColor: NSColor.alternatingContentBackgroundColors[1])
                                    : Color.clear
                            )
                            .id(index)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(height: Layout.logHeight)
            .onChange(of: engine.recentLines.count) { _, count in
                proxy.scrollTo(count - 1)
            }
        }
    }
}

struct StatusTab: View {
    @EnvironmentObject private var engine: Engine

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.sectionSpacing) {
            AppHeader()
            StatusHeader()
            if engine.engineMissing {
                Text("Engine missing from the app bundle.").foregroundStyle(.red)
            }
            ActionRow()
            Divider()
            SettingsRows()
            Divider()
            RecentOutput()
        }
    }
}

struct ContentView: View {
    private enum Tab {
        case status
        case settings
    }

    @State private var tab = Tab.status

    var body: some View {
        TabView(selection: $tab) {
            // Both tabs pin to the top; the window takes the taller tab's height and would otherwise center the shorter one.
            StatusTab()
                .padding(Layout.padding)
                .frame(maxHeight: .infinity, alignment: .top)
                .tabItem { Text("Status") }
                .tag(Tab.status)
            SettingsView()
                .padding(Layout.padding)
                .frame(maxHeight: .infinity, alignment: .top)
                .tabItem { Text("Settings") }
                .tag(Tab.settings)
        }
        .padding(Layout.padding)
        .frame(width: tab == .settings ? Layout.settingsWindowWidth : Layout.windowWidth)
        // The window takes the content's own height, so a size saved by an older, taller layout can't stretch it.
        .fixedSize(horizontal: false, vertical: true)
    }
}

struct MenuBarMenu: View {
    @EnvironmentObject private var engine: Engine
    @EnvironmentObject private var updateChecker: UpdateChecker
    @Environment(\.openWindow) private var openWindow

    private func openMainWindow() {
        openWindow(id: NasibApp.mainWindowIdentifier)
        NSApp.activate(ignoringOtherApps: true)
    }

    var body: some View {
        // The header row also opens the window, so it reads as an enabled item rather than a greyed-out label.
        Button { openMainWindow() } label: {
            Label { Text("Nasib") } icon: { Image(nsImage: AppIcon.image(points: Layout.menuIconSize)) }
        }
        Text(engine.state.title)
        Divider()
        Button("Open Nasib") { openMainWindow() }
        if engine.isSearching || engine.isSurveying {
            Button("Stop Search") { engine.stop() }
        } else {
            Button("Find Now") { engine.findNow() }
                .disabled(engine.isBusy)
        }
        Divider()
        if let release = updateChecker.available {
            Button("Update to \(release.version)…") { updateChecker.openReleasePage() }
        }
        Button("About Nasib") { AppIcon.showAboutPanel() }
        Button("Quit Nasib") { NSApp.terminate(nil) }
    }
}
