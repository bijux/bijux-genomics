use std::path::Path;

use anyhow::Result;
use bijux_dna_environment::api::RuntimeKind;

use crate::command_runner::{run_command, run_command_with_context, CommandOutputV1};

use super::runtime_policy::network_allowed;
use super::{runner_failure, RunnerEffectKind};

/// Execute a lightweight observer command using docker.
///
/// # Errors
/// Returns an error if execution fails or docker is unavailable.
pub fn execute_observer_command(
    image: &str,
    mount_dir: &Path,
    args: &[String],
    runner: RuntimeKind,
) -> Result<CommandOutputV1> {
    let mount_dir = mount_dir
        .canonicalize()
        .map_err(|err| runner_failure(RunnerEffectKind::Filesystem, err.to_string()))?;
    if runner == RuntimeKind::Local {
        let Some((command, command_args)) = args.split_first() else {
            return Err(runner_failure(
                RunnerEffectKind::CommandSpawn,
                "local observer command args must include an executable".to_string(),
            ));
        };
        return run_command_with_context(command, command_args, Some(&mount_dir), None)
            .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()));
    }
    let (bin, command_args) = build_observer_command_args(image, &mount_dir, args, runner);
    let output = run_command(bin, &command_args)
        .map_err(|err| runner_failure(RunnerEffectKind::CommandSpawn, err.to_string()))?;
    Ok(output)
}

pub(super) fn build_observer_command_args(
    image: &str,
    mount_dir: &Path,
    args: &[String],
    runner: RuntimeKind,
) -> (&'static str, Vec<String>) {
    let mount_arg = format!("{}:/data:ro", mount_dir.display());
    match runner {
        RuntimeKind::Local => ("sh", vec!["-c".to_string(), "true".to_string()]),
        RuntimeKind::Docker => {
            let mut command_args: Vec<String> = vec!["run".to_string(), "--rm".to_string()];
            if !network_allowed() {
                command_args.push("--network".to_string());
                command_args.push("none".to_string());
            }
            command_args.extend(["-v".to_string(), mount_arg, image.to_string()]);
            command_args.extend(args.iter().cloned());
            ("docker", command_args)
        }
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            let mut command_args = vec![
                "exec".to_string(),
                "--cleanenv".to_string(),
                "--no-home".to_string(),
                "--containall".to_string(),
                "--bind".to_string(),
                mount_arg,
            ];
            command_args.push(image.to_string());
            command_args.extend(args.iter().cloned());
            let bin = if runner == RuntimeKind::Apptainer { "apptainer" } else { "singularity" };
            (bin, command_args)
        }
    }
}
