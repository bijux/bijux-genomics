use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(|p| p.parent()) {
        Some(root) => Ok(root.to_path_buf()),
        None => Err("repo root not found".into()),
    }
}

fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "expected file to exist: {}", path.display());
}

#[test]
fn fastq_trim_emits_contract_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }

    let root = repo_root()?;
    let artifacts = root.join("artifacts");
    let r1 = root.join("tests/data/fastq/ERR769587/ERR769587.fastq.gz");

    let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
    cmd.current_dir(&root).args([
        "bench",
        "fastq",
        "trim",
        "--sample-id",
        "ERR769587",
        "--r1",
        r1.to_str().ok_or("invalid utf-8 path")?,
        "--out",
        artifacts.to_str().ok_or("invalid utf-8 path")?,
        "--tools",
        "fastp",
    ]);
    cmd.assert().success();

    let run_root = artifacts
        .join("bench")
        .join("trim")
        .join("ERR769587")
        .join("fastp");
    let mut run_iter = run_root.join("run").read_dir()?;
    let run_dir = run_iter.next().ok_or("missing run entry")??.path();

    assert_file_exists(&run_dir.join("run_manifest.json"));
    assert_file_exists(&run_dir.join("metrics.json"));
    assert_file_exists(&run_dir.join("retention_report.json"));
    assert_file_exists(&run_dir.join("run_artifacts/adapters/effective_adapters.json"));
    assert_file_exists(&run_dir.join("run_artifacts/adapters/adapter_bank_ref.json"));
    assert_file_exists(&run_dir.join("run_artifacts/reports/adapter_trimming_report.json"));

    let manifest_data = fs::read_to_string(run_dir.join("run_manifest.json"))?;
    assert!(
        manifest_data.contains("adapter_bank"),
        "adapter bank ref missing from run_manifest"
    );
    assert!(
        manifest_data.contains("effective_adapters"),
        "effective adapters missing from run_manifest"
    );
    assert!(
        manifest_data.contains("adapter_bank_ref"),
        "adapter bank ref missing from run_manifest"
    );
    assert!(
        manifest_data.contains("adapter_trimming_report"),
        "adapter trimming report missing from run_manifest"
    );

    let run_index_dir = artifacts.join("runs");
    let mut found = None;
    if let Ok(entries) = run_index_dir.read_dir() {
        for entry in entries.flatten() {
            let candidate = entry.path().join("execution_manifest.json");
            if candidate.exists() {
                found = Some(candidate);
                break;
            }
        }
    }
    let run_manifest_path = found.ok_or("execution_manifest.json not found under runs/")?;
    assert_file_exists(&run_manifest_path);
    Ok(())
}
