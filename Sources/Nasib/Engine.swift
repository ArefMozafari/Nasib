import Foundation

/// Drives the bundled ws-find.sh script and mirrors its progress for the UI.
@MainActor
final class Engine: ObservableObject {
    enum State: Equatable {
        case unknown
        case down
        case searching(location: String, protocolName: String)
        case connected(location: String, protocolName: String)
    }

    static let stateDirectory = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".ws-find")
    static let logFile = stateDirectory.appendingPathComponent("log.csv")

    private static let scriptName = "ws-find"
    private static let scriptExtension = "sh"
    private static let shellPath = "/bin/zsh"
    private nonisolated static let windscribeCliPath = "/usr/local/bin/windscribe-cli"
    private static let processPath = "/usr/local/bin:/usr/bin:/bin"
    private static let watchInterval: TimeInterval = 60
    private static let recentLineLimit = 200
    private static let autoReconnectKey = "autoReconnect"
    private static let surveyArgument = "--survey"
    private static let watchArgument = "--watch"

    @Published private(set) var state: State = .unknown
    @Published private(set) var isBusy = false
    @Published private(set) var isSurveying = false
    @Published private(set) var recentLines: [String] = []
    @Published private(set) var engineMissing = false
    @Published var autoReconnect: Bool {
        didSet { UserDefaults.standard.set(autoReconnect, forKey: Self.autoReconnectKey) }
    }

    private var process: Process?
    private var watchTimer: Timer?

    init() {
        autoReconnect = UserDefaults.standard.object(forKey: Self.autoReconnectKey) as? Bool ?? true
        engineMissing = Self.scriptURL == nil
        startWatchTimer()
        refreshStatus()
    }

    private static var scriptURL: URL? {
        Bundle.main.url(forResource: scriptName, withExtension: scriptExtension)
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
        case .nothingWorked:
            state = .down
        case .surveyFinished, .other:
            break
        }
    }

    private func processDidExit() {
        process = nil
        isBusy = false
        isSurveying = false
        refreshStatus()
    }

    /// Runs the script with the given arguments; refuses while another run is in progress.
    private func run(arguments: [String]) {
        guard process == nil, let scriptURL = Self.scriptURL else { return }
        let process = Process()
        process.executableURL = URL(fileURLWithPath: Self.shellPath)
        process.arguments = [scriptURL.path] + arguments
        process.environment = ["PATH": Self.processPath, "HOME": FileManager.default.homeDirectoryForCurrentUser.path]
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
            isBusy = true
        } catch {
            append(line: "could not start engine: \(error.localizedDescription)")
        }
    }

    private func startWatchTimer() {
        watchTimer = Timer.scheduledTimer(withTimeInterval: Self.watchInterval, repeats: true) { [weak self] _ in
            Task { @MainActor in self?.watchTick() }
        }
    }

    /// Runs the script's watch mode, which exits at once when the tunnel is healthy.
    private func watchTick() {
        guard autoReconnect, process == nil else { return }
        run(arguments: [Self.watchArgument])
    }

    /// Runs `windscribe-cli status` off the main thread and returns its output, empty when the CLI is missing.
    private nonisolated static func readCliStatus() -> String {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: windscribeCliPath)
        process.arguments = ["status"]
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = FileHandle.nullDevice
        guard (try? process.run()) != nil else { return "" }
        let output = String(decoding: pipe.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
        process.waitUntilExit()
        return output
    }

    /// Reads the current tunnel from the CLI without touching it.
    func refreshStatus() {
        Task {
            let output = await Task.detached { Self.readCliStatus() }.value
            guard !isBusy else { return }
            if let status = EngineOutputParser.parseStatus(output: output) {
                state = .connected(location: status.location, protocolName: status.protocolName)
            } else {
                state = .down
            }
        }
    }

    func findNow() {
        run(arguments: [])
    }

    func startSurvey() {
        guard process == nil else { return }
        isSurveying = true
        run(arguments: [Self.surveyArgument])
    }

    /// Sends Ctrl-C; the script reconnects the last good server before exiting.
    func stopSurvey() {
        process?.interrupt()
    }
}
