use std::io;
use std::path::{Path, PathBuf};

/// Resolve a finalized workspace executable without compiling or launching it.
///
/// `NEXTEST_BIN_EXE_<binary>` takes precedence over the caller's Cargo-provided
/// executable. Callers must bind a binary built from their source and feature
/// selection before parallel workers start. The Cargo fallback is appropriate
/// only when no concurrent producer can replace that executable.
///
/// # Errors
/// Returns an error if the selected path is relative, missing, not a regular
/// file, or lacks executable permission on Unix. An invalid explicit binding
/// never falls back to another executable.
pub fn workspace_binary_path(binary: &str, compiled_path: &Path) -> io::Result<PathBuf> {
    let variable = format!("NEXTEST_BIN_EXE_{binary}");
    let selected = std::env::var_os(&variable);
    let path = selected.map_or_else(|| compiled_path.to_path_buf(), PathBuf::from);
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{variable} executable must be absolute: {}", path.display()),
        ));
    }
    let metadata = path.metadata().map_err(|error| {
        io::Error::new(error.kind(), format!("resolve {variable} {}: {error}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{variable} executable must be a regular file: {}", path.display()),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("{variable} executable lacks executable permission: {}", path.display()),
            ));
        }
    }
    Ok(path)
}
