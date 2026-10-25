import XCTest
@testable import Nasib

final class SettingsViewTests: XCTestCase {
    func testShouldRefuseToSaveWithoutALocation() {
        XCTAssertEqual(SettingsView.saveProblem(locationCount: 0, protocolCount: 3), "Check at least one location.")
    }

    func testShouldRefuseToSaveWithoutAProtocol() {
        XCTAssertEqual(SettingsView.saveProblem(locationCount: 2, protocolCount: 0), "Check at least one protocol.")
    }

    func testShouldSaveWhenLocationsAreNotLoaded() {
        XCTAssertNil(SettingsView.saveProblem(locationCount: nil, protocolCount: 1))
    }

    func testShouldSaveWithALocationAndAProtocol() {
        XCTAssertNil(SettingsView.saveProblem(locationCount: 1, protocolCount: 1))
    }
}
