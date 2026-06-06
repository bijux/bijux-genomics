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
fn bench_local_vcf_stats_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-stats-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/local-smoke/vcf.stats/bcftools/stats.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let stats_json_path = repo_root.join("target/local-smoke/vcf.stats/bcftools/stats.json");
    let bcftools_stats_path =
        repo_root.join("target/local-smoke/vcf.stats/bcftools/bcftools_stats.txt");
    let metrics_path = repo_root.join("target/local-smoke/vcf.stats/bcftools/metrics.json");
    let manifest_path = repo_root.join("target/local-smoke/vcf.stats/bcftools/stage-result.json");
    let input_vcf_path =
        repo_root.join("target/local-smoke/vcf.stats/bcftools/artifacts/input/stats_input.vcf");

    assert!(stats_json_path.is_file(), "expected stats json at {}", stats_json_path.display());
    assert!(
        bcftools_stats_path.is_file(),
        "expected bcftools stats at {}",
        bcftools_stats_path.display()
    );
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(input_vcf_path.is_file(), "expected input VCF at {}", input_vcf_path.display());

    let metrics_raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&metrics_raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_stats_smoke.metrics.v1")
    );
    assert_eq!(metrics.get("variant_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(metrics.get("snp_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(metrics.get("indel_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("transition_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(metrics.get("transversion_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("ti_tv").and_then(serde_json::Value::as_f64), Some(2.0));
    assert_eq!(metrics.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.stats"));
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("bcftools")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-stats-smoke --tool-id bcftools")
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 3);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("stats_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.stats/bcftools/stats.json")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("bcftools_stats_txt")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.stats/bcftools/bcftools_stats.txt")
    }));
}
