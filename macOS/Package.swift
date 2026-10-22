// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Nasib",
    platforms: [.macOS(.v15)],
    targets: [
        .executableTarget(name: "Nasib", path: "Sources/Nasib"),
        .testTarget(name: "NasibTests", dependencies: ["Nasib"], path: "Tests/NasibTests"),
    ],
    // Swift 5 language mode: the engine uses plain Process callbacks, and strict concurrency adds nothing here.
    swiftLanguageModes: [.v5]
)
