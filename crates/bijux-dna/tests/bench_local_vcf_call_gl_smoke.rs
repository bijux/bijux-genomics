#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_local_vcf_call_gl_smoke_reports_real_governed_outputs() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-call-gl-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_call_gl_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-call-gl-smoke --tool-id bcftools")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.call_gl"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_validation")
    );
    assert_eq!(
        payload.get("sample_name").and_then(serde_json::Value::as_str),
        Some("core-v1-pass")
    );
    assert_eq!(
        payload.get("input_bam").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam")
    );
    assert_eq!(
        payload.get("reference").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
    );
    assert_eq!(
        payload
            .get("materialized_reference_path")
            .and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_gl/bcftools/artifacts/reference/corpus_01_bam_reference.fasta")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_gl/bcftools")
    );
    assert_eq!(
        payload.get("output_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_gl/bcftools/gl.vcf.gz")
    );
    assert_eq!(
        payload.get("output_tbi_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_gl/bcftools/gl.vcf.gz.tbi")
    );
    assert_eq!(
        payload.get("metrics_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_gl/bcftools/metrics.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_gl/bcftools/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("likelihood_field").and_then(serde_json::Value::as_str), Some("PL"));
    assert_eq!(payload.get("sites_with_likelihoods").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(
        payload.get("samples_with_likelihoods").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("missing_likelihoods").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("parseable").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("gt_present").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(payload.get("gl_present").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload
        .get("validation_checks")
        .and_then(serde_json::Value::as_object)
        .expect("validation_checks object");
    assert_eq!(checks.get("bgzip").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("tabix_index").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("sorted").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("contig_header_sane").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("sample_ids_valid").and_then(serde_json::Value::as_bool), Some(true));

    let repo_root = support::repo_root().expect("repo root");
    let metrics_path = repo_root.join("runs/bench/local-smoke/vcf.call_gl/bcftools/metrics.json");
    let raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_call_gl_smoke.metrics.v1")
    );
    assert_eq!(metrics.get("likelihood_field").and_then(serde_json::Value::as_str), Some("PL"));
    assert_eq!(metrics.get("sites_with_likelihoods").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(
        metrics.get("samples_with_likelihoods").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(metrics.get("missing_likelihoods").and_then(serde_json::Value::as_u64), Some(0));
}
