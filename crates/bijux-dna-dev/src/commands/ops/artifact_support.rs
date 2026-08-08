use super::{anyhow, sha256_hex, Context, Path, PathBuf, Result, Workspace};
use crate::model::ops::OpsCommandOutcome;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

static PARTIAL_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(super) struct MaterializedFile {
    pub(super) action: String,
    pub(super) observed_sha256: String,
}

pub(super) fn materialize_controlled_file(
    path: &Path,
    url: &str,
    expected_sha256: &str,
    download: bool,
    verbose: bool,
    label: &str,
    stdout: &mut String,
) -> Result<MaterializedFile> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    if path.exists() {
        let observed = sha256_hex(path)?;
        if observed != expected_sha256 {
            return Err(anyhow!(
                "refusing to replace checksum-mismatched {label}: expected {expected_sha256}, got {observed}"
            ));
        }
        return Ok(MaterializedFile { action: "reuse".to_string(), observed_sha256: observed });
    }

    if !download {
        return Err(anyhow!(
            "{label} is not materialized; pass --download to acquire checksum-verified source bytes"
        ));
    }
    if verbose {
        stdout.push_str(&format!("[download] {label} <- {url}\n"));
    }
    let observed = download_verified_file(path, url, expected_sha256, label)?;
    Ok(MaterializedFile { action: "download".to_string(), observed_sha256: observed })
}

fn download_verified_file(
    path: &Path,
    url: &str,
    expected_sha256: &str,
    label: &str,
) -> Result<String> {
    let sequence = PARTIAL_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .context("acquisition target has no UTF-8 file name")?;
    let partial =
        path.with_file_name(format!(".{file_name}.partial-{}-{sequence}", std::process::id()));
    let result = (|| -> Result<String> {
        let mut response = reqwest::blocking::get(url)
            .with_context(|| format!("download {url}"))?
            .error_for_status()
            .with_context(|| format!("download {url}"))?;
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&partial)
            .with_context(|| format!("create partial acquisition {}", partial.display()))?;
        std::io::copy(&mut response, &mut output)
            .with_context(|| format!("stream {url} to {}", partial.display()))?;
        output.flush().with_context(|| format!("flush {}", partial.display()))?;
        output.sync_all().with_context(|| format!("sync {}", partial.display()))?;
        drop(output);

        let observed = sha256_hex(&partial)?;
        if observed != expected_sha256 {
            return Err(anyhow!(
                "checksum mismatch for {label}: expected {expected_sha256}, got {observed}"
            ));
        }
        std::fs::hard_link(&partial, path)
            .with_context(|| format!("publish verified acquisition {}", path.display()))?;
        std::fs::remove_file(&partial)
            .with_context(|| format!("remove partial acquisition {}", partial.display()))?;
        Ok(observed)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
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

#[cfg(test)]
mod tests {
    use super::materialize_controlled_file;

    #[test]
    fn controlled_file_refuses_placeholder_materialization() -> anyhow::Result<()> {
        let root = std::env::temp_dir()
            .join(format!("bijux-controlled-file-missing-{}", std::process::id()));
        let path = root.join("reference.fa.gz");
        let mut stdout = String::new();

        let error = match materialize_controlled_file(
            &path,
            "https://example.invalid/reference.fa.gz",
            &"a".repeat(64),
            false,
            false,
            "reference",
            &mut stdout,
        ) {
            Ok(_) => anyhow::bail!("missing production assets must not become placeholders"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("pass --download"));
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn controlled_file_preserves_mismatched_existing_bytes() -> anyhow::Result<()> {
        let root = std::env::temp_dir()
            .join(format!("bijux-controlled-file-existing-{}", std::process::id()));
        std::fs::create_dir_all(&root)?;
        let path = root.join("reference.fa.gz");
        std::fs::write(&path, b"existing evidence")?;
        let mut stdout = String::new();

        let error = match materialize_controlled_file(
            &path,
            "https://example.invalid/reference.fa.gz",
            &"b".repeat(64),
            true,
            false,
            "reference",
            &mut stdout,
        ) {
            Ok(_) => anyhow::bail!("mismatched evidence must not be replaced"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("refusing to replace"));
        assert_eq!(std::fs::read(path)?, b"existing evidence");
        Ok(())
    }
}
