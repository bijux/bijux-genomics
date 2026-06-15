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
fn bench_readiness_all_domain_active_scope_blockers_reports_exact_removed_bindings() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-active-scope-blockers", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_active_scope_blockers.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/active-scope-blockers.tsv")
    );
    assert_eq!(
        payload.get("removed_from_scope_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/removed-from-scope.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let blocker_type_counts = payload
        .get("blocker_type_counts")
        .and_then(serde_json::Value::as_object)
        .expect("blocker type counts");
    assert_eq!(
        blocker_type_counts.get("benchmark_not_ready").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        blocker_type_counts.get("lifecycle_not_active").and_then(serde_json::Value::as_u64),
        Some(7)
    );
    assert!(
        blocker_type_counts.get("non_executable_adapter").is_none(),
        "current active-scope blockers should be fully explained by lifecycle and benchmark readiness exits"
    );

    let blocker_path_counts = payload
        .get("blocker_path_counts")
        .and_then(serde_json::Value::as_object)
        .expect("blocker path counts");
    assert_eq!(
        blocker_path_counts
            .get("benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json")
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        blocker_path_counts
            .get("benchmarks/readiness/all-domains/no-planned-rows.json")
            .and_then(serde_json::Value::as_u64),
        Some(7)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 10);
    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty(), "active-scope blocker table must fail closed on drift");

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.index_reference")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
            && row.get("corpus_id").and_then(serde_json::Value::as_str) == Some("not_assigned")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("reference_fasta+reference_index_output")
            && row.get("blocker_type").and_then(serde_json::Value::as_str)
                == Some("benchmark_not_ready")
            && row.get("blocker_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json")
    }));
    assert!(
        rows.iter().all(|row| {
            !(row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.estimate_library_complexity_prealign")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna"))
        }),
        "supported complexity rows must stay out of active-scope blockers"
    );
    assert!(
        rows.iter().all(|row| {
            !(row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2"))
        }),
        "vcf.population_structure/plink2 must stay out of active-scope blockers once it is benchmark ready"
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
        "every blocker row must stay absent from governed active downstream surfaces"
    );
}
