use nasib_linux::update_checker::{is_newer, parse_release, should_notify, Release};

const RELEASE_PAGE: &str = "https://github.com/ArefMozafari/Nasib/releases/tag/v0.3.0";

mod versions {
    use super::*;

    #[test]
    fn should_treat_higher_version_as_newer() {
        assert!(is_newer("0.3.0", "0.2.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
    }

    #[test]
    fn should_compare_versions_numerically() {
        assert!(is_newer("0.10.0", "0.9.0"));
    }

    #[test]
    fn should_not_treat_same_or_older_version_as_newer() {
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.1", "0.2.0"));
    }

    #[test]
    fn should_treat_missing_components_as_zero() {
        assert!(!is_newer("0.2", "0.2.0"));
        assert!(is_newer("0.2.1", "0.2"));
    }
}

mod release {
    use super::*;

    #[test]
    fn should_parse_the_latest_release() {
        let json = format!(
            r#"{{"tag_name": "v0.3.0", "html_url": "{RELEASE_PAGE}", "name": "Nasib 0.3.0"}}"#
        );

        assert_eq!(
            parse_release(&json),
            Some(Release {
                version: "0.3.0".to_string(),
                page: RELEASE_PAGE.to_string()
            })
        );
    }

    #[test]
    fn should_return_none_for_a_response_without_a_release() {
        assert_eq!(parse_release(r#"{"message": "Not Found"}"#), None);
        assert_eq!(parse_release("not json"), None);
    }
}

mod notifying {
    use super::*;

    #[test]
    fn should_notify_each_version_once() {
        assert!(should_notify("0.4.0", None));
        assert!(should_notify("0.4.0", Some("0.3.1")));
        assert!(!should_notify("0.4.0", Some("0.4.0")));
    }
}
