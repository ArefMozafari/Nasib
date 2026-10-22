import SwiftUI

/// Sizing and spacing tokens for the main window and the menu.
enum Layout {
    static let windowWidth: CGFloat = 360
    static let sectionSpacing: CGFloat = 16
    static let rowSpacing: CGFloat = 8
    static let padding: CGFloat = 20
    static let statusDotSize: CGFloat = 10
    static let logHeight: CGFloat = 140
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

    var symbolName: String {
        switch self {
        case .unknown: "questionmark.circle"
        case .down: "xmark.shield"
        case .searching: "arrow.triangle.2.circlepath"
        case .connected: "checkmark.shield.fill"
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
            Button("Find Now") { engine.findNow() }
                .disabled(engine.isBusy || engine.engineMissing)
            if engine.isSurveying {
                Button("Stop Survey") { engine.stopSurvey() }
            } else {
                Button("Start Survey") { engine.startSurvey() }
                    .disabled(engine.isBusy || engine.engineMissing)
            }
            Spacer()
            Button("Show Log") { NSWorkspace.shared.open(Engine.logFile) }
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
            Toggle("Launch at login", isOn: $launchAtLogin)
                .onChange(of: launchAtLogin) { _, enabled in setLaunchAtLogin(enabled) }
            if let loginItemError {
                Text(loginItemError).font(.caption).foregroundStyle(.red)
            }
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
                        Text(line).font(.caption.monospaced()).id(index)
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

struct ContentView: View {
    @EnvironmentObject private var engine: Engine

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.sectionSpacing) {
            StatusHeader()
            if engine.engineMissing {
                Text("Engine script missing from the app bundle.").foregroundStyle(.red)
            }
            ActionRow()
            Divider()
            SettingsRows()
            Divider()
            RecentOutput()
        }
        .padding(Layout.padding)
        .frame(width: Layout.windowWidth)
    }
}

struct MenuBarMenu: View {
    @EnvironmentObject private var engine: Engine
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Text(engine.state.title)
        Divider()
        Button("Open Nasib") {
            openWindow(id: NasibApp.mainWindowIdentifier)
            NSApp.activate(ignoringOtherApps: true)
        }
        Button("Find Now") { engine.findNow() }
            .disabled(engine.isBusy)
        Divider()
        Button("Quit Nasib") { NSApp.terminate(nil) }
    }
}
