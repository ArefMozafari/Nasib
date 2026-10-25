use std::time::Duration;

use nasib_engine::timings::Timings;

mod default_timings {
    use super::*;

    #[test]
    fn should_match_ws_find_defaults() {
        let timings = Timings::default();

        assert_eq!(timings.probe_timeout, Duration::from_secs(8));
        assert_eq!(timings.connect_timeout, Duration::from_secs(10));
        assert_eq!(timings.settle_delay, Duration::from_secs(2));
        assert_eq!(timings.disconnect_timeout, Duration::from_secs(15));
        assert_eq!(timings.status_poll_interval, Duration::from_secs(1));
        assert_eq!(timings.reach_timeout, Duration::from_secs(2));
    }
}

mod scaled {
    use super::*;

    #[test]
    fn should_divide_every_timing_by_the_scale() {
        let timings = Timings::default().scaled(100.0);

        assert_eq!(timings.probe_timeout, Duration::from_millis(80));
        assert_eq!(timings.connect_timeout, Duration::from_millis(100));
        assert_eq!(timings.settle_delay, Duration::from_millis(20));
        assert_eq!(timings.disconnect_timeout, Duration::from_millis(150));
        assert_eq!(timings.status_poll_interval, Duration::from_millis(10));
        assert_eq!(timings.reach_timeout, Duration::from_millis(20));
    }

    #[test]
    fn should_keep_the_ratio_between_poll_and_timeout() {
        let timings = Timings::default().scaled(1000.0);

        assert_eq!(
            timings.connect_timeout.as_nanos() / timings.status_poll_interval.as_nanos(),
            10
        );
    }
}
