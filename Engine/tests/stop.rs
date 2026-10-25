use std::thread;
use std::time::{Duration, Instant};

use nasib_engine::stop::{StopSignal, Stopped};

const LONG_WAIT: Duration = Duration::from_secs(10);
const REQUEST_DELAY: Duration = Duration::from_millis(20);
const PROMPT_LIMIT: Duration = Duration::from_millis(200);
const SHORT_WAIT: Duration = Duration::from_millis(30);

mod run {
    use super::*;

    #[test]
    fn should_return_the_work_result() {
        assert_eq!(StopSignal::new().run(|| 42), Ok(42));
    }

    #[test]
    fn should_return_stopped_while_the_work_is_still_running() {
        let stop = StopSignal::new();
        let requester = stop.clone();
        let started = Instant::now();
        thread::spawn(move || {
            thread::sleep(REQUEST_DELAY);
            requester.request();
        });

        let result = stop.run(|| thread::sleep(LONG_WAIT));

        assert_eq!(result, Err(Stopped));
        assert!(started.elapsed() < PROMPT_LIMIT);
    }

    #[test]
    fn should_finish_the_work_on_a_shielded_signal() {
        let stop = StopSignal::new();
        let shielded = stop.shield();
        stop.request();

        assert_eq!(
            shielded.run(|| {
                thread::sleep(SHORT_WAIT);
                "done"
            }),
            Ok("done")
        );
    }
}

mod wait {
    use super::*;

    #[test]
    fn should_return_stopped_when_requested_during_wait() {
        let stop = StopSignal::new();
        let requester = stop.clone();
        let started = Instant::now();
        thread::spawn(move || {
            thread::sleep(REQUEST_DELAY);
            requester.request();
        });

        let result = stop.wait(LONG_WAIT);

        assert_eq!(result, Err(Stopped));
        assert!(started.elapsed() < PROMPT_LIMIT);
    }

    #[test]
    fn should_wait_the_full_duration_without_a_request() {
        let stop = StopSignal::new();
        let started = Instant::now();

        let result = stop.wait(SHORT_WAIT);

        assert_eq!(result, Ok(()));
        assert!(started.elapsed() >= SHORT_WAIT);
    }

    #[test]
    fn should_return_stopped_at_once_after_an_earlier_request() {
        let stop = StopSignal::new();
        stop.request();
        let started = Instant::now();

        assert_eq!(stop.wait(LONG_WAIT), Err(Stopped));
        assert!(started.elapsed() < PROMPT_LIMIT);
    }
}

mod is_requested {
    use super::*;

    #[test]
    fn should_be_false_before_a_request() {
        assert!(!StopSignal::new().is_requested());
    }

    #[test]
    fn should_be_true_after_a_request_from_a_clone() {
        let stop = StopSignal::new();
        stop.clone().request();

        assert!(stop.is_requested());
    }
}

mod shield {
    use super::*;

    #[test]
    fn should_ignore_requests_on_a_shielded_signal() {
        let stop = StopSignal::new();
        let shielded = stop.shield();
        stop.request();
        shielded.request();

        assert!(!shielded.is_requested());
        assert_eq!(shielded.wait(SHORT_WAIT), Ok(()));
    }
}
