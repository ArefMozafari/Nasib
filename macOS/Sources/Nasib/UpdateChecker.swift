import AppKit

/// Asks GitHub for the latest release at launch and once a day, and offers it when it is newer than this build.
/// Updating stays manual: the app opens the release page and the user downloads the new zip.
@MainActor
final class UpdateChecker: ObservableObject {
    struct Release: Equatable {
        let version: String
        let pageURL: URL
    }

    /// The fields of GitHub's latest-release response that the checker reads.
    private struct LatestReleaseResponse: Decodable {
        let tagName: String
        let htmlURL: URL

        enum CodingKeys: String, CodingKey {
            case tagName = "tag_name"
            case htmlURL = "html_url"
        }
    }

    private static let latestReleaseURL = URL(string: "https://api.github.com/repos/ArefMozafari/Nasib/releases/latest")!
    private static let checkInterval: TimeInterval = 24 * 60 * 60
    private nonisolated static let tagPrefix = "v"
    private nonisolated static let versionSeparator: Character = "."
    private static let versionKey = "CFBundleShortVersionString"
    // Remembers which version the user was already told about, so a relaunch doesn't notify again.
    private static let notifiedVersionKey = "notifiedUpdateVersion"
    private static let notificationTitle = "Update available"

    @Published private(set) var available: Release?

    private var timer: Timer?

    init() {
        timer = Timer.scheduledTimer(withTimeInterval: Self.checkInterval, repeats: true) { [weak self] _ in
            Task { @MainActor in await self?.check() }
        }
        Task { await check() }
    }

    private static var currentVersion: String {
        Bundle.main.object(forInfoDictionaryKey: versionKey) as? String ?? ""
    }

    /// Compares dotted version numbers component by component; a missing component counts as zero.
    nonisolated static func isNewer(_ candidate: String, than current: String) -> Bool {
        let candidateParts = candidate.split(separator: versionSeparator).map { Int($0) ?? 0 }
        let currentParts = current.split(separator: versionSeparator).map { Int($0) ?? 0 }
        for index in 0..<max(candidateParts.count, currentParts.count) {
            let candidatePart = index < candidateParts.count ? candidateParts[index] : 0
            let currentPart = index < currentParts.count ? currentParts[index] : 0
            if candidatePart != currentPart { return candidatePart > currentPart }
        }
        return false
    }

    /// Reads the version and release page from GitHub's latest-release response; nil for anything else.
    nonisolated static func parseRelease(_ data: Data) -> Release? {
        guard let response = try? JSONDecoder().decode(LatestReleaseResponse.self, from: data) else { return nil }
        let version = response.tagName.hasPrefix(tagPrefix) ? String(response.tagName.dropFirst(tagPrefix.count)) : response.tagName
        return Release(version: version, pageURL: response.htmlURL)
    }

    /// Posts one notification per new version.
    private func notifyOnce(about release: Release) {
        let defaults = UserDefaults.standard
        guard defaults.string(forKey: Self.notifiedVersionKey) != release.version else { return }
        defaults.set(release.version, forKey: Self.notifiedVersionKey)
        Notifier.post(title: Self.notificationTitle, body: "Nasib \(release.version) is ready to download.", opening: release.pageURL)
    }

    /// Stays quiet on failure: a filtered network often can't reach GitHub, and the next check retries.
    private func check() async {
        guard let (data, _) = try? await URLSession.shared.data(from: Self.latestReleaseURL),
              let release = Self.parseRelease(data),
              Self.isNewer(release.version, than: Self.currentVersion)
        else { return }
        available = release
        notifyOnce(about: release)
    }

    func openReleasePage() {
        guard let available else { return }
        NSWorkspace.shared.open(available.pageURL)
    }
}
