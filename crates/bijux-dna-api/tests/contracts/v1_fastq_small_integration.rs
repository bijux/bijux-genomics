use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("workspace root not found"))
}

fn run_fastq_toy(out_dir: &Path) -> Result<PathBuf> {
    let root = repo_root()?;
    let status = Command::new(root.join("scripts/test/toy_runs.sh"))
        .arg("run")
        .arg("--profile")
        .arg("fastq")
        .arg("--out")
        .arg(out_dir)
        .current_dir(&root)
        .status()
        .context("run toy fastq profile")?;
    if !status.success() {
        return Err(anyhow!("toy fastq run failed with status {status}"));
    }
    Ok(out_dir.join("fastq_reference_adna"))
}

#[test]
fn fastq_small_pipeline_emits_multi_stage_manifest() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let run_dir = run_fastq_toy(temp.path())?;
    let manifest_path = run_dir.join("manifest.json");
    let checksums_path = run_dir.join("artifact_checksums.json");
    let metrics_path = run_dir.join("metrics.json");
    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&raw)?;
    assert_eq!(
        manifest
            .get("profile_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default(),
        "fastq_reference_adna"
    );
    let checksums: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&checksums_path)?)?;
    let artifacts = checksums
        .get("artifacts")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow!("artifact_checksums missing artifacts object"))?;
    assert!(
        (3..=5).contains(&artifacts.len()),
        "expected 3-5 stable artifacts, got {}",
        artifacts.len()
    );
    let metrics: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&metrics_path)?)?;
    for key in ["reads_total", "bases_total", "retention_ratio"] {
        assert!(
            metrics.get(key).is_some(),
            "metrics.json missing required key `{key}`"
        );
    }
    Ok(())
}

#[test]
fn fastq_small_pipeline_is_reproducible_between_two_runs() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let run_a = run_fastq_toy(&temp.path().join("run_a"))?;
    let run_b = run_fastq_toy(&temp.path().join("run_b"))?;

    let checksums_a = std::fs::read_to_string(run_a.join("artifact_checksums.json"))?;
    let checksums_b = std::fs::read_to_string(run_b.join("artifact_checksums.json"))?;
    assert_eq!(
        checksums_a, checksums_b,
        "artifact checksum drift detected between repeated fastq toy runs"
    );
    Ok(())
}
