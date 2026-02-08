use std::process::Command;

use super::{
    available_runners_with, docker_image_exists_with, EnvError, ResolvedImage, RuntimeKind,
};

/// List available runners based on local command probes.
///
/// # Errors
/// Returns an error if probing cannot be performed.
pub fn available_runners() -> Result<Vec<RuntimeKind>, EnvError> {
    Ok(available_runners_with(probe_command))
}

#[must_use]
pub fn docker_image_exists(image: &ResolvedImage) -> bool {
    docker_image_exists_with(image, |args| {
        Command::new("docker")
            .args(args)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}

fn probe_command(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub(crate) fn run_command(cmd: &str, args: &[&str]) -> Result<(), EnvError> {
    let status = Command::new(cmd).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(EnvError::Platform(format!(
            "command failed: {cmd} {args:?}"
        )))
    }
}
