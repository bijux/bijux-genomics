use super::{anyhow, sha256_hex, Context, Path, PathBuf, Result, Workspace};
use crate::model::ops::OpsCommandOutcome;
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
    expected_size_bytes: Option<u64>,
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
        if let Some(expected) = expected_size_bytes {
            let observed =
                std::fs::metadata(path).with_context(|| format!("inspect existing {label}"))?.len();
            if observed != expected {
                return Err(anyhow!(
                    "refusing to replace size-mismatched {label}: expected {expected} bytes, got {observed}"
                ));
            }
        }
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
    let observed = download_verified_file(path, url, expected_sha256, expected_size_bytes, label)?;
    Ok(MaterializedFile { action: "download".to_string(), observed_sha256: observed })
}

pub(super) fn admit_controlled_file(
    source: &Path,
    path: &Path,
    expected_sha256: &str,
    expected_size_bytes: u64,
    label: &str,
) -> Result<MaterializedFile> {
    if path.exists() {
        let observed_size =
            std::fs::metadata(path).with_context(|| format!("inspect existing {label}"))?.len();
        if observed_size != expected_size_bytes {
            return Err(anyhow!(
                "refusing to replace size-mismatched {label}: expected {expected_size_bytes} bytes, got {observed_size}"
            ));
        }
        let observed = sha256_hex(path)?;
        if observed != expected_sha256 {
            return Err(anyhow!(
                "refusing to replace checksum-mismatched {label}: expected {expected_sha256}, got {observed}"
            ));
        }
        return Ok(MaterializedFile { action: "reuse".to_string(), observed_sha256: observed });
    }
    if !source.is_file() {
        return Err(anyhow!("{label} admission source is not a file: {}", source.display()));
    }
    let source_size = std::fs::metadata(source)
        .with_context(|| format!("inspect {label} admission source"))?
        .len();
    if source_size != expected_size_bytes {
        return Err(anyhow!(
            "refusing size-mismatched {label} admission: expected {expected_size_bytes} bytes, got {source_size}"
        ));
    }
    let source_sha256 = sha256_hex(source)?;
    if source_sha256 != expected_sha256 {
        return Err(anyhow!(
            "refusing checksum-mismatched {label} admission: expected {expected_sha256}, got {source_sha256}"
        ));
    }
    let mut input = std::fs::File::open(source)
        .with_context(|| format!("open {label} admission source {}", source.display()))?;
    bijux_dna_infra::atomic_create_with(path, |output| {
        std::io::copy(&mut input, output)?;
        Ok(())
    })
    .with_context(|| format!("admit {label} to {}", path.display()))?;
    let observed = sha256_hex(path)?;
    if observed != expected_sha256 {
        return Err(anyhow!(
            "admitted {label} checksum changed during copy: expected {expected_sha256}, got {observed}"
        ));
    }
    Ok(MaterializedFile { action: "admit".to_string(), observed_sha256: observed })
}

fn download_verified_file(
    path: &Path,
    url: &str,
    expected_sha256: &str,
    expected_size_bytes: Option<u64>,
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
        bijux_dna_infra::atomic_create_with(&partial, |output| {
            std::io::copy(&mut response, output)?;
            output.flush()?;
            output.sync_all()
        })
        .with_context(|| format!("stream {url} to {}", partial.display()))?;

        let observed_size = std::fs::metadata(&partial)
            .with_context(|| format!("inspect {}", partial.display()))?
            .len();
        if let Some(expected) = expected_size_bytes {
            if expected != observed_size {
                return Err(anyhow!(
                    "size mismatch for {label}: expected {expected} bytes, got {observed_size}"
                ));
            }
        }

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
    workspace.admit_artifact_path(Path::new(&configured))
}

pub(super) fn ensure_artifact_root_inside_artifacts(workspace: &Workspace) -> Result<()> {
    artifact_root_path(workspace)?;
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
    if let Some(wrapper) = compiler_wrapper_env(workspace)? {
        envs.push(wrapper);
    }
    Ok(envs)
}

pub(super) fn compiler_wrapper_env(workspace: &Workspace) -> Result<Option<(String, String)>> {
    let explicit = match std::env::var("RUSTC_WRAPPER") {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(anyhow!("RUSTC_WRAPPER is not valid UTF-8"));
        }
    };
    let discovered = if explicit.is_none() {
        std::process::Command::new("sh")
            .args(["-c", "command -v sccache || true"])
            .current_dir(&workspace.root)
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    };
    Ok(select_compiler_wrapper(explicit, discovered))
}

fn select_compiler_wrapper(
    explicit: Option<String>,
    discovered: Option<String>,
) -> Option<(String, String)> {
    explicit
        .or_else(|| discovered.filter(|value| !value.is_empty()))
        .map(|value| ("RUSTC_WRAPPER".to_string(), value))
}

pub(super) fn run_make_target(workspace: &Workspace, target: &str) -> Result<OpsCommandOutcome> {
    super::run_program_with_env(workspace, "make", &[target.to_string()], &artifact_env(workspace)?)
}

pub(super) fn resolve_workspace_path(workspace: &Workspace, raw: &str) -> PathBuf {
    path_from_arg(workspace, raw)
}

#[cfg(test)]
mod tests {
    use super::{admit_controlled_file, materialize_controlled_file, select_compiler_wrapper};

    #[test]
    fn explicit_empty_compiler_wrapper_disables_discovered_cache() {
        assert_eq!(
            select_compiler_wrapper(Some(String::new()), Some("/usr/bin/sccache".to_string())),
            Some(("RUSTC_WRAPPER".to_string(), String::new()))
        );
    }

    #[test]
    fn compiler_wrapper_discovery_ignores_empty_results() {
        assert_eq!(select_compiler_wrapper(None, Some(String::new())), None);
        assert_eq!(
            select_compiler_wrapper(None, Some("/usr/bin/sccache".to_string())),
            Some(("RUSTC_WRAPPER".to_string(), "/usr/bin/sccache".to_string()))
        );
    }

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
            None,
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
            None,
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

    #[test]
    fn controlled_file_admission_copies_only_checksum_verified_bytes() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("panel.msav.source");
        let destination = root.path().join("derived/panel.msav");
        std::fs::write(&source, b"checksum-bound derived panel")?;
        let checksum = super::sha256_hex(&source)?;

        let admitted = admit_controlled_file(
            &source,
            &destination,
            &checksum,
            std::fs::metadata(&source)?.len(),
            "panel_msav",
        )?;

        assert_eq!(admitted.action, "admit");
        assert_eq!(std::fs::read(destination)?, b"checksum-bound derived panel");
        Ok(())
    }
}
