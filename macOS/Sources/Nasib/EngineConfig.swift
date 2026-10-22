import Foundation

/// The overrides the engine script sources from ~/.ws-find/config: which cities to search and
/// which protocols to try, in order. Rendered as zsh array assignments.
struct EngineConfig: Equatable {
    static let allProtocols = ["wireguard", "ikev2", "tcp", "stealth", "wstunnel", "udp"]
    static let defaultCities = [
        "Istanbul", "Dubai", "Sofia", "Stockholm", "Amsterdam", "Frankfurt", "Zurich",
        "Toronto", "Montreal", "Winnipeg", "New York", "Chicago", "Atlanta",
        "Bratislava", "Helsinki", "Brussels",
    ]
    static let `default` = EngineConfig(cities: defaultCities, protocols: allProtocols)
    static let fileURL = Engine.stateDirectory.appendingPathComponent("config")

    private static let header = "# Written by Nasib from its Settings tab; the engine sources this file."
    private static let citiesKey = "CITIES"
    private static let protocolsKey = "PROTOCOL_ORDER"
    private static let singleQuote: Character = "'"
    private static let escapedSingleQuote = "'\\''"

    var cities: [String]
    var protocols: [String]

    /// Quotes one city for zsh: single quotes, with embedded single quotes closed, escaped, reopened.
    private static func quoted(_ value: String) -> String {
        let escaped = value.replacingOccurrences(of: String(singleQuote), with: escapedSingleQuote)
        return "\(singleQuote)\(escaped)\(singleQuote)"
    }

    /// Reads back the single-quoted items of a rendered cities array.
    private static func unquotedItems(_ body: String) -> [String] {
        var items: [String] = []
        var current = ""
        var insideQuotes = false
        var index = body.startIndex
        while index < body.endIndex {
            let character = body[index]
            if insideQuotes {
                if body[index...].hasPrefix(escapedSingleQuote) {
                    current.append(singleQuote)
                    index = body.index(index, offsetBy: escapedSingleQuote.count)
                    continue
                }
                if character == singleQuote {
                    insideQuotes = false
                    items.append(current)
                    current = ""
                } else {
                    current.append(character)
                }
            } else if character == singleQuote {
                insideQuotes = true
            }
            index = body.index(after: index)
        }
        return items
    }

    /// Returns the text between the parentheses of `KEY=( ... )`, or nil when the line is absent.
    private static func arrayBody(forKey key: String, in text: String) -> String? {
        for line in text.split(whereSeparator: \.isNewline) {
            let prefix = "\(key)=("
            guard line.hasPrefix(prefix), line.hasSuffix(")") else { continue }
            return String(line.dropFirst(prefix.count).dropLast())
        }
        return nil
    }

    func rendered() -> String {
        let cityList = cities.map(Self.quoted).joined(separator: " ")
        let protocolList = protocols.joined(separator: " ")
        return """
        \(Self.header)
        \(Self.citiesKey)=(\(cityList))
        \(Self.protocolsKey)=(\(protocolList))

        """
    }

    /// Parses a rendered file; nil when either array line is missing.
    static func parse(_ text: String) -> EngineConfig? {
        guard let citiesBody = arrayBody(forKey: citiesKey, in: text),
              let protocolsBody = arrayBody(forKey: protocolsKey, in: text) else { return nil }
        let protocols = protocolsBody.split(separator: " ").map(String.init).filter { allProtocols.contains($0) }
        return EngineConfig(cities: unquotedItems(citiesBody), protocols: protocols)
    }

    static func load() -> EngineConfig {
        guard let text = try? String(contentsOf: fileURL, encoding: .utf8), let config = parse(text) else {
            return .default
        }
        return config
    }

    func save() throws {
        try FileManager.default.createDirectory(at: Engine.stateDirectory, withIntermediateDirectories: true)
        try rendered().write(to: Self.fileURL, atomically: true, encoding: .utf8)
    }
}
