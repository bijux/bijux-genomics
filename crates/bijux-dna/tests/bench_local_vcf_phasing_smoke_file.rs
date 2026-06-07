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
fn bench_local_vcf_phasing_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-phasing-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz"
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_vcf = repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz");
    let output_tbi =
        repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz.tbi");
    let panel_assets_path =
        repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/panel_assets.json");
    let phasing_qc_path =
        repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/phasing_qc.json");
    let phasing_manifest_path =
        repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/phasing_manifest.json");
    let phase_block_stats_path =
        repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/phase_block_stats.tsv");
    let switch_error_proxy_path =
        repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/switch_error_proxy.tsv");
    let logs_path = repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/logs.txt");
    let metrics_path = repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/metrics.json");
    let manifest_path =
        repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/stage-result.json");
    let input_vcf = repo_root
        .join("runs/bench/local-smoke/vcf.phasing/shapeit5/artifacts/input/phasing_input.vcf");

    assert!(output_vcf.is_file(), "expected output VCF at {}", output_vcf.display());
    assert!(output_tbi.is_file(), "expected output index at {}", output_tbi.display());
    assert!(
        panel_assets_path.is_file(),
        "expected panel assets at {}",
        panel_assets_path.display()
    );
    assert!(phasing_qc_path.is_file(), "expected phasing qc at {}", phasing_qc_path.display());
    assert!(
        phasing_manifest_path.is_file(),
        "expected phasing manifest at {}",
        phasing_manifest_path.display()
    );
    assert!(
        phase_block_stats_path.is_file(),
        "expected phase block stats at {}",
        phase_block_stats_path.display()
    );
    assert!(
        switch_error_proxy_path.is_file(),
        "expected switch proxy at {}",
        switch_error_proxy_path.display()
    );
    assert!(logs_path.is_file(), "expected logs at {}", logs_path.display());
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(input_vcf.is_file(), "expected synthetic input at {}", input_vcf.display());

    let panel_assets_raw = std::fs::read_to_string(&panel_assets_path).expect("read panel assets");
    let panel_assets: serde_json::Value =
        serde_json::from_str(&panel_assets_raw).expect("parse panel assets");
    assert_eq!(
        panel_assets.get("panel_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_mini")
    );
    assert_eq!(
        panel_assets.get("map_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_chr_map")
    );
    assert!(panel_assets
        .get("materialized_files")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| !rows.is_empty()));

    let phasing_manifest_raw =
        std::fs::read_to_string(&phasing_manifest_path).expect("read phasing manifest");
    let phasing_manifest: serde_json::Value =
        serde_json::from_str(&phasing_manifest_raw).expect("parse phasing manifest");
    assert_eq!(
        phasing_manifest.get("backend").and_then(serde_json::Value::as_str),
        Some("shapeit5")
    );
    assert_eq!(
        phasing_manifest.pointer("/map/map_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_chr_map")
    );

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.phasing"));
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("shapeit5")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-phasing-smoke --tool-id shapeit5")
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 9);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("phased_vcf")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("panel_assets_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.phasing/shapeit5/panel_assets.json")
    }));
}
