mod support;

use std::fs;

use nasib_engine::run_lock::{LockRefusal, RunLock};
use support::TemporaryDirectory;

mod acquire {
    use super::*;

    #[test]
    fn should_refuse_a_second_lock_while_held() {
        let directory = TemporaryDirectory::new();
        let path = directory.path().join("lock");

        let held = RunLock::acquire(&path);

        assert!(held.is_ok());
        assert_eq!(
            RunLock::acquire(&path).unwrap_err(),
            LockRefusal::HeldByAnotherRun
        );
    }

    #[test]
    fn should_grant_the_lock_after_release() {
        let directory = TemporaryDirectory::new();
        let path = directory.path().join("lock");

        drop(RunLock::acquire(&path));

        assert!(RunLock::acquire(&path).is_ok());
    }

    #[test]
    fn should_create_the_state_folder_when_missing() {
        let directory = TemporaryDirectory::new();

        assert!(RunLock::acquire(&directory.path().join("state").join("lock")).is_ok());
    }

    #[test]
    fn should_remove_an_empty_leftover_lock_directory() {
        let directory = TemporaryDirectory::new();
        let path = directory.path().join("lock");
        fs::create_dir(&path).unwrap();

        assert!(RunLock::acquire(&path).is_ok());
        assert!(path.is_file());
    }

    #[test]
    fn should_refuse_when_the_lock_path_is_a_non_empty_directory() {
        let directory = TemporaryDirectory::new();
        let path = directory.path().join("lock");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("keep"), "").unwrap();

        assert_eq!(
            RunLock::acquire(&path).unwrap_err(),
            LockRefusal::BlockedByFolder
        );
        assert!(path.join("keep").exists());
    }
}
