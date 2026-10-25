use nasib_linux::appearance::color_scheme_prefers_dark;

mod portal_values {
    use super::*;

    #[test]
    fn should_map_the_portal_values() {
        assert_eq!(color_scheme_prefers_dark(1), Some(true));
        assert_eq!(color_scheme_prefers_dark(2), Some(false));
        assert_eq!(color_scheme_prefers_dark(0), None);
    }

    #[test]
    fn should_ignore_unknown_values() {
        assert_eq!(color_scheme_prefers_dark(3), None);
        assert_eq!(color_scheme_prefers_dark(u32::MAX), None);
    }
}
