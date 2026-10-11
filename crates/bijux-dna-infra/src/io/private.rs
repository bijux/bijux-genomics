//! Atomic publication retaining the temporary file's private creation permissions.
use super::IoError;
use std::io::Write;
use std::path::Path;

/// Atomically replace a file with bytes written through a privately created temporary file.
/// Unix files remain mode 0600 through publication, including replacement of a public file.
/// Destination-directory admission and multi-file transactions belong to the caller.
/// # Errors
/// Returns creation, write, flush or persistence errors. Failed staging is removed on drop.
pub fn atomic_write_private_bytes(path: &Path, bytes: &[u8]) -> Result<(), IoError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged = crate::private_temp_file_in(parent, ".private-publication-")?;
    staged.as_file_mut().write_all(bytes).map_err(IoError::from_io)?;
    staged.as_file_mut().sync_all().map_err(IoError::from_io)?;
    staged.persist(path).map_err(|error| IoError::from_io(error.error))?;
    Ok(())
}
