//! What a dragged row carries in Settings, so a row dropped on the other list is refused.

const PAYLOAD_SEPARATOR: char = ':';

/// The two lists in Settings whose rows can be dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReorderList {
    Locations,
    Protocols,
}

impl ReorderList {
    fn name(self) -> &'static str {
        match self {
            ReorderList::Locations => "locations",
            ReorderList::Protocols => "protocols",
        }
    }
}

/// The text a dragged row carries: its list and its index.
pub fn drag_payload(list: ReorderList, index: usize) -> String {
    format!("{}{PAYLOAD_SEPARATOR}{index}", list.name())
}

/// The dragged row's index when it comes from `list`; `None` for a row from the other list or anything else.
pub fn dragged_index(list: ReorderList, payload: &str) -> Option<usize> {
    let (name, index) = payload.split_once(PAYLOAD_SEPARATOR)?;
    if name != list.name() {
        return None;
    }
    index.parse().ok()
}
