use nasib_linux::controller::ConnectionState;
use nasib_linux::text::{state_title, update_to};

mod titles {
    use super::*;

    #[test]
    fn should_title_each_connection_state() {
        assert_eq!(state_title(&ConnectionState::Unknown), "Checking…");
        assert_eq!(state_title(&ConnectionState::Down), "Not connected");
        assert_eq!(
            state_title(&ConnectionState::Searching {
                location: "Lygos".to_string(),
                protocol: "tcp:443".to_string()
            }),
            "Trying Lygos / tcp:443"
        );
        assert_eq!(
            state_title(&ConnectionState::Connected {
                location: "Istanbul - Lygos".to_string(),
                protocol: "TCP".to_string()
            }),
            "Istanbul - Lygos / TCP"
        );
    }

    #[test]
    fn should_name_the_update_version() {
        assert_eq!(update_to("0.4.0"), "Update to 0.4.0");
    }
}
