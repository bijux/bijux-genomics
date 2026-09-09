//! Test process adaptation for explicit archived workspace executables.

use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::Command;

pub fn workspace_binary_command(root: &Path, package: &str, binary: &str) -> io::Result<Command> {
    let selected = std::env::var_os(format!("NEXTEST_BIN_EXE_{binary}"));
    command_with_path(root, package, binary, selected.as_deref())
}

pub(crate) fn command_with_path(
    root: &Path,
    package: &str,
    binary: &str,
    selected: Option<&OsStr>,
) -> io::Result<Command> {
    let mut command = if let Some(selected) = selected {
        let path = Path::new(selected);
        if !path.is_absolute() || !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "configured workspace executable {binary} is not an absolute file: {}",
                    path.display()
                ),
            ));
        }
        Command::new(path)
    } else {
        // Outside archive execution Cargo remains the freshness authority. An existing debug
        // file alone cannot establish source, feature, profile or toolchain compatibility.
        let mut command = Command::new("cargo");
        command.args(["run", "--locked", "-q", "-p", package, "--bin", binary, "--"]);
        command
    };
    command.current_dir(root);
    Ok(command)
}
