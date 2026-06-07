#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
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
fn bench_local_real_smoke_core_subset_json_reports_governed_real_execution_slice() {
    let payload = run_cli_json(&["bench", "local", "run-real-smoke-core-subset", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_real_smoke_core_subset.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-real-smoke/core-subset/REAL_SMOKE_SUMMARY.json")
    );
    assert_eq!(payload.get("execution_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("stage_execution_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("pipeline_bridge_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(2));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 4);

    let execution_ids = rows
        .iter()
        .filter_map(|row| row.get("execution_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        execution_ids,
        BTreeSet::from([
            "bam.validate",
            "bridge:bam-to-vcf.call",
            "fastq.validate_reads",
            "vcf.stats",
        ])
    );

    let bridge = rows
        .iter()
        .find(|row| {
            row.get("execution_id").and_then(serde_json::Value::as_str)
                == Some("bridge:bam-to-vcf.call")
        })
        .expect("bridge row");
    assert_eq!(
        bridge.get("execution_kind").and_then(serde_json::Value::as_str),
        Some("pipeline_bridge")
    );
    assert_eq!(bridge.get("bridge_source_domain").and_then(serde_json::Value::as_str), Some("bam"));
    assert_eq!(bridge.get("bridge_target_domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(bridge.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.call"));
    assert_eq!(
        bridge.get("manifest_status").and_then(serde_json::Value::as_str),
        Some("succeeded")
    );
    assert_eq!(bridge.get("manifest_exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert!(bridge.get("normalized_metrics").and_then(serde_json::Value::as_object).is_some_and(
        |metrics| metrics.contains_key("variant_count") && metrics.contains_key("sample_count")
    ));

    let vcf_stats = rows
        .iter()
        .find(|row| {
            row.get("execution_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
        })
        .expect("vcf.stats row");
    assert_eq!(vcf_stats.get("execution_kind").and_then(serde_json::Value::as_str), Some("stage"));
    assert!(vcf_stats
        .get("normalized_metrics")
        .and_then(serde_json::Value::as_object)
        .is_some_and(
            |metrics| metrics.contains_key("ti_tv") && metrics.contains_key("transition_count")
        ));

    let fastq = rows
        .iter()
        .find(|row| {
            row.get("execution_id").and_then(serde_json::Value::as_str)
                == Some("fastq.validate_reads")
        })
        .expect("fastq row");
    assert!(fastq
        .get("normalized_metrics")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|metrics| metrics.contains_key("case_count")
            && metrics.contains_key("all_cases_passed")));
}
