import XCTest
@testable import Nasib

final class EngineConfigTests: XCTestCase {
    func testShouldRenderZshArrayAssignments() {
        let config = EngineConfig(cities: ["Istanbul", "New York"], protocols: ["ikev2", "tcp"])
        let rendered = config.rendered()
        XCTAssertTrue(rendered.contains("CITIES=('Istanbul' 'New York')"))
        XCTAssertTrue(rendered.contains("PROTOCOL_ORDER=(ikev2 tcp)"))
    }

    func testShouldRoundTripCityWithApostrophe() {
        let config = EngineConfig(cities: ["Saint John's", "Zurich"], protocols: ["wireguard"])
        XCTAssertEqual(EngineConfig.parse(config.rendered()), config)
    }

    func testShouldRenderEmptyCitiesAsEmptyArray() {
        let config = EngineConfig(cities: [], protocols: ["udp"])
        XCTAssertTrue(config.rendered().contains("CITIES=()"))
        XCTAssertEqual(EngineConfig.parse(config.rendered()), config)
    }

    func testShouldKeepProtocolOrderAndDropUnknownNames() {
        let text = "CITIES=('Dubai')\nPROTOCOL_ORDER=(tcp bogus wireguard)\n"
        XCTAssertEqual(EngineConfig.parse(text)?.protocols, ["tcp", "wireguard"])
    }

    func testShouldReturnNilWhenAnArrayLineIsMissing() {
        XCTAssertNil(EngineConfig.parse("CITIES=('Dubai')\n"))
        XCTAssertNil(EngineConfig.parse("# nothing here\n"))
    }

    func testShouldDefaultToEveryProtocol() {
        XCTAssertEqual(EngineConfig.default.protocols, EngineConfig.allProtocols)
    }
}
