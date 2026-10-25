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

    func testShouldSplitPairOnLastSeparator() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "trying: Inside / Job / ikev2"),
            .trying(location: "Inside / Job", protocolName: "ikev2")
        )
    }

    func testShouldParseCliMissing() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "windscribe-cli missing: /usr/local/bin/windscribe-cli"),
            .cliMissing(path: "/usr/local/bin/windscribe-cli")
        )
        XCTAssertEqual(EngineEvent.cliMissing(path: "/usr/local/bin/windscribe-cli").tone, .error)
    }

    func testShouldParseConnectedStatusLine() {
        XCTAssertEqual(
            EngineOutputParser.parseStatus(line: "connected: Atlanta - Magic City / IKEv2"),
            TunnelStatus(location: "Atlanta - Magic City", protocolName: "IKEv2")
        )
    }

    func testShouldReturnNilForDown() {
        XCTAssertNil(EngineOutputParser.parseStatus(line: "down"))
    }

    func testShouldReturnNilStatusWhenTheCliIsMissing() {
        XCTAssertNil(EngineOutputParser.parseStatus(line: "windscribe-cli missing: /usr/local/bin/windscribe-cli"))
    }

    func testShouldReturnNilStatusForEmptyOutput() {
        XCTAssertNil(EngineOutputParser.parseStatus(line: ""))
    }

    func testShouldParsePhaseLine() {
        XCTAssertEqual(EngineOutputParser.parse(line: "phase: tcp on port 443"), .phase("tcp on port 443"))
    }

    func testShouldKeepThePortInTryingLine() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "trying: The 6 / tcp:443"),
            .trying(location: "The 6", protocolName: "tcp:443")
        )
    }

    func testShouldGivePhasesTheirTone() {
        XCTAssertEqual(EngineEvent.phase("reachability").tone, .phase)
    }
}
