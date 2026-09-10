//! Capture code-freeze filesystem and process observations for the bundle model.
use crate::command_runner::run_command;
use anyhow::{Context, Result};
use bijux_dna_bench_model::HpcCodeFreezeObservation;
use std::path::Path;

/// Observe the current script and repository tooling without interpreting scientific readiness.
/// # Errors
/// Returns script acquisition errors. Failed Git/DVC observations preserve existing missingness.
pub fn observe_hpc_code_freeze(script_path: &Path) -> Result<HpcCodeFreezeObservation> {
    let slurm_script = std::fs::read_to_string(script_path)
        .with_context(|| format!("read {}", script_path.display()))?;
    Ok(HpcCodeFreezeObservation {
        slurm_script,
        git_head: git_stdout(&["rev-parse", "HEAD"]),
        git_branch: git_stdout(&["rev-parse", "--abbrev-ref", "HEAD"]),
        git_status_porcelain: git_stdout(&["status", "--porcelain"]),
        dvc_available: run_command("dvc", &["--version".to_string()])
            .is_ok_and(|output| output.exit_code == 0),
    })
}

fn git_stdout(args: &[&str]) -> Option<String> {
    let arg_vec = args.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let output = run_command("git", &arg_vec).ok()?;
    if output.exit_code != 0 {
        return None;
    }
    Some(output.stdout.trim().to_string())
}
