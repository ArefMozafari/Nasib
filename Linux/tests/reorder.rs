use nasib_linux::reorder::{drag_payload, dragged_index, ReorderList};

mod payloads {
    use super::*;

    #[test]
    fn should_read_back_a_row_from_the_same_list() {
        let payload = drag_payload(ReorderList::Locations, 3);

        assert_eq!(dragged_index(ReorderList::Locations, &payload), Some(3));
    }

    // A protocol row dropped on the locations must not move a location, and the other way round.
    #[test]
    fn should_refuse_a_row_from_the_other_list() {
        let payload = drag_payload(ReorderList::Protocols, 3);

        assert_eq!(dragged_index(ReorderList::Locations, &payload), None);
        assert_eq!(
            dragged_index(
                ReorderList::Protocols,
                &drag_payload(ReorderList::Locations, 0)
            ),
            None
        );
    }

    #[test]
    fn should_refuse_anything_else() {
        assert_eq!(dragged_index(ReorderList::Locations, "3"), None);
        assert_eq!(dragged_index(ReorderList::Locations, "locations:x"), None);
    }
}
