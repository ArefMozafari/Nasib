import SwiftUI

/// The Locations section of Settings: the checked datacenters in the order to try, side by side with a
/// searchable list of every datacenter to add from, and the switch that reorders them by recent success.
/// Both columns have the same rows, a heading, a search field, a list and a line below, so they line up.
struct LocationsSettings: View {
    @Binding var selection: LocationSelection
    @Binding var rankBySuccess: Bool
    let catalog: EngineCatalog
    @State private var orderSearchText = ""
    @State private var catalogSearchText = ""

    /// A toggle state for one datacenter that adds or removes it from the order.
    private func isSelected(_ nickname: String) -> Binding<Bool> {
        Binding(
            get: { selection.isSelected(nickname) },
            set: { _ in selection.toggle(nickname) }
        )
    }

    private static func query(_ text: String) -> String {
        text.trimmingCharacters(in: .whitespaces)
    }

    /// The checked datacenters that match the order's search, in the user's order.
    private var filteredOrder: [EngineCatalog.Entry] {
        let query = Self.query(orderSearchText)
        guard !query.isEmpty else { return selection.ordered }
        return selection.ordered.filter { $0.title.localizedCaseInsensitiveContains(query) }
    }

    /// The regions with only the datacenters that match the catalog's search, in catalog order.
    private var filteredRegions: [EngineCatalog.Region] {
        let query = Self.query(catalogSearchText)
        guard !query.isEmpty else { return catalog.regions }
        return catalog.regions.compactMap { region in
            let matches = region.name.localizedCaseInsensitiveContains(query)
                ? region.datacenters
                : region.datacenters.filter { $0.title.localizedCaseInsensitiveContains(query) }
            return matches.isEmpty ? nil : EngineCatalog.Region(name: region.name, datacenters: matches)
        }
    }

    private var visibleCatalogNicknames: [String] {
        filteredRegions.flatMap(\.datacenters).map(\.nickname)
    }

    /// A region's name with one link that checks every datacenter it shows, or unchecks them once all are.
    private func regionHeader(_ region: EngineCatalog.Region) -> some View {
        let nicknames = region.datacenters.map(\.nickname)
        let allSelected = selection.areAllSelected(nicknames)
        return HStack {
            Text(region.name)
            Spacer()
            Button(allSelected ? "Deselect all" : "Select all") {
                if allSelected {
                    selection.deselect(nicknames)
                } else {
                    selection.select(nicknames)
                }
            }
            .buttonStyle(.link)
            .font(.caption)
        }
    }

    private func searchField(_ prompt: String, text: Binding<String>) -> some View {
        TextField(prompt, text: text)
            .textFieldStyle(.roundedBorder)
            .frame(height: Layout.columnControlHeight)
    }

    private var orderColumn: some View {
        VStack(alignment: .leading, spacing: Layout.rowSpacing) {
            HStack {
                Text("Your order").font(.subheadline.weight(.semibold))
                Spacer()
                // Acts on what the search shows.
                SelectionButtons(deselectAll: { selection.deselect(filteredOrder.map(\.nickname)) })
            }
            searchField("Search your order", text: $orderSearchText)
            List {
                ForEach(filteredOrder) { entry in
                    Toggle(entry.title, isOn: isSelected(entry.nickname))
                }
                // Dragging is off while the order is filtered, since a filtered row's place is ambiguous.
                .onMove(perform: Self.query(orderSearchText).isEmpty ? { source, destination in
                    selection.move(from: source, to: destination)
                } : nil)
            }
            Toggle("Rank by recent success", isOn: $rankBySuccess)
                .help(Tooltip.rankBySuccess)
                .frame(height: Layout.columnControlHeight)
        }
    }

    private var pickerColumn: some View {
        VStack(alignment: .leading, spacing: Layout.rowSpacing) {
            HStack {
                Text("Add locations").font(.subheadline.weight(.semibold))
                Spacer()
                // Both act on what the search shows.
                SelectionButtons(
                    selectAll: { selection.select(visibleCatalogNicknames) },
                    deselectAll: { selection.deselect(visibleCatalogNicknames) }
                )
            }
            searchField("Search cities, nicknames or regions", text: $catalogSearchText)
            List {
                ForEach(filteredRegions) { region in
                    Section {
                        ForEach(region.datacenters) { entry in
                            Toggle(entry.title, isOn: isSelected(entry.nickname))
                        }
                    } header: {
                        regionHeader(region)
                    }
                }
            }
            // Fills the line the ranking switch takes in the other column, so both lists end together.
            Text("\(selection.ordered.count) of \(catalog.entries.count) checked")
                .font(.caption).foregroundStyle(.secondary)
                .frame(height: Layout.columnControlHeight)
        }
    }

    var body: some View {
        // A group box sets the section apart from the window on the system's own background, like the
        // Protocols section below it.
        GroupBox {
            VStack(alignment: .leading, spacing: Layout.rowSpacing) {
                Text("Locations, in the order to try").font(.headline)
                Text("Drag to reorder. Uncheck one to stop trying it.")
                    .font(.caption).foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                HStack(alignment: .top, spacing: Layout.sectionSpacing) {
                    orderColumn
                    pickerColumn
                }
                .frame(height: Layout.locationListsHeight)
            }
            .padding(Layout.groupPadding)
        }
    }
}
