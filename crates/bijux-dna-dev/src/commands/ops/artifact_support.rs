use super::{anyhow, sha256_hex, Context, Path, PathBuf, Result, Workspace};
use crate::model::ops::OpsCommandOutcome;

pub(super) struct MaterializedFile {
    pub(super) action: String,
    pub(super) observed_sha256: String,
}

pub(super) fn materialize_controlled_file(
    path: &Path,
    url: &str,
    expected_sha256: &str,
    synthetic_bytes: &[u8],
    download: bool,
    verbose: bool,
    label: &str,
    stdout: &mut String,
) -> Result<MaterializedFile> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let mut action = "reuse".to_string();
    let mut observed = if path.exists() { sha256_hex(path)? } else { String::new() };
    if path.exists() {
        if observed != expected_sha256 && download {
            action = "redownload".to_string();
            if verbose {
                stdout.push_str(&format!("[download] {label} <- {url}\n"));
            }
            bijux_dna_infra::write_bytes(path, download_bytes(url)?)
                .with_context(|| format!("write {}", path.display()))?;
            observed = sha256_hex(path)?;
        } else if observed != expected_sha256 {
            action = "rewrite-synthetic".to_string();
            bijux_dna_infra::write_bytes(path, synthetic_bytes)
                .with_context(|| format!("write {}", path.display()))?;
            observed = sha256_hex(path)?;
        }
    } else if download {
        action = "download".to_string();
        if verbose {
            stdout.push_str(&format!("[download] {label} <- {url}\n"));
        }
        bijux_dna_infra::write_bytes(path, download_bytes(url)?)
            .with_context(|| format!("write {}", path.display()))?;
        observed = sha256_hex(path)?;
    } else {
        action = "write-synthetic".to_string();
        bijux_dna_infra::write_bytes(path, synthetic_bytes)
            .with_context(|| format!("write {}", path.display()))?;
        observed = sha256_hex(path)?;
    }
    if observed != expected_sha256 {
        return Err(anyhow!(
            "checksum mismatch for {label}: expected {expected_sha256}, got {observed}"
        ));
    }
    Ok(MaterializedFile { action, observed_sha256: observed })
}

fn download_bytes(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::blocking::get(url)
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    Ok(response.bytes()?.to_vec())
}

pub(super) fn sha256_hex_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn path_from_arg(workspace: &Workspace, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        candidate
    } else {
        workspace.root.join(candidate)
    }
}

pub(super) fn artifact_root_path(workspace: &Workspace) -> Result<PathBuf> {
    let configured = std::env::var("ARTIFACT_ROOT")
        .unwrap_or_else(|_| std::env::var("ISO_ROOT").unwrap_or_else(|_| "artifacts".to_string()));
    let path = if PathBuf::from(&configured).is_absolute() {
        PathBuf::from(&configured)
    } else {
        workspace.root.join(&configured)
    };
    Ok(path)
}

pub(super) fn ensure_artifact_root_inside_artifacts(workspace: &Workspace) -> Result<()> {
    let display = artifact_root_path(workspace)?.display().to_string();
    if !display.contains("/artifacts") && !display.ends_with("artifacts") {
        return Err(anyhow!("artifact root must stay under artifacts/: {display}"));
    }
    Ok(())
}

pub(super) fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = artifact_root_path(workspace)?;
    let cargo_target_dir = artifact_root.join("target");
    for dir in [&artifact_root, &cargo_target_dir] {
        bijux_dna_infra::ensure_dir(dir)?;
    }
    Ok(vec![
        ("ARTIFACT_ROOT".to_string(), artifact_root.display().to_string()),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        ("CARGO_TARGET_DIR".to_string(), cargo_target_dir.display().to_string()),
    ])
}

pub(super) fn artifact_env_with_common_test_env(
    workspace: &Workspace,
) -> Result<Vec<(String, String)>> {
    let mut envs = artifact_env(workspace)?;
    envs.push(("TZ".to_string(), "UTC".to_string()));
    envs.push(("LC_ALL".to_string(), "C".to_string()));
    if let Ok(value) = std::env::var("CARGO_TARGET_DIR") {
        if !value.trim().is_empty() {
            envs.push(("CARGO_TARGET_DIR".to_string(), value));
        }
    }
    if let Ok(output) = std::process::Command::new("sh")
        .args(["-c", "command -v sccache || true"])
        .current_dir(&workspace.root)
        .output()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            envs.push(("RUSTC_WRAPPER".to_string(), path));
        }
    }
    Ok(envs)
}

pub(super) fn run_make_target(workspace: &Workspace, target: &str) -> Result<OpsCommandOutcome> {
    super::run_program_with_env(workspace, "make", &[target.to_string()], &artifact_env(workspace)?)
}

pub(super) fn resolve_workspace_path(workspace: &Workspace, raw: &str) -> PathBuf {
    path_from_arg(workspace, raw)
}
