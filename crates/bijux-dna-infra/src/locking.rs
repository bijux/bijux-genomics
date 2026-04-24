use fs4::fs_std::FileExt;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::{IoError, IoErrorKind};

#[derive(Debug)]
pub struct FileLock {
    file: File,
}

impl FileLock {
    /// Acquire an exclusive lock on a file within a timeout.
    ///
    /// # Errors
    /// Returns a lock timeout error or IO error on failure.
    pub fn acquire(path: &Path, timeout: Duration) -> Result<Self, IoError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(IoError::from_io)?;
        let start = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(true) => return Ok(Self { file }),
                Ok(false) => {
                    if start.elapsed() >= timeout {
                        return Err(IoError::new(
                            IoErrorKind::LockTimeout,
                            "file lock timeout".to_string(),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(err) => {
                    if start.elapsed() >= timeout {
                        return Err(IoError::new(IoErrorKind::LockTimeout, err.to_string()));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs4::fs_std::FileExt::unlock(&self.file);
    }
}
