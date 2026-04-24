use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::{IoError, IoErrorKind};

/// Read a file with a maximum byte limit.
///
/// # Errors
/// Returns an IO error if reading fails or the file exceeds the limit.
pub fn read_to_end_bounded(path: &Path, max_bytes: usize) -> Result<Vec<u8>, IoError> {
    let file = File::open(path).map_err(IoError::from_io)?;
    let mut buffer = Vec::new();
    let limit = match u64::try_from(max_bytes) {
        Ok(value) => value.saturating_add(1),
        Err(_) => u64::MAX,
    };
    file.take(limit).read_to_end(&mut buffer).map_err(IoError::from_io)?;
    if buffer.len() > max_bytes {
        return Err(IoError::new(
            IoErrorKind::Corruption,
            format!("file exceeds max bytes ({max_bytes})"),
        ));
    }
    Ok(buffer)
}
