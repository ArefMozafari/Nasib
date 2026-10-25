import Foundation

/// Every datacenter the engine knows, grouped by Windscribe's region names, as `nasib-engine --catalog`
/// prints it. Settings lists it so users can pick and order their locations.
struct EngineCatalog: Decodable, Equatable {
    struct Entry: Decodable, Equatable, Hashable, Identifiable {
        let city: String
        let nickname: String
        /// True when the Windscribe log names it; settings from before locations carry these over.
        var logged = false
        var id: String { nickname }

        private enum CodingKeys: String, CodingKey {
            case city, nickname, logged
        }

        init(city: String, nickname: String, logged: Bool = false) {
            self.city = city
            self.nickname = nickname
            self.logged = logged
        }

        /// The engine leaves `logged` out unless it's true.
        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            city = try container.decode(String.self, forKey: .city)
            nickname = try container.decode(String.self, forKey: .nickname)
            logged = try container.decodeIfPresent(Bool.self, forKey: .logged) ?? false
        }

        /// The name Windscribe shows, such as "Toronto - The 6".
        var title: String { city.isEmpty ? nickname : "\(city) - \(nickname)" }
    }

    struct Region: Decodable, Equatable, Identifiable {
        let name: String
        let datacenters: [Entry]
        var id: String { name }
    }

    let regions: [Region]

    /// Every datacenter, in region order.
    var entries: [Entry] { regions.flatMap(\.datacenters) }

    /// Reads the one line `--catalog` prints; nil for anything else.
    static func decode(_ line: String) -> EngineCatalog? {
        try? JSONDecoder().decode(EngineCatalog.self, from: Data(line.utf8))
    }
}
