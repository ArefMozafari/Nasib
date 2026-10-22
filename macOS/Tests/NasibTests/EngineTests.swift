import XCTest
@testable import Nasib

final class EngineTests: XCTestCase {
    private let connected = Engine.State.connected(location: "Zurich - Altstadt", protocolName: "IKEv2")
    private let searching = Engine.State.searching(location: "Altstadt", protocolName: "ikev2")

    func testShouldDetectDropWhenConnectedTunnelGoesDown() {
        XCTAssertTrue(Engine.tunnelDropped(from: connected, to: .down))
    }

    func testShouldNotDetectDropWhileStillConnected() {
        XCTAssertFalse(Engine.tunnelDropped(from: connected, to: connected))
    }

    func testShouldNotDetectDropWhenTunnelWasNeverConnected() {
        XCTAssertFalse(Engine.tunnelDropped(from: .unknown, to: .down))
        XCTAssertFalse(Engine.tunnelDropped(from: .down, to: .down))
    }

    func testShouldNotDetectDropAfterStoppedSearch() {
        XCTAssertFalse(Engine.tunnelDropped(from: searching, to: .down))
    }

    func testShouldPollStatusEveryThreeSeconds() {
        XCTAssertEqual(Engine.statusInterval, 3)
    }
}
