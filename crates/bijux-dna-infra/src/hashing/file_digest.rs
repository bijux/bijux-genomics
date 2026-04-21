use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::IoError;

pub(super) fn hash_file_sha256(path: &Path) -> Result<String, IoError> {
    use sha2::{Digest, Sha256};

    let mut file = File::open(path).map_err(IoError::from_io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(IoError::from_io)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Ok(hex)
}
