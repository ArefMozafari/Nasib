use nasib_engine::mode::Mode;

fn parse(arguments: &[&str]) -> Result<Mode, String> {
    Mode::parse(
        &arguments
            .iter()
            .map(|argument| argument.to_string())
            .collect::<Vec<_>>(),
    )
}

mod parse {
    use super::*;

    #[test]
    fn should_search_once_without_arguments() {
        assert_eq!(parse(&[]), Ok(Mode::Find));
    }

    #[test]
    fn should_read_every_flag() {
        let cases = [
            ("--watch", Mode::Watch),
            ("--survey", Mode::Survey),
            ("--reach", Mode::Reach),
            ("--rank", Mode::Rank),
            ("--locations", Mode::Locations),
            ("--status", Mode::Status),
        ];
        for (flag, expected) in cases {
            assert_eq!(parse(&[flag]), Ok(expected), "{flag}");
        }
    }

    #[test]
    fn should_reject_an_unknown_flag() {
        assert!(parse(&["--fast"]).is_err());
    }

    #[test]
    fn should_reject_more_than_one_argument() {
        assert!(parse(&["--watch", "--survey"]).is_err());
    }
}
