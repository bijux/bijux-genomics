use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_bench::compare::compare_runs;
use bijux_core::run_index::{insert_run, RunIndexEntry};
use bijux_engine::api::ExecutionManifest;

#[test]
fn bench_compare_snapshot() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    let root = repo_root
        .join("target")
        .join("test-fixtures")
        .join("bench_compare");
    let run_a_dir = root.join("run-a");
    let run_b_dir = root.join("run-b");
    fs::create_dir_all(&run_a_dir)?;
    fs::create_dir_all(&run_b_dir)?;

    let manifest_a = ExecutionManifest {
        run_id: "run-a".to_string(),
        stage: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: "sha256:abc".to_string(),
        command: "fastp -i input -o output".to_string(),
        input_hashes: vec!["ih".to_string()],
        input_files: vec!["input.fastq.gz".to_string()],
        output_dir: run_a_dir.display().to_string(),
        runner: "docker".to_string(),
        platform: "local".to_string(),
        arch: "arm64".to_string(),
    };
    let manifest_b = ExecutionManifest {
        run_id: "run-b".to_string(),
        stage: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: "sha256:abc".to_string(),
        command: "fastp -i input -o output".to_string(),
        input_hashes: vec!["ih".to_string()],
        input_files: vec!["input.fastq.gz".to_string()],
        output_dir: run_b_dir.display().to_string(),
        runner: "docker".to_string(),
        platform: "local".to_string(),
        arch: "arm64".to_string(),
    };

    fs::write(
        run_a_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest_a)?,
    )?;
    fs::write(
        run_b_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest_b)?,
    )?;
    fs::write(
        run_a_dir.join("metrics.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "runtime_s": 1.0,
            "memory_mb": 100.0
        }))?,
    )?;
    fs::write(
        run_b_dir.join("metrics.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "runtime_s": 2.0,
            "memory_mb": 120.0
        }))?,
    )?;

    let run_index = root.join("run_index.jsonl");
    insert_run(
        &run_index,
        &RunIndexEntry {
            run_id: "run-a".to_string(),
            domain: "fastq".to_string(),
            pipeline: "bench".to_string(),
            stages: vec!["fastq.trim".to_string()],
            tools: vec!["fastp".to_string()],
            objective: None,
            platform: "local".to_string(),
            success: true,
        },
    )?;
    insert_run(
        &run_index,
        &RunIndexEntry {
            run_id: "run-b".to_string(),
            domain: "fastq".to_string(),
            pipeline: "bench".to_string(),
            stages: vec!["fastq.trim".to_string()],
            tools: vec!["fastp".to_string()],
            objective: None,
            platform: "local".to_string(),
            success: true,
        },
    )?;

    let comparison = compare_runs("run-a", "run-b", &run_index, &root)?;
    let rendered = serde_json::to_string_pretty(&comparison)?;
    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("bench_compare.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
