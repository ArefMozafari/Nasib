import Foundation

/// How an output line is colored. A failed attempt is routine while searching, so only
/// "nothing worked" gets the error tone.
enum OutputTone: Equatable {
    case primary
    case success
    case muted
    case warning
    case error
    case accent
    case detail
    /// The header that starts each stage of a search.
    case phase
}

/// One line of the engine's output, turned into something the UI can react to.
enum EngineEvent: Equatable {
    case trying(location: String, protocolName: String)
    case connected(location: String, protocolName: String)
    case failed(location: String, protocolName: String)
    case nothingWorked
    case surveyFinished
    case surveyStopped
    case searchStopped
    case tunnelDown
    case cliMissing(path: String)
    case phase(String)
    case other(String)

    var tone: OutputTone {
        switch self {
        case .trying: .primary
        case .connected: .success
        case .failed: .muted
        case .tunnelDown: .warning
        case .nothingWorked, .cliMissing: .error
        case .searchStopped, .surveyStopped, .surveyFinished: .accent
        case .phase: .phase
        case .other: .detail
        }
    }
}

/// What `nasib-engine --status` reports: the CLI's own location and protocol names.
struct TunnelStatus: Equatable {
    let location: String
    let protocolName: String
}

enum EngineOutputParser {
    private static let tryingPrefix = "trying: "
    private static let connectedPrefix = "connected: "
    private static let failedPrefix = "failed: "
    private static let nothingWorkedLine = "nothing worked"
    private static let surveyFinishedLine = "survey finished"
    private static let surveyStoppedLine = "survey stopped"
    private static let searchStoppedLine = "search stopped"
    private static let tunnelDownLine = "tunnel down"
    private static let cliMissingPrefix = "windscribe-cli missing: "
    private static let phasePrefix = "phase: "
    private static let pairSeparator = " / "

    /// Splits "Location / protocol" into its two parts.
    private static func splitPair(_ text: String) -> (location: String, protocolName: String)? {
        guard let range = text.range(of: pairSeparator, options: .backwards) else { return nil }
        return (String(text[..<range.lowerBound]), String(text[range.upperBound...]))
    }

    static func parse(line: String) -> EngineEvent {
        if line.hasPrefix(tryingPrefix), let pair = splitPair(String(line.dropFirst(tryingPrefix.count))) {
            return .trying(location: pair.location, protocolName: pair.protocolName)
        }
        if line.hasPrefix(connectedPrefix), let pair = splitPair(String(line.dropFirst(connectedPrefix.count))) {
            return .connected(location: pair.location, protocolName: pair.protocolName)
        }
        if line.hasPrefix(failedPrefix), let pair = splitPair(String(line.dropFirst(failedPrefix.count))) {
            return .failed(location: pair.location, protocolName: pair.protocolName)
        }
        if line == nothingWorkedLine { return .nothingWorked }
        if line == surveyFinishedLine { return .surveyFinished }
        if line == surveyStoppedLine { return .surveyStopped }
        if line == searchStoppedLine { return .searchStopped }
        if line == tunnelDownLine { return .tunnelDown }
        if line.hasPrefix(cliMissingPrefix) { return .cliMissing(path: String(line.dropFirst(cliMissingPrefix.count))) }
        if line.hasPrefix(phasePrefix) { return .phase(String(line.dropFirst(phasePrefix.count))) }
        return .other(line)
    }

    /// Reads the one line `nasib-engine --status` prints. Returns nil for `down`, a missing CLI, or
    /// anything else that isn't a fully connected tunnel.
    static func parseStatus(line: String) -> TunnelStatus? {
        guard line.hasPrefix(connectedPrefix), let pair = splitPair(String(line.dropFirst(connectedPrefix.count))) else {
            return nil
        }
        return TunnelStatus(location: pair.location, protocolName: pair.protocolName)
    }
}
