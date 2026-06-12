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
fn bench_readiness_removed_from_scope_reports_only_non_active_bindings() {
    let payload = run_cli_json(&["bench", "readiness", "render-removed-from-scope", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.removed_from_scope.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/removed-from-scope.tsv")
    );
    assert_eq!(
        payload.get("full_benchmark_report_check_source").and_then(serde_json::Value::as_str),
        Some("tracked_report_json")
    );
    assert_eq!(payload.get("candidate_row_count").and_then(serde_json::Value::as_u64), Some(146));
    assert_eq!(payload.get("candidate_stage_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(payload.get("candidate_tool_count").and_then(serde_json::Value::as_u64), Some(75));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(131));
    assert_eq!(payload.get("active_stage_count").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(payload.get("active_tool_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("removed_row_count").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(payload.get("removed_stage_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("removed_tool_count").and_then(serde_json::Value::as_u64), Some(14));
    assert_eq!(
        payload.get("fully_removed_stage_count").and_then(serde_json::Value::as_u64),
        Some(9)
    );
    assert_eq!(
        payload.get("fully_removed_tool_count").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let scope_exit_kind_counts = payload
        .get("scope_exit_kind_counts")
        .and_then(serde_json::Value::as_object)
        .expect("scope exit kind counts");
    assert_eq!(
        scope_exit_kind_counts.get("benchmark_not_ready").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        scope_exit_kind_counts.get("lifecycle_not_active").and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert!(
        scope_exit_kind_counts.get("non_executable_adapter").is_none(),
        "current removed scope should be fully explained by lifecycle and benchmark readiness exits"
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 16);
    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty(), "removed rows must stay outside governed active surfaces");

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.index_reference")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("not_benchmark_ready")
            && row.get("adapter_status").and_then(serde_json::Value::as_str) == Some("runnable")
            && row.get("scope_exit_kind").and_then(serde_json::Value::as_str)
                == Some("benchmark_not_ready")
            && row.get("stage_removed_from_active_scope").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("tool_removed_from_active_scope").and_then(serde_json::Value::as_bool)
                == Some(true)
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("status").and_then(serde_json::Value::as_str) == Some("planned")
            && row.get("adapter_status").and_then(serde_json::Value::as_str)
                == Some("declared_only")
            && row.get("scope_exit_kind").and_then(serde_json::Value::as_str)
                == Some("lifecycle_not_active")
            && row.get("stage_removed_from_active_scope").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("tool_removed_from_active_scope").and_then(serde_json::Value::as_bool)
                == Some(false)
    }));
    assert!(
        rows.iter().all(|row| {
            !(row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation_metrics")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle"))
        }),
        "active imputation metrics rows must stay out of removed-from-scope output"
    );
    assert!(
        rows.iter().all(|row| {
            row.get("absent_from_active_matrix").and_then(serde_json::Value::as_bool) == Some(true)
                && row.get("absent_from_rendered_commands").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && row.get("absent_from_expected_results").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && row.get("absent_from_full_benchmark_report").and_then(serde_json::Value::as_bool)
                    == Some(true)
        }),
        "removed rows must stay absent from every governed active downstream surface"
    );
}
