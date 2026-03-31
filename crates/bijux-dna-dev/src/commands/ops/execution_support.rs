use super::{
    fs, merge_outcomes, Context, OpsCommandOutcome, Path, Result, Value, WalkDir, Workspace,
};
use crate::runtime::process::ProcessRunner;

pub(super) fn run_programs_with_env(
    workspace: &Workspace,
    commands: &[(&str, Vec<&str>)],
    envs: &[(String, String)],
) -> Result<OpsCommandOutcome> {
    let mut aggregate = OpsCommandOutcome::success(String::new());
    for (program, args) in commands {
        let outcome = run_program_with_env(
            workspace,
            program,
            &args
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>(),
            envs,
        )?;
        aggregate = merge_outcomes(aggregate, outcome);
        if !aggregate.is_success() {
            return Ok(aggregate);
        }
    }
    Ok(aggregate)
}

pub(super) fn walk_file_list(
    workspace: &Workspace,
    root: &str,
    extension: Option<&str>,
) -> Result<String> {
    let mut files = WalkDir::new(workspace.path(root))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            extension.is_none()
                || entry.path().extension().and_then(|ext| ext.to_str()) == extension
        })
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();
    Ok(format!("{}\n", files.join("\n")))
}

pub(super) fn run_program(
    workspace: &Workspace,
    program: &str,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    run_program_with_env(workspace, program, args, &[])
}

pub(super) fn run_program_with_env(
    workspace: &Workspace,
    program: &str,
    args: &[String],
    envs: &[(String, String)],
) -> Result<OpsCommandOutcome> {
    let runner = ProcessRunner::new(workspace);
    let output = runner.run_owned_with_env(program, args, envs)?;
    Ok(OpsCommandOutcome::from_output(output))
}

pub(super) fn read_utf8(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

pub(super) fn write_utf8(path: &Path, raw: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(path, raw).with_context(|| format!("write {}", path.display()))
}

pub(super) fn write_json_pretty(path: &Path, value: &Value) -> Result<()> {
    write_utf8(path, &format!("{}\n", serde_json::to_string_pretty(value)?))
}

pub(super) fn read_json_value(path: &Path) -> Result<Value> {
    serde_json::from_str(&read_utf8(path)?).with_context(|| format!("parse {}", path.display()))
}

pub(super) fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE")
    )
}
