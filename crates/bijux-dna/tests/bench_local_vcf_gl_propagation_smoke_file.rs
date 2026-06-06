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
fn bench_local_vcf_gl_propagation_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-gl-propagation-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/local-smoke/vcf.gl_propagation/bcftools/propagated.vcf.gz"
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_vcf =
        repo_root.join("target/local-smoke/vcf.gl_propagation/bcftools/propagated.vcf.gz");
    let output_tbi =
        repo_root.join("target/local-smoke/vcf.gl_propagation/bcftools/propagated.vcf.gz.tbi");
    let output_bcf =
        repo_root.join("target/local-smoke/vcf.gl_propagation/bcftools/propagated.bcf");
    let output_bcf_csi =
        repo_root.join("target/local-smoke/vcf.gl_propagation/bcftools/propagated.bcf.csi");
    let report_path =
        repo_root.join("target/local-smoke/vcf.gl_propagation/bcftools/gl_propagation_report.json");
    let metrics_path =
        repo_root.join("target/local-smoke/vcf.gl_propagation/bcftools/metrics.json");
    let manifest_path =
        repo_root.join("target/local-smoke/vcf.gl_propagation/bcftools/stage-result.json");
    let input_vcf = repo_root
        .join("target/local-smoke/vcf.gl_propagation/bcftools/artifacts/input/gl_input.vcf");

    assert!(output_vcf.is_file(), "expected output VCF at {}", output_vcf.display());
    assert!(output_tbi.is_file(), "expected output index at {}", output_tbi.display());
    assert!(output_bcf.is_file(), "expected output BCF at {}", output_bcf.display());
    assert!(output_bcf_csi.is_file(), "expected output BCF index at {}", output_bcf_csi.display());
    assert!(report_path.is_file(), "expected report at {}", report_path.display());
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(input_vcf.is_file(), "expected synthetic input at {}", input_vcf.display());

    let report_raw = std::fs::read_to_string(&report_path).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&report_raw).expect("parse report");
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.gl_propagation_report.v1")
    );
    assert_eq!(report.get("records_seen").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(report.get("has_gl_or_pl").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("allele_reordered_records").and_then(serde_json::Value::as_u64), Some(0));

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        manifest.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.gl_propagation")
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
        Some("bijux-dna bench local run-vcf-gl-propagation-smoke --tool-id bcftools")
    );
    assert_eq!(
        manifest
            .get("runtime")
            .and_then(|value| value.get("exit_code"))
            .and_then(serde_json::Value::as_i64),
        Some(0)
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 6);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("gl_propagated_vcf")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.gl_propagation/bcftools/propagated.vcf.gz")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("gl_propagation_report")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.gl_propagation/bcftools/gl_propagation_report.json")
    }));
}
