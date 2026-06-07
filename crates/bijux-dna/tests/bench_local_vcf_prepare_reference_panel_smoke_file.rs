#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_local_vcf_prepare_reference_panel_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-prepare-reference-panel-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz"
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_vcf =
        repo_root.join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz");
    let output_tbi = repo_root
        .join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz.tbi");
    let metrics_path =
        repo_root.join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/metrics.json");
    let panel_manifest_path = repo_root
        .join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel_manifest.json");
    let overlap_path =
        repo_root.join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/overlap.json");
    let panel_overlap_path = repo_root
        .join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel_overlap.json");
    let panel_files_path = repo_root
        .join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel_files.json");
    let overlap_tsv_path =
        repo_root.join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/overlap.tsv");
    let chunks_path =
        repo_root.join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/chunks.json");
    let manifest_path = repo_root
        .join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/stage-result.json");
    let input_vcf = repo_root.join(
        "runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/artifacts/input/prepare_reference_panel_input.vcf",
    );

    assert!(output_vcf.is_file(), "expected output VCF at {}", output_vcf.display());
    assert!(output_tbi.is_file(), "expected output index at {}", output_tbi.display());
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(
        panel_manifest_path.is_file(),
        "expected panel manifest at {}",
        panel_manifest_path.display()
    );
    assert!(overlap_path.is_file(), "expected overlap json at {}", overlap_path.display());
    assert!(
        panel_overlap_path.is_file(),
        "expected panel overlap json at {}",
        panel_overlap_path.display()
    );
    assert!(
        panel_files_path.is_file(),
        "expected panel files json at {}",
        panel_files_path.display()
    );
    assert!(overlap_tsv_path.is_file(), "expected overlap tsv at {}", overlap_tsv_path.display());
    assert!(chunks_path.is_file(), "expected chunks json at {}", chunks_path.display());
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(input_vcf.is_file(), "expected synthetic input at {}", input_vcf.display());

    let panel_manifest_raw =
        std::fs::read_to_string(&panel_manifest_path).expect("read panel manifest");
    let panel_manifest: serde_json::Value =
        serde_json::from_str(&panel_manifest_raw).expect("parse panel manifest");
    assert_eq!(
        panel_manifest.pointer("/normalization/status").and_then(serde_json::Value::as_str),
        Some("sorted_indexed_deduplicated")
    );
    assert_eq!(
        panel_manifest
            .pointer("/normalization/input_variant_count")
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        panel_manifest
            .pointer("/normalization/output_variant_count")
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        panel_manifest
            .pointer("/normalization/duplicate_sites_removed")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        manifest.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.prepare_reference_panel")
    );
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("bcftools")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-prepare-reference-panel-smoke --tool-id bcftools")
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 9);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("prepared_panel_vcf")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("panel_manifest_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some(
                    "runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel_manifest.json",
                )
    }));
}
