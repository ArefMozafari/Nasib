import Foundation

/// Which cities to search and which protocols to try, in order, saved in ~/.ws-find/config.json for the
/// engine to read. Every Nasib app writes the same format.
struct EngineConfig: Equatable, Codable {
    static let allProtocols = ["wireguard", "ikev2", "tcp", "stealth", "wstunnel", "udp"]
    static let defaultCities = [
        "Istanbul", "Dubai", "Sofia", "Stockholm", "Amsterdam", "Frankfurt", "Zurich",
        "Toronto", "Montreal", "Winnipeg", "New York", "Chicago", "Atlanta",
        "Bratislava", "Helsinki", "Brussels",
    ]
    static let `default` = EngineConfig(cities: defaultCities, protocols: allProtocols)

    private static let fileName = "config.json"
    /// The zsh-syntax file the old engine script sourced, read once to carry its settings over.
    private static let legacyFileName = "config"
    private static let citiesKey = "CITIES"
    private static let protocolsKey = "PROTOCOL_ORDER"
    private static let singleQuote: Character = "'"
    private static let escapedSingleQuote = "'\\''"

    var cities: [String]
    var protocols: [String]

    private enum CodingKeys: String, CodingKey {
        case cities
        case protocols = "protocolOrder"
    }

    /// Keeps only protocols the engine knows, each once, in the order given.
    private static func knownProtocols(_ protocols: [String]) -> [String] {
        var known: [String] = []
        for name in protocols where allProtocols.contains(name) && !known.contains(name) {
            known.append(name)
        }
        return known
    }

    /// Reads back the single-quoted items of a legacy cities array.
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

    /// Parses the legacy zsh-syntax file; nil when either array line is missing.
    static func parseLegacy(_ text: String) -> EngineConfig? {
        guard let citiesBody = arrayBody(forKey: citiesKey, in: text),
              let protocolsBody = arrayBody(forKey: protocolsKey, in: text) else { return nil }
        let protocols = knownProtocols(protocolsBody.split(separator: " ").map(String.init))
        return EngineConfig(cities: unquotedItems(citiesBody), protocols: protocols)
    }

    /// Reads the saved file, or returns the defaults when it is missing or unreadable.
    static func load(from directory: URL = Engine.stateDirectory) -> EngineConfig {
        guard let data = try? Data(contentsOf: directory.appendingPathComponent(fileName)),
              let stored = try? JSONDecoder().decode(EngineConfig.self, from: data) else {
            return .default
        }
        return EngineConfig(cities: stored.cities, protocols: knownProtocols(stored.protocols))
    }

    func save(to directory: URL = Engine.stateDirectory) throws {
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .withoutEscapingSlashes]
        try encoder.encode(self).write(to: directory.appendingPathComponent(Self.fileName), options: .atomic)
    }

    /// Carries the settings saved by older versions over to config.json, once. A legacy file that can't be
    /// read is left where it is, so nothing the user saved is lost.
    static func migrateLegacyFileIfNeeded(in directory: URL = Engine.stateDirectory) {
        let fileManager = FileManager.default
        let jsonFile = directory.appendingPathComponent(fileName)
        let legacyFile = directory.appendingPathComponent(legacyFileName)
        guard !fileManager.fileExists(atPath: jsonFile.path),
              let text = try? String(contentsOf: legacyFile, encoding: .utf8),
              let config = parseLegacy(text),
              (try? config.save(to: directory)) != nil else { return }
        try? fileManager.removeItem(at: legacyFile)
    }
}
