import SwiftUI

/// The small "Select all" and "Deselect all" links next to a list's heading, the same in every Settings
/// list. A list where selecting everything means nothing, such as the order itself, passes no select action.
struct SelectionButtons: View {
    var selectAll: (() -> Void)?
    let deselectAll: () -> Void

    var body: some View {
        HStack(spacing: Layout.rowSpacing) {
            if let selectAll {
                Button("Select all", action: selectAll)
            }
            Button("Deselect all", action: deselectAll)
        }
        .buttonStyle(.link)
        .font(.caption)
    }
}
