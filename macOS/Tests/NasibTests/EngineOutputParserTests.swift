import XCTest
@testable import Nasib

final class EngineOutputParserTests: XCTestCase {
    func testShouldParseTryingLine() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "trying: Stolen Cheese / ikev2"),
            .trying(location: "Stolen Cheese", protocolName: "ikev2")
        )
    }

    func testShouldParseConnectedLine() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "connected: Magic City / ikev2"),
            .connected(location: "Magic City", protocolName: "ikev2")
        )
    }

    func testShouldParseNothingWorkedLine() {
        XCTAssertEqual(EngineOutputParser.parse(line: "nothing worked"), .nothingWorked)
    }

    func testShouldParseSurveyFinishedLine() {
        XCTAssertEqual(EngineOutputParser.parse(line: "survey finished"), .surveyFinished)
    }

    func testShouldParseFailedLine() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "failed: Sausage Party / wireguard"),
            .failed(location: "Sausage Party", protocolName: "wireguard")
        )
    }

    func testShouldParseSearchStoppedLine() {
        XCTAssertEqual(EngineOutputParser.parse(line: "search stopped"), .searchStopped)
    }

    func testShouldParseSurveyStoppedLine() {
        XCTAssertEqual(EngineOutputParser.parse(line: "survey stopped"), .surveyStopped)
    }

    func testShouldGiveEachEventItsTone() {
        XCTAssertEqual(EngineEvent.trying(location: "Lygos", protocolName: "tcp").tone, .primary)
        XCTAssertEqual(EngineEvent.connected(location: "Lygos", protocolName: "tcp").tone, .success)
        XCTAssertEqual(EngineEvent.failed(location: "Lygos", protocolName: "tcp").tone, .muted)
        XCTAssertEqual(EngineEvent.tunnelDown.tone, .warning)
        XCTAssertEqual(EngineEvent.nothingWorked.tone, .error)
        XCTAssertEqual(EngineEvent.searchStopped.tone, .accent)
        XCTAssertEqual(EngineEvent.surveyStopped.tone, .accent)
        XCTAssertEqual(EngineEvent.surveyFinished.tone, .accent)
        XCTAssertEqual(EngineEvent.other("protocol order (from Settings): tcp").tone, .detail)
    }

    func testShouldParseTunnelDownLine() {
        XCTAssertEqual(EngineOutputParser.parse(line: "tunnel down"), .tunnelDown)
    }

    func testShouldTreatUnknownLineAsOther() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "protocol order (from Settings): ikev2, wireguard"),
            .other("protocol order (from Settings): ikev2, wireguard")
        )
    }

    func testShouldParseConnectedStatusWithProtocolPortStripped() {
        let output = """
        Internet connectivity: available
        Login state: Logged in
        Connect state: Connected: Atlanta - Magic City
        Protocol: IKEv2:500
        VPN IP: 198.51.100.20
        """
        XCTAssertEqual(
            EngineOutputParser.parseStatus(output: output),
            TunnelStatus(location: "Atlanta - Magic City", protocolName: "IKEv2")
        )
    }

    func testShouldReturnNilStatusWhenDisconnected() {
        let output = """
        Connect state: Disconnected
        Public IP: 203.0.113.10
        """
        XCTAssertNil(EngineOutputParser.parseStatus(output: output))
    }

    func testShouldReturnNilStatusWhileConnecting() {
        XCTAssertNil(EngineOutputParser.parseStatus(output: "Connect state: Connecting\nProtocol: IKEv2:500"))
    }
}
