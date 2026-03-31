use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;

use super::command_template::container_command_template;
use super::inputs::{input_bind_roots, preserve_absolute_input_paths};
use super::runtime_policy::stage_workdir_in_container;
use super::{runner_failure, RunnerEffectKind};

pub(super) fn build_apptainer_exec_args(
    step: &ExecutionStep,
    inputs: &[PathBuf],
    input_root: &Path,
    out_dir: &Path,
    _runner: RuntimeKind,
) -> Result<Vec<String>> {
    let preserve_absolute_inputs = preserve_absolute_input_paths(inputs);
    let bind_roots = input_bind_roots(inputs, input_root, preserve_absolute_inputs);
    let output_mount = format!("{}:/data/output", out_dir.display());
    let mut args: Vec<String> = vec![
        "exec".to_string(),
        "--cleanenv".to_string(),
        "--no-home".to_string(),
        "--containall".to_string(),
    ];
    for bind_root in bind_roots {
        let input_mount = if preserve_absolute_inputs {
            format!("{}:{}:ro", bind_root.display(), bind_root.display())
        } else {
            format!("{}:/data/input:ro", bind_root.display())
        };
        args.push("--bind".to_string());
        args.push(input_mount);
    }
    args.push("--bind".to_string());
    args.push(output_mount);
    args.push("--pwd".to_string());
    args.push(stage_workdir_in_container(out_dir, RuntimeKind::Apptainer));
    args.push(step.image.image.clone());
    args.extend(container_command_template(
        &step.command.template,
        input_root,
        out_dir,
        preserve_absolute_inputs,
    ));
    if args.is_empty() {
        return Err(runner_failure(
            RunnerEffectKind::CommandSpawn,
            "apptainer/singularity command args are empty",
        ));
    }
    Ok(args)
}
