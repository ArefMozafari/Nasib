import SwiftUI

/// One protocol row in the ordered list: its name and whether the engine should try it.
struct ProtocolRow: Identifiable, Equatable {
    let name: String
    var enabled: Bool
    var id: String { name }
}

/// Edits the locations, ranking and protocol order the engine reads from its config file.
struct SettingsView: View {
    /// What was saved, kept so the cities older versions saved carry through a save untouched.
    @State private var saved: EngineConfig
    @State private var protocolRows: [ProtocolRow]
    @State private var rankBySuccess: Bool
    /// Nil until the engine's catalog has been read.
    @State private var catalog: EngineCatalog?
    @State private var selection: LocationSelection?
    @State private var message: String?

    private static let emptyCatalog = EngineCatalog(regions: [])
    private static let emptyOrderMessage = "Check at least one location."
    private static let noProtocolMessage = "Check at least one protocol."
    private static let unreadableCatalogMessage = "Couldn't read the datacenter list."
    private static let citiesKeptMessage = "Couldn't read the datacenter list, so your saved cities stay as they are."

    init() {
        let config = EngineConfig.load()
        _saved = State(initialValue: config)
        _protocolRows = State(initialValue: Self.rows(for: config))
        _rankBySuccess = State(initialValue: config.rankBySuccess)
    }

    /// Enabled protocols first in their saved order, then the disabled ones in default order.
    private static func rows(for config: EngineConfig) -> [ProtocolRow] {
        let enabled = config.protocols.map { ProtocolRow(name: $0, enabled: true) }
        let disabled = EngineConfig.allProtocols
            .filter { !config.protocols.contains($0) }
            .map { ProtocolRow(name: $0, enabled: false) }
        return enabled + disabled
    }

    private var editedConfig: EngineConfig {
        EngineConfig(
            cities: saved.cities,
            protocols: protocolRows.filter(\.enabled).map(\.name),
            locations: selection?.nicknames ?? saved.locations,
            rankBySuccess: rankBySuccess
        )
    }

    /// Reads the catalog off the main thread. Without it, the saved order stays editable on its own.
    private func loadCatalog() async {
        let loaded = await Task.detached { Engine.readCatalog() }.value
        catalog = loaded ?? Self.emptyCatalog
        selection = LocationSelection(saved: saved, catalog: loaded ?? Self.emptyCatalog)
    }

    /// Why the settings can't be saved, or nil when they can. An empty location list would mean every
    /// datacenter to the engine, and an empty protocol list would mean nothing is ever tried. A nil
    /// location count means the locations aren't being edited, so the saved ones are kept.
    static func saveProblem(locationCount: Int?, protocolCount: Int) -> String? {
        if locationCount == 0 { return emptyOrderMessage }
        if protocolCount == 0 { return noProtocolMessage }
        return nil
    }

    private func save() {
        let problem = Self.saveProblem(
            locationCount: selection?.ordered.count,
            protocolCount: protocolRows.filter(\.enabled).count
        )
        if let problem {
            message = problem
            return
        }
        do {
            let config = editedConfig
            try config.save()
            saved = config
            message = "Saved. The next search uses these settings."
        } catch {
            message = error.localizedDescription
        }
    }

    private func resetToDefaults() {
        selection?.resetToDefaults()
        rankBySuccess = EngineConfig.default.rankBySuccess
        protocolRows = Self.rows(for: .default)
        message = nil
    }

    /// The selection as a binding, once the catalog has been read.
    private var selectionBinding: Binding<LocationSelection>? {
        guard let current = selection else { return nil }
        return Binding(get: { selection ?? current }, set: { selection = $0 })
    }

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.sectionSpacing) {
            if let selectionBinding, let catalog {
                LocationsSettings(selection: selectionBinding, rankBySuccess: $rankBySuccess, catalog: catalog)
                if catalog.regions.isEmpty {
                    Text(Self.unreadableCatalogMessage).font(.caption).foregroundStyle(.red)
                }
            } else if catalog != nil {
                Text(Self.citiesKeptMessage).font(.caption).foregroundStyle(.red)
            } else {
                ProgressView().frame(maxWidth: .infinity, minHeight: Layout.locationListsHeight)
            }
            GroupBox { protocolsSection.padding(Layout.groupPadding) }
            HStack {
                Button("Reset to Defaults") { resetToDefaults() }
                    .help(Tooltip.resetToDefaults)
                Spacer()
                Button("Save") { save() }
                    .keyboardShortcut(.defaultAction)
                    .help(Tooltip.save)
            }
            if let message {
                Text(message).font(.caption).foregroundStyle(.secondary)
            }
        }
        .task { await loadCatalog() }
    }

    private var protocolsSection: some View {
            VStack(alignment: .leading, spacing: Layout.rowSpacing) {
                HStack {
                    Text("Protocols, in the order to try").font(.headline)
                    Spacer()
                    SelectionButtons(
                        selectAll: { protocolRows.indices.forEach { protocolRows[$0].enabled = true } },
                        deselectAll: { protocolRows.indices.forEach { protocolRows[$0].enabled = false } }
                    )
                }
                Text("Tried in exactly this order. Drag to reorder; unchecked protocols are never tried.")
                    .font(.caption).foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                List {
                    ForEach($protocolRows) { $row in
                        Toggle(row.name, isOn: $row.enabled)
                    }
                    .onMove { source, destination in
                        protocolRows.move(fromOffsets: source, toOffset: destination)
                    }
                }
                .frame(height: Layout.protocolListHeight)
            }
    }
}
