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

    func testShouldTreatUnknownLineAsOther() {
        XCTAssertEqual(
            EngineOutputParser.parse(line: "protocol order: ikev2, wireguard"),
            .other("protocol order: ikev2, wireguard")
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
