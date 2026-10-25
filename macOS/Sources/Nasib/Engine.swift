import Foundation

/// Drives the bundled nasib-engine and mirrors its progress for the UI.
@MainActor
final class Engine: ObservableObject {
    enum State: Equatable {
        case unknown
        case down
        case searching(location: String, protocolName: String)
        case connected(location: String, protocolName: String)
    }

    nonisolated static let stateDirectory = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".ws-find")
    nonisolated static let logFile = stateDirectory.appendingPathComponent("log.csv")

    private nonisolated static let engineName = "nasib-engine"
    private nonisolated static let statusArgument = "--status"
    private static let processPath = "/usr/local/bin:/usr/bin:/bin"
    private static let watchInterval: TimeInterval = 60
    /// Reading the CLI status costs no traffic, so it runs far more often than the blocked-site check.
    nonisolated static let statusInterval: TimeInterval = 3
    private static let recentLineLimit = 200
    private static let autoReconnectKey = "autoReconnect"
    private static let surveyArgument = "--survey"
    private static let watchArgument = "--watch"
    private static let connectedNotificationTitle = "Connected"
    private static let nothingWorkedNotificationTitle = "No working server"
    private static let nothingWorkedNotificationBody = "Every datacenter and protocol failed."
    private static let tunnelDownNotificationTitle = "Tunnel down"
    private static let tunnelDownNotificationBody = "Searching for a working server…"

    @Published private(set) var state: State = .unknown
    @Published private(set) var isBusy = false
    @Published private(set) var isSurveying = false
    /// True during a find, from Find Now or from an auto-reconnect check that found the tunnel down.
    /// A healthy check is busy for a second or two but never searching, so the button doesn't flicker.
    @Published private(set) var isSearching = false
    @Published private(set) var recentLines: [String] = []
    @Published private(set) var engineMissing = false
    @Published var autoReconnect: Bool {
        didSet { UserDefaults.standard.set(autoReconnect, forKey: Self.autoReconnectKey) }
    }

    private var process: Process?
    /// Held open for the whole run: the engine treats its standard input closing as a stop.
    private var processInput: Pipe?
    private var watchTimer: Timer?
    private var statusTimer: Timer?
    private var isRefreshing = false

    init() {
        autoReconnect = UserDefaults.standard.object(forKey: Self.autoReconnectKey) as? Bool ?? true
        engineMissing = Self.engineURL == nil
        Notifier.requestPermission()
        startTimers()
        refreshStatus()
    }

    /// The engine binary, next to the app's own executable in Contents/MacOS.
    private nonisolated static var engineURL: URL? {
        Bundle.main.url(forAuxiliaryExecutable: engineName)
    }

    private func append(line: String) {
        recentLines.append(line)
        if recentLines.count > Self.recentLineLimit {
            recentLines.removeFirst(recentLines.count - Self.recentLineLimit)
        }
    }

    private func handle(line: String) {
        append(line: line)
        switch EngineOutputParser.parse(line: line) {
        case let .trying(location, protocolName):
            state = .searching(location: location, protocolName: protocolName)
        case let .connected(location, protocolName):
            state = .connected(location: location, protocolName: protocolName)
            Notifier.post(title: Self.connectedNotificationTitle, body: "\(location) / \(protocolName)")
        case .nothingWorked:
            state = .down
            Notifier.post(title: Self.nothingWorkedNotificationTitle, body: Self.nothingWorkedNotificationBody)
        case .tunnelDown:
            state = .down
            isSearching = true
            Notifier.post(title: Self.tunnelDownNotificationTitle, body: Self.tunnelDownNotificationBody)
        case .cliMissing:
            state = .down
        case .failed, .searchStopped, .surveyStopped, .surveyFinished, .other:
            break
        }
    }

    private func processDidExit() {
        process = nil
        processInput = nil
        isBusy = false
        isSurveying = false
        isSearching = false
        refreshStatus()
    }

    /// Runs the engine with the given arguments; refuses while another run is in progress.
    private func run(arguments: [String]) {
        guard process == nil, let engineURL = Self.engineURL else { return }
        let process = Process()
        process.executableURL = engineURL
        process.arguments = arguments
        process.environment = [
            "PATH": Self.processPath,
            "HOME": FileManager.default.homeDirectoryForCurrentUser.path,
        ]
        let input = Pipe()
        process.standardInput = input
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe
        pipe.fileHandleForReading.readabilityHandler = { handle in
            let text = String(decoding: handle.availableData, as: UTF8.self)
            let lines = text.split(whereSeparator: \.isNewline).map(String.init)
            Task { @MainActor in lines.forEach { self.handle(line: $0) } }
        }
        process.terminationHandler = { _ in
            pipe.fileHandleForReading.readabilityHandler = nil
            Task { @MainActor in self.processDidExit() }
        }
        do {
            try process.run()
            self.process = process
            processInput = input
            isBusy = true
        } catch {
            append(line: "could not start engine: \(error.localizedDescription)")
        }
    }

    /// Runs the script's watch mode, which exits at once when the tunnel is healthy.
    private func watchTick() {
        guard autoReconnect, process == nil else { return }
        run(arguments: [Self.watchArgument])
    }

    private func startTimers() {
        watchTimer = Timer.scheduledTimer(withTimeInterval: Self.watchInterval, repeats: true) { [weak self] _ in
            Task { @MainActor in self?.watchTick() }
        }
        statusTimer = Timer.scheduledTimer(withTimeInterval: Self.statusInterval, repeats: true) { [weak self] _ in
            Task { @MainActor in self?.refreshStatus() }
        }
    }

    /// Runs `nasib-engine --status` off the main thread and returns the line it prints, empty when the
    /// engine can't run.
    private nonisolated static func readEngineStatus() -> String {
        guard let engineURL else { return "" }
        let process = Process()
        process.executableURL = engineURL
        process.arguments = [statusArgument]
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = FileHandle.nullDevice
        guard (try? process.run()) != nil else { return "" }
        let output = String(decoding: pipe.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
        process.waitUntilExit()
        return output.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    /// True when a tunnel the app showed as connected has gone down. A stopped search or a tunnel that was
    /// never up is not a drop, so it doesn't start a reconnect the user didn't ask for.
    nonisolated static func tunnelDropped(from previous: State, to current: State) -> Bool {
        guard case .connected = previous else { return false }
        return current == .down
    }

    /// Reads the current tunnel from the CLI without touching it, and starts the watch at once when
    /// auto-reconnect is on and the tunnel just dropped, instead of waiting for the next minute's check.
    func refreshStatus() {
        guard !isRefreshing, process == nil else { return }
        isRefreshing = true
        Task {
            let line = await Task.detached { Self.readEngineStatus() }.value
            isRefreshing = false
            guard !isBusy else { return }
            let previous = state
            if let status = EngineOutputParser.parseStatus(line: line) {
                state = .connected(location: status.location, protocolName: status.protocolName)
            } else {
                state = .down
            }
            if Self.tunnelDropped(from: previous, to: state) {
                watchTick()
            }
        }
    }

    func findNow() {
        guard process == nil else { return }
        isSearching = true
        run(arguments: [])
    }

    func startSurvey() {
        guard process == nil else { return }
        isSurveying = true
        run(arguments: [Self.surveyArgument])
    }

    /// Sends Ctrl-C. A stopped find disconnects; a stopped survey reconnects the last server that worked.
    func stop() {
        process?.interrupt()
    }
}
