import XCTest
@testable import Nasib

final class LocationSelectionTests: XCTestCase {
    private let catalog = EngineCatalog(regions: [
        .init(name: "Turkey", datacenters: [.init(city: "Istanbul", nickname: "Lygos")]),
        .init(name: "United Arab Emirates", datacenters: [.init(city: "Dubai", nickname: "Khalifa")]),
        .init(name: "Canada East", datacenters: [
            .init(city: "Toronto", nickname: "The 6"),
            .init(city: "Toronto", nickname: "Comfort Zone", logged: true),
            .init(city: "Toronto", nickname: "Peak"),
        ]),
        .init(name: "Other", datacenters: [.init(city: "Toronto", nickname: "Drake", logged: true)]),
    ])

    private func saved(locations: [String]? = nil, cities: [String] = EngineConfig.defaultCities) -> EngineConfig {
        var config = EngineConfig.default
        config.locations = locations
        config.cities = cities
        return config
    }

    func testShouldStartFromSavedLocationsInOrder() {
        let selection = LocationSelection(saved: saved(locations: ["The 6", "Lygos"]), catalog: catalog)
        XCTAssertEqual(selection?.nicknames, ["The 6", "Lygos"])
    }

    func testShouldStartFromDefaultLocationsThenLoggedOnesInSavedCities() {
        let selection = LocationSelection(saved: saved(cities: ["Toronto", "Dubai"]), catalog: catalog)
        XCTAssertEqual(selection?.nicknames, ["Khalifa", "The 6", "Comfort Zone", "Drake"])
    }

    func testShouldStartFromEveryLocationWhenTheSavedListIsEmpty() {
        let selection = LocationSelection(saved: saved(locations: []), catalog: catalog)
        XCTAssertEqual(selection?.nicknames, ["Lygos", "Khalifa", "The 6", "Comfort Zone", "Peak", "Drake"])
    }

    func testShouldKeepSavedLocationsWhenTheCatalogCannotBeRead() {
        let selection = LocationSelection(saved: saved(locations: ["The 6", "Lygos"]), catalog: EngineCatalog(regions: []))
        XCTAssertEqual(selection?.nicknames, ["The 6", "Lygos"])
    }

    // Without the catalog, the old cities can't be turned into locations, so Settings must not write any.
    func testShouldNotStartWithoutACatalogWhenOnlyCitiesAreSaved() {
        XCTAssertNil(LocationSelection(saved: saved(cities: ["Toronto"]), catalog: EngineCatalog(regions: [])))
    }

    func testShouldAppendWhenToggledOn() {
        var selection = LocationSelection(saved: saved(locations: ["The 6"]), catalog: catalog)!
        selection.toggle("Peak")
        XCTAssertEqual(selection.nicknames, ["The 6", "Peak"])
        XCTAssertTrue(selection.isSelected("Peak"))
    }

    func testShouldRemoveWhenToggledOff() {
        var selection = LocationSelection(saved: saved(locations: ["The 6", "Lygos"]), catalog: catalog)!
        selection.toggle("The 6")
        XCTAssertEqual(selection.nicknames, ["Lygos"])
        XCTAssertFalse(selection.isSelected("The 6"))
    }

    func testShouldMoveARow() {
        var selection = LocationSelection(saved: saved(locations: ["The 6", "Lygos", "Khalifa"]), catalog: catalog)!
        selection.move(from: IndexSet(integer: 2), to: 0)
        XCTAssertEqual(selection.nicknames, ["Khalifa", "The 6", "Lygos"])
    }

    func testShouldResetToWhatAnUntouchedInstallSearches() {
        var selection = LocationSelection(saved: saved(locations: ["Peak"]), catalog: catalog)!
        selection.resetToDefaults()
        XCTAssertEqual(selection.nicknames, ["Lygos", "Khalifa", "The 6", "Comfort Zone", "Drake"])
    }

    func testShouldSelectAllGivenLocationsAfterTheOrderInTheirOrder() {
        var selection = LocationSelection(saved: saved(locations: ["Comfort Zone"]), catalog: catalog)!
        selection.select(["The 6", "Comfort Zone", "Peak"])
        XCTAssertEqual(selection.nicknames, ["Comfort Zone", "The 6", "Peak"])
        XCTAssertTrue(selection.areAllSelected(["The 6", "Comfort Zone", "Peak"]))
    }

    func testShouldDeselectOnlyTheGivenLocations() {
        var selection = LocationSelection(saved: saved(locations: ["Lygos", "The 6", "Peak"]), catalog: catalog)!
        selection.deselect(["The 6", "Peak", "Drake"])
        XCTAssertEqual(selection.nicknames, ["Lygos"])
        XCTAssertFalse(selection.areAllSelected(["Lygos", "The 6"]))
    }

    func testShouldDeselectEverything() {
        var selection = LocationSelection(saved: saved(locations: ["Lygos", "The 6"]), catalog: catalog)!
        selection.deselectAll()
        XCTAssertTrue(selection.nicknames.isEmpty)
        XCTAssertFalse(selection.canSave)
    }

    func testShouldNotSaveAnEmptyOrder() {
        var selection = LocationSelection(saved: saved(locations: ["The 6"]), catalog: catalog)!
        XCTAssertTrue(selection.canSave)
        selection.toggle("The 6")
        XCTAssertFalse(selection.canSave)
    }
}
