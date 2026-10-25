import XCTest
@testable import Nasib

final class EngineCatalogTests: XCTestCase {
    func testShouldDecodeTheCatalogLine() {
        let line = #"{"regions":[{"name":"Canada East","datacenters":[{"city":"Toronto","nickname":"The 6"}]},{"name":"Other","datacenters":[{"city":"Oslo","nickname":"Fjord"}]}]}"#

        let catalog = EngineCatalog.decode(line)

        XCTAssertEqual(catalog?.regions.map(\.name), ["Canada East", "Other"])
        XCTAssertEqual(catalog?.regions.first?.datacenters.first, EngineCatalog.Entry(city: "Toronto", nickname: "The 6"))
    }

    func testShouldReadWhichDatacentersTheLogKnows() {
        let line = #"{"regions":[{"name":"Canada East","datacenters":[{"city":"Toronto","nickname":"Comfort Zone","logged":true},{"city":"Toronto","nickname":"The 6"}]}]}"#

        let entries = EngineCatalog.decode(line)?.entries

        XCTAssertEqual(entries?.map(\.logged), [true, false])
    }

    func testShouldReturnNilForAnythingElse() {
        XCTAssertNil(EngineCatalog.decode("windscribe-cli missing: /usr/local/bin/windscribe-cli"))
        XCTAssertNil(EngineCatalog.decode(""))
    }

    func testShouldListEveryEntryInOrder() {
        let catalog = EngineCatalog(regions: [
            .init(name: "A", datacenters: [.init(city: "Toronto", nickname: "The 6")]),
            .init(name: "B", datacenters: [.init(city: "Dubai", nickname: "Khalifa")]),
        ])

        XCTAssertEqual(catalog.entries.map(\.nickname), ["The 6", "Khalifa"])
    }
}
