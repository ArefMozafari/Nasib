use nasib_linux::launch::{is_background_launch, should_show_window};

mod show_window {
    use super::*;

    #[test]
    fn should_show_the_window_when_launched_by_hand() {
        assert!(should_show_window(false, true));
        assert!(should_show_window(false, false));
    }

    #[test]
    fn should_stay_hidden_with_background_and_a_tray() {
        assert!(!should_show_window(true, true));
    }

    // Without a tray nothing could bring the window back, so it shows even from autostart.
    #[test]
    fn should_show_the_window_without_a_tray_even_in_background() {
        assert!(should_show_window(true, false));
    }
}

mod arguments {
    use super::*;

    #[test]
    fn should_read_the_background_flag() {
        assert!(is_background_launch(&[
            "nasib".to_string(),
            "--background".to_string()
        ]));
        assert!(!is_background_launch(&["nasib".to_string()]));
    }
}
