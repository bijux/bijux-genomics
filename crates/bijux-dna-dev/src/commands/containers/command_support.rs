use std::fmt::Write as _;

#[allow(clippy::wildcard_imports)]
use super::*;

pub(super) fn run_container_runtime_check() -> Result<ContainerCommandOutcome> {
    let system_type = std::env::var("SYSTEM_TYPE").unwrap_or_else(|_| "local".to_string());
    let container_type = checked_container_type()?;
    Ok(ContainerCommandOutcome::success(format!(
        "SYSTEM_TYPE={system_type} CONTAINER_TYPE={container_type}\n"
    )))
}

#[allow(clippy::unnecessary_wraps)]
pub(super) fn success_line(line: impl Into<String>) -> Result<ContainerCommandOutcome> {
    Ok(ContainerCommandOutcome::success(format!("{}\n", line.into())))
}

#[allow(clippy::unnecessary_wraps)]
pub(super) fn failure_lines(title: &str, errors: &[String]) -> Result<ContainerCommandOutcome> {
    let mut stderr = String::new();
    stderr.push_str(title);
    stderr.push('\n');
    for error in errors {
        stderr.push_str(error);
        if !error.ends_with('\n') {
            stderr.push('\n');
        }
    }
    Ok(ContainerCommandOutcome::failure(stderr))
}

pub(super) fn read_utf8(path: &std::path::Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

pub(super) fn write_utf8(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(path, content).with_context(|| format!("write {}", path.display()))
}

pub(super) fn append_named_outcome(
    aggregate: &mut ContainerCommandOutcome,
    name: &str,
    #[allow(clippy::needless_pass_by_value)] outcome: ContainerCommandOutcome,
) {
    let _ = writeln!(aggregate.stdout, "== {name}");
    *aggregate = merge_outcomes(aggregate.clone(), &outcome);
}

pub(super) fn iso_root_path(workspace: &Workspace) -> PathBuf {
    PathBuf::from(
        std::env::var("ISO_ROOT")
            .unwrap_or_else(|_| workspace.path("artifacts").display().to_string()),
    )
}

pub(super) fn iso_run_id() -> String {
    env_or_default("ISO_RUN_ID", "run")
}

pub(super) fn policy_path(workspace: &Workspace, env_key: &str, default_rel: &str) -> PathBuf {
    std::env::var(env_key).map_or_else(|_| workspace.path(default_rel), PathBuf::from)
}

pub(super) fn read_json(path: &std::path::Path) -> Result<serde_json::Value> {
    serde_json::from_str(&read_utf8(path)?)
        .with_context(|| format!("parse JSON {}", path.display()))
}

pub(super) fn json_string_pretty(value: &serde_json::Value) -> Result<String> {
    Ok(format!("{}\n", serde_json::to_string_pretty(value)?))
}

pub(super) fn git_last_modified_timestamp(workspace: &Workspace, rel_path: &str) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args(["log", "-1", "--format=%cI", "--", rel_path])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

pub(super) fn git_is_shallow_repository(workspace: &Workspace) -> bool {
    std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args(["rev-parse", "--is-shallow-repository"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().eq("true"))
        .unwrap_or(false)
}

pub(super) fn out_path_arg(
    workspace: &Workspace,
    args: &[String],
    default_rel: &str,
    usage: &str,
) -> Result<PathBuf> {
    match args {
        [] => Ok(workspace.path(default_rel)),
        [single] if single == "--help" || single == "-h" => Err(anyhow!(usage.to_string())),
        [single] => Ok(path_from_arg(workspace, single)),
        _ => Err(anyhow!(usage.to_string())),
    }
}

pub(super) fn path_from_arg(workspace: &Workspace, arg: &str) -> PathBuf {
    let path = PathBuf::from(arg);
    if path.is_absolute() {
        path
    } else {
        workspace.root.join(path)
    }
}
