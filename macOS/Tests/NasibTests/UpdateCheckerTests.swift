import XCTest
@testable import Nasib

final class UpdateCheckerTests: XCTestCase {
    private static let releasePage = "https://github.com/ArefMozafari/Nasib/releases/tag/v0.3.0"

    func testShouldTreatHigherVersionAsNewer() {
        XCTAssertTrue(UpdateChecker.isNewer("0.3.0", than: "0.2.0"))
        XCTAssertTrue(UpdateChecker.isNewer("1.0.0", than: "0.9.9"))
    }

    func testShouldCompareComponentsAsNumbersNotText() {
        XCTAssertTrue(UpdateChecker.isNewer("0.10.0", than: "0.9.0"))
    }

    func testShouldNotTreatSameOrOlderVersionAsNewer() {
        XCTAssertFalse(UpdateChecker.isNewer("0.2.0", than: "0.2.0"))
        XCTAssertFalse(UpdateChecker.isNewer("0.1.1", than: "0.2.0"))
    }

    func testShouldTreatMissingComponentsAsZero() {
        XCTAssertFalse(UpdateChecker.isNewer("0.2", than: "0.2.0"))
        XCTAssertTrue(UpdateChecker.isNewer("0.2.1", than: "0.2"))
    }

    func testShouldParseTagWithoutPrefixAndPageFromLatestRelease() {
        let json = #"{"tag_name": "v0.3.0", "html_url": "\#(Self.releasePage)", "name": "Nasib 0.3.0"}"#
        let release = UpdateChecker.parseRelease(Data(json.utf8))
        XCTAssertEqual(release, UpdateChecker.Release(version: "0.3.0", pageURL: URL(string: Self.releasePage)!))
    }

    func testShouldReturnNilForResponseWithoutRelease() {
        XCTAssertNil(UpdateChecker.parseRelease(Data(#"{"message": "Not Found"}"#.utf8)))
        XCTAssertNil(UpdateChecker.parseRelease(Data("not json".utf8)))
    }
}
