import XCTest
@testable import Nasib

final class EngineConfigTests: XCTestCase {
    private var directory: URL!

    override func setUpWithError() throws {
        directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        try? FileManager.default.removeItem(at: directory)
    }

    private var jsonFile: URL { directory.appendingPathComponent("config.json") }
    private var legacyFile: URL { directory.appendingPathComponent("config") }

    func testShouldRoundTripJson() throws {
        let config = EngineConfig(cities: ["Saint John's", "New York"], protocols: ["ikev2", "tcp"])
        try config.save(to: directory)
        XCTAssertEqual(EngineConfig.load(from: directory), config)
    }

    func testShouldWriteTheKeysTheEngineReads() throws {
        try EngineConfig(cities: ["Dubai"], protocols: ["udp"]).save(to: directory)
        let text = try String(contentsOf: jsonFile, encoding: .utf8)
        XCTAssertTrue(text.contains("\"cities\""))
        XCTAssertTrue(text.contains("\"protocolOrder\""))
    }

    func testShouldLoadTheDefaultsWhenNothingIsSaved() {
        XCTAssertEqual(EngineConfig.load(from: directory), .default)
    }

    func testShouldDropUnknownProtocolsWhenLoading() throws {
        try #"{ "cities": ["Dubai"], "protocolOrder": ["tcp", "bogus", "wireguard"] }"#
            .write(to: jsonFile, atomically: true, encoding: .utf8)
        XCTAssertEqual(EngineConfig.load(from: directory).protocols, ["tcp", "wireguard"])
    }

    func testShouldMigrateLegacyConfigAndDeleteIt() throws {
        try "# Written by Nasib\nCITIES=('Saint John'\\''s' 'Zurich')\nPROTOCOL_ORDER=(wireguard tcp)\n"
            .write(to: legacyFile, atomically: true, encoding: .utf8)

        EngineConfig.migrateLegacyFileIfNeeded(in: directory)

        XCTAssertEqual(
            EngineConfig.load(from: directory),
            EngineConfig(cities: ["Saint John's", "Zurich"], protocols: ["wireguard", "tcp"])
        )
        XCTAssertFalse(FileManager.default.fileExists(atPath: legacyFile.path))
    }

    func testShouldMigrateAnEmptyCityList() throws {
        try "CITIES=()\nPROTOCOL_ORDER=(udp)\n".write(to: legacyFile, atomically: true, encoding: .utf8)

        EngineConfig.migrateLegacyFileIfNeeded(in: directory)

        XCTAssertEqual(EngineConfig.load(from: directory), EngineConfig(cities: [], protocols: ["udp"]))
    }

    func testShouldNotMigrateWhenJsonExists() throws {
        let saved = EngineConfig(cities: ["Dubai"], protocols: ["ikev2"])
        try saved.save(to: directory)
        try "CITIES=('Zurich')\nPROTOCOL_ORDER=(tcp)\n".write(to: legacyFile, atomically: true, encoding: .utf8)

        EngineConfig.migrateLegacyFileIfNeeded(in: directory)

        XCTAssertEqual(EngineConfig.load(from: directory), saved)
        XCTAssertTrue(FileManager.default.fileExists(atPath: legacyFile.path))
    }

    func testShouldKeepLegacyFileWhenItCannotBeParsed() throws {
        try "# nothing here\n".write(to: legacyFile, atomically: true, encoding: .utf8)

        EngineConfig.migrateLegacyFileIfNeeded(in: directory)

        XCTAssertFalse(FileManager.default.fileExists(atPath: jsonFile.path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: legacyFile.path))
    }

    func testShouldKeepProtocolOrderAndDropUnknownNamesInLegacyFiles() {
        let text = "CITIES=('Dubai')\nPROTOCOL_ORDER=(tcp bogus wireguard)\n"
        XCTAssertEqual(EngineConfig.parseLegacy(text)?.protocols, ["tcp", "wireguard"])
    }

    func testShouldReturnNilWhenALegacyArrayLineIsMissing() {
        XCTAssertNil(EngineConfig.parseLegacy("CITIES=('Dubai')\n"))
        XCTAssertNil(EngineConfig.parseLegacy("# nothing here\n"))
    }

    func testShouldDefaultToEveryProtocol() {
        XCTAssertEqual(EngineConfig.default.protocols, EngineConfig.allProtocols)
    }
}
