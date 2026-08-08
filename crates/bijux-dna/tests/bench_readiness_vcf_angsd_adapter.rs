#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_vcf_angsd_adapter_reports_governed_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-angsd-adapter", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_angsd_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("angsd"));
    assert_eq!(
        payload.get("tool_status").and_then(serde_json::Value::as_str),
        Some("external_artifact")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/angsd.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("supported_stage_row_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("argv_valid_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(payload.get("bam_list_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(2));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 2);

    let has_stage = |stage_id: &str, stage_status: &str, benchmark_status: &str| {
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("stage_status").and_then(serde_json::Value::as_str) == Some(stage_status)
                && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                    == Some(benchmark_status)
                && row.get("argv_validation_passed").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && row.get("missing_input_test_passed").and_then(serde_json::Value::as_bool)
                    == Some(true)
        })
    };

    assert!(
        has_stage("vcf.call_gl", "supported", "not_benchmark_ready"),
        "report must retain the governed angsd GL calling row"
    );
    assert!(
        has_stage("vcf.call_pseudohaploid", "supported", "not_benchmark_ready"),
        "report must retain the governed angsd pseudohaploid row"
    );
    let call_gl = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call_gl"))
        .expect("call_gl row");
    assert_eq!(
        call_gl.get("bam_list_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/angsd/vcf.call_gl/angsd-inputs.bam.list")
    );
    assert_eq!(
        call_gl.get("likelihood_model").and_then(serde_json::Value::as_str),
        Some("GL2_doGlf2_doMajorMinor1_doMaf1")
    );
    let call_gl_argv = call_gl
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("call_gl argv");
    assert!(
        call_gl_argv.iter().filter_map(serde_json::Value::as_str).any(|part| part == "-GL"),
        "call_gl row must keep ANGSD genotype-likelihood flags"
    );

    assert!(rows.iter().all(|row| {
        matches!(
            row.get("stage_id").and_then(serde_json::Value::as_str),
            Some("vcf.call_gl" | "vcf.call_pseudohaploid")
        )
    }));
}
