//! Allows one search, watch or survey at a time across every process: the auto-reconnect check fires
//! every minute and a search takes longer than that. An OS file lock is released when its process dies,
//! so a crash can't leave a stale lock behind.

use std::fs::{self, File, OpenOptions};
use std::path::Path;

/// Holds the lock until dropped.
#[derive(Debug)]
pub struct RunLock {
    _file: File,
}

/// Why the lock wasn't taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockRefusal {
    /// Another run holds it.
    HeldByAnotherRun,
    /// A folder with files in it sits at the lock path; it is never removed, since someone put them there.
    BlockedByFolder,
    /// The lock file couldn't be created, for example because the state folder isn't writable.
    Unavailable,
}

impl RunLock {
    /// Takes the lock. An empty directory at the path is what ws-find.sh used as its lock, so it is
    /// removed first; anything else there is left alone.
    pub fn acquire(path: &Path) -> Result<RunLock, LockRefusal> {
        if path.is_dir() {
            fs::remove_dir(path).map_err(|_| LockRefusal::BlockedByFolder)?;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| LockRefusal::Unavailable)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)
            .map_err(|_| LockRefusal::Unavailable)?;
        file.try_lock().map_err(|_| LockRefusal::HeldByAnotherRun)?;
        Ok(RunLock { _file: file })
    }
}
