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
fn bench_local_vcf_impute_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-impute-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/local-smoke/vcf.impute/beagle/imputed.vcf.gz"
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_vcf = repo_root.join("target/local-smoke/vcf.impute/beagle/imputed.vcf.gz");
    let output_tbi = repo_root.join("target/local-smoke/vcf.impute/beagle/imputed.vcf.gz.tbi");
    let panel_assets_path =
        repo_root.join("target/local-smoke/vcf.impute/beagle/panel_assets.json");
    let imputation_qc_path =
        repo_root.join("target/local-smoke/vcf.impute/beagle/imputation_qc.json");
    let imputation_qc_tsv_path =
        repo_root.join("target/local-smoke/vcf.impute/beagle/imputation_qc.tsv");
    let imputation_manifest_path =
        repo_root.join("target/local-smoke/vcf.impute/beagle/imputation_manifest.json");
    let overlap_stats_path =
        repo_root.join("target/local-smoke/vcf.impute/beagle/overlap_stats.json");
    let warnings_path = repo_root.join("target/local-smoke/vcf.impute/beagle/warnings.json");
    let imputation_accept_path =
        repo_root.join("target/local-smoke/vcf.impute/beagle/imputation_accept.json");
    let panel_mismatch_path =
        repo_root.join("target/local-smoke/vcf.impute/beagle/panel_mismatch_diagnostics.json");
    let maf_bins_path = repo_root.join("target/local-smoke/vcf.impute/beagle/maf_bins.tsv");
    let logs_path = repo_root.join("target/local-smoke/vcf.impute/beagle/logs.txt");
    let metrics_path = repo_root.join("target/local-smoke/vcf.impute/beagle/metrics.json");
    let manifest_path = repo_root.join("target/local-smoke/vcf.impute/beagle/stage-result.json");
    let input_vcf =
        repo_root.join("target/local-smoke/vcf.impute/beagle/artifacts/input/impute_input.vcf");
    let truth_vcf =
        repo_root.join("target/local-smoke/vcf.impute/beagle/artifacts/input/impute_truth.vcf");

    assert!(output_vcf.is_file(), "expected output VCF at {}", output_vcf.display());
    assert!(output_tbi.is_file(), "expected output index at {}", output_tbi.display());
    assert!(
        panel_assets_path.is_file(),
        "expected panel assets at {}",
        panel_assets_path.display()
    );
    assert!(
        imputation_qc_path.is_file(),
        "expected imputation qc at {}",
        imputation_qc_path.display()
    );
    assert!(
        imputation_qc_tsv_path.is_file(),
        "expected imputation qc tsv at {}",
        imputation_qc_tsv_path.display()
    );
    assert!(
        imputation_manifest_path.is_file(),
        "expected imputation manifest at {}",
        imputation_manifest_path.display()
    );
    assert!(
        overlap_stats_path.is_file(),
        "expected overlap stats at {}",
        overlap_stats_path.display()
    );
    assert!(warnings_path.is_file(), "expected warnings at {}", warnings_path.display());
    assert!(
        imputation_accept_path.is_file(),
        "expected imputation accept report at {}",
        imputation_accept_path.display()
    );
    assert!(
        panel_mismatch_path.is_file(),
        "expected panel mismatch diagnostics at {}",
        panel_mismatch_path.display()
    );
    assert!(maf_bins_path.is_file(), "expected maf bins at {}", maf_bins_path.display());
    assert!(logs_path.is_file(), "expected logs at {}", logs_path.display());
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(input_vcf.is_file(), "expected synthetic input at {}", input_vcf.display());
    assert!(truth_vcf.is_file(), "expected synthetic truth at {}", truth_vcf.display());

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

    let imputation_qc_raw =
        std::fs::read_to_string(&imputation_qc_path).expect("read imputation qc");
    let imputation_qc: serde_json::Value =
        serde_json::from_str(&imputation_qc_raw).expect("parse imputation qc");
    assert_eq!(imputation_qc.get("backend").and_then(serde_json::Value::as_str), Some("beagle"));
    assert_eq!(
        imputation_qc.pointer("/concordance/truth_provided").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        imputation_qc
            .pointer("/concordance/imputed_match_count")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.impute"));
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("beagle")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-impute-smoke --tool-id beagle")
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 13);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("imputed_vcf")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.impute/beagle/imputed.vcf.gz")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("imputation_qc_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.impute/beagle/imputation_qc.json")
    }));
}
