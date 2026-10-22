import SwiftUI

/// One protocol row in the ordered list: its name and whether the engine should try it.
struct ProtocolRow: Identifiable, Equatable {
    let name: String
    var enabled: Bool
    var id: String { name }
}

/// Edits the cities and protocol order the engine reads from its config file.
struct SettingsView: View {
    @State private var citiesText: String
    @State private var protocolRows: [ProtocolRow]
    @State private var message: String?

    init() {
        let config = EngineConfig.load()
        _citiesText = State(initialValue: config.cities.joined(separator: "\n"))
        _protocolRows = State(initialValue: Self.rows(for: config))
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
        let cities = citiesText
            .split(whereSeparator: \.isNewline)
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
        return EngineConfig(cities: cities, protocols: protocolRows.filter(\.enabled).map(\.name))
    }

    private func save() {
        do {
            try editedConfig.save()
            message = "Saved. The next search uses these settings."
        } catch {
            message = error.localizedDescription
        }
    }

    private func resetToDefaults() {
        citiesText = EngineConfig.default.cities.joined(separator: "\n")
        protocolRows = Self.rows(for: .default)
        message = nil
    }

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.sectionSpacing) {
            VStack(alignment: .leading, spacing: Layout.rowSpacing) {
                Text("Cities to search").font(.headline)
                Text("One per line, as Windscribe names them. Leave empty to search every city.")
                    .font(.caption).foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                TextEditor(text: $citiesText)
                    .font(.body.monospaced())
                    .frame(height: Layout.citiesEditorHeight)
            }
            VStack(alignment: .leading, spacing: Layout.rowSpacing) {
                Text("Protocols, in the order to try").font(.headline)
                Text("Drag to reorder. Unchecked protocols are never tried.")
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
            HStack {
                Button("Reset to Defaults") { resetToDefaults() }
                Spacer()
                Button("Save") { save() }
                    .keyboardShortcut(.defaultAction)
            }
            if let message {
                Text(message).font(.caption).foregroundStyle(.secondary)
            }
        }
    }
}
