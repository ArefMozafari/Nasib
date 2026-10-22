import Foundation

/// One line of the engine script's output, turned into something the UI can react to.
enum EngineEvent: Equatable {
    case trying(location: String, protocolName: String)
    case connected(location: String, protocolName: String)
    case nothingWorked
    case surveyFinished
    case tunnelDown
    case other(String)
}

/// What `windscribe-cli status` currently reports.
struct TunnelStatus: Equatable {
    let location: String
    let protocolName: String
}

enum EngineOutputParser {
    private static let tryingPrefix = "trying: "
    private static let connectedPrefix = "connected: "
    private static let nothingWorkedLine = "nothing worked"
    private static let surveyFinishedLine = "survey finished"
    private static let tunnelDownLine = "tunnel down"
    private static let pairSeparator = " / "
    private static let connectedStatePrefix = "Connect state: Connected: "
    private static let protocolPrefix = "Protocol: "
    private static let protocolPortSeparator: Character = ":"

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
        if line == nothingWorkedLine { return .nothingWorked }
        if line == surveyFinishedLine { return .surveyFinished }
        if line == tunnelDownLine { return .tunnelDown }
        return .other(line)
    }

    /// Returns nil when the CLI reports anything other than a fully connected tunnel.
    static func parseStatus(output: String) -> TunnelStatus? {
        var location: String?
        var protocolName: String?
        for line in output.split(whereSeparator: \.isNewline) {
            if line.hasPrefix(connectedStatePrefix) {
                location = String(line.dropFirst(connectedStatePrefix.count))
            } else if line.hasPrefix(protocolPrefix) {
                let value = line.dropFirst(protocolPrefix.count)
                protocolName = String(value.split(separator: protocolPortSeparator).first ?? value)
            }
        }
        guard let location, let protocolName else { return nil }
        return TunnelStatus(location: location, protocolName: protocolName)
    }
}
