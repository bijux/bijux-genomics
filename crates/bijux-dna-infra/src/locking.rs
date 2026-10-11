use fs4::fs_std::FileExt;
use std::fs::{File, OpenOptions};
use std::path::Path;

use crate::{ensure_dir, IoError, IoErrorKind};

#[derive(Debug)]
pub struct FileLock {
    file: File,
}

impl FileLock {
    /// Attempt one exclusive file-lock acquisition without waiting.
    ///
    /// # Errors
    /// Returns a lock-contention error or IO error on failure.
    pub fn try_acquire(path: &Path) -> Result<Self, IoError> {
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            ensure_dir(parent)?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(IoError::from_io)?;
        Self::try_lock(file)
    }

    /// Acquire an existing lease without creating a file or requiring write access.
    ///
    /// # Errors
    /// Returns lock contention or an IO error. An absent lease returns `None`.
    pub fn try_acquire_existing(path: &Path) -> Result<Option<Self>, IoError> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(IoError::from_io(error)),
        };
        Self::try_lock(file).map(Some)
    }

    fn try_lock(file: File) -> Result<Self, IoError> {
        match file.try_lock_exclusive() {
            Ok(true) => Ok(Self { file }),
            Ok(false) => Err(IoError::new(IoErrorKind::LockContended, "file lock is already held")),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                Err(IoError::new(IoErrorKind::LockContended, "file lock is already held"))
            }
            Err(error) => Err(IoError::from_io(error)),
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs4::fs_std::FileExt::unlock(&self.file);
    }
}
