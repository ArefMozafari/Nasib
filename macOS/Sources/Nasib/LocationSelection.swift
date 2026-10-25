import Foundation

/// The locations checked in Settings, in the user's order. It starts from what the engine would search
/// for the saved settings, so opening Settings never changes the search by itself.
struct LocationSelection: Equatable {
    private(set) var ordered: [EngineCatalog.Entry]
    private let catalog: EngineCatalog

    var nicknames: [String] { ordered.map(\.nickname) }

    /// An empty order would mean every datacenter to the engine, so Settings doesn't save one.
    var canSave: Bool { !ordered.isEmpty }

    private static func includes(city: String, in cities: [String]) -> Bool {
        cities.isEmpty || cities.contains { $0.lowercased() == city.lowercased() }
    }

    /// What the engine searches without saved locations: the default locations in the given cities,
    /// then every other datacenter the Windscribe log knows in those cities.
    private static func carriedOver(cities: [String], catalog: EngineCatalog) -> [EngineCatalog.Entry] {
        let defaults = EngineConfig.defaultLocations
            .compactMap { nickname in catalog.entries.first { $0.nickname == nickname } }
            .filter { includes(city: $0.city, in: cities) }
        let logged = catalog.entries.filter {
            $0.logged && includes(city: $0.city, in: cities) && !defaults.contains($0)
        }
        return defaults + logged
    }

    /// Nil when only cities are saved and the catalog couldn't be read: the cities can't be turned into
    /// locations then, and Settings must leave them as they are.
    init?(saved: EngineConfig, catalog: EngineCatalog) {
        self.catalog = catalog
        switch saved.locations {
        case .some(let locations) where locations.isEmpty:
            ordered = catalog.entries
        case .some(let locations):
            // Without a catalog, the saved order stays editable under the nicknames alone.
            ordered = locations.compactMap { nickname in
                catalog.entries.first { $0.nickname == nickname }
                    ?? (catalog.regions.isEmpty ? EngineCatalog.Entry(city: "", nickname: nickname) : nil)
            }
        case .none:
            guard !catalog.regions.isEmpty else { return nil }
            ordered = Self.carriedOver(cities: saved.cities, catalog: catalog)
        }
    }

    func isSelected(_ nickname: String) -> Bool {
        ordered.contains { $0.nickname == nickname }
    }

    /// Appends the datacenter to the order, or removes it when it's already there.
    mutating func toggle(_ nickname: String) {
        if let index = ordered.firstIndex(where: { $0.nickname == nickname }) {
            ordered.remove(at: index)
        } else if let entry = catalog.entries.first(where: { $0.nickname == nickname }) {
            ordered.append(entry)
        }
    }

    func areAllSelected(_ nicknames: [String]) -> Bool {
        nicknames.allSatisfy(isSelected)
    }

    /// Appends the given datacenters that aren't checked yet, in the order given.
    mutating func select(_ nicknames: [String]) {
        for nickname in nicknames where !isSelected(nickname) {
            toggle(nickname)
        }
    }

    mutating func deselect(_ nicknames: [String]) {
        ordered.removeAll { nicknames.contains($0.nickname) }
    }

    mutating func deselectAll() {
        ordered.removeAll()
    }

    mutating func move(from source: IndexSet, to destination: Int) {
        ordered.move(fromOffsets: source, toOffset: destination)
    }

    /// Restores what an untouched install searches.
    mutating func resetToDefaults() {
        ordered = Self.carriedOver(cities: EngineConfig.defaultCities, catalog: catalog)
    }
}
