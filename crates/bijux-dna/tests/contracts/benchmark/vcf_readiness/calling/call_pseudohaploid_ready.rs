#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

fn run_cli_report(args: &[&str], report_path: &str) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");

    assert!(!output.status.success(), "unadmitted ANGSD evidence must keep the gate open");
    serde_json::from_slice(&std::fs::read(repo_root.join(report_path)).expect("read report"))
        .expect("parse report as json")
}

#[test]
fn bench_readiness_vcf_call_pseudohaploid_ready_reports_unadmitted_angsd_evidence() {
    let payload = run_cli_report(
        &["bench", "readiness", "render-vcf-call-pseudohaploid-ready", "--json"],
        "artifacts/benchmark-smoke/readiness/vcf/call-pseudohaploid-ready.json",
    );

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_call_pseudohaploid_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/vcf/call-pseudohaploid-ready.json")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(2));
    let complete_row_count = payload
        .get("complete_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("complete row count");
    let incomplete_row_count = payload
        .get("incomplete_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("incomplete row count");
    assert_eq!(complete_row_count + incomplete_row_count, 2);
    assert!(incomplete_row_count >= 1);
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        payload.get("violation_count").and_then(serde_json::Value::as_u64),
        Some(incomplete_row_count)
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));

    let required_metric_names = payload
        .get("required_metric_names")
        .and_then(serde_json::Value::as_array)
        .expect("required metric names");
    assert_eq!(
        required_metric_names,
        &vec![
            serde_json::Value::String("target_sites".to_string()),
            serde_json::Value::String("covered_sites".to_string()),
            serde_json::Value::String("called_sites".to_string()),
            serde_json::Value::String("missing_sites".to_string()),
            serde_json::Value::String("sampling_policy".to_string()),
            serde_json::Value::String("random_seed".to_string()),
            serde_json::Value::String("sample_count".to_string()),
        ]
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 2);
    let row = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools"))
        .expect("bcftools row");

    assert_eq!(
        row.get("result_id").and_then(serde_json::Value::as_str),
        Some("vcf:vcf_production_regression:vcf.call_pseudohaploid:bam_bundle:bcftools")
    );
    assert_eq!(
        row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.call_pseudohaploid")
    );
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(row.get("tool_status").and_then(serde_json::Value::as_str), Some("production"));
    assert_eq!(row.get("retained_scope_state").and_then(serde_json::Value::as_str), Some("active"));
    assert_eq!(
        row.get("all_domain_active_row_present").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    assert_eq!(row.get("command_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_bcftools_adapter")
    );
    assert_eq!(row.get("command_step_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        row.get("command_step_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("mpileup".to_string()),
            serde_json::Value::String("call_pseudohaploid".to_string()),
            serde_json::Value::String("index_pseudohaploid_vcf".to_string()),
        ])
    );

    assert_eq!(row.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str),
        Some("parse_bcftools_call_pseudohaploid_metrics")
    );
    assert_eq!(
        row.get("parser_fixture_schema_id").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.call_pseudohaploid.v1")
    );
    assert_eq!(
        row.get("parser_fixture_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_pseudohaploid")
    );

    assert_eq!(row.get("expected_result_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("expected_outputs").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("pseudohaploid_vcf".to_string())])
    );
    assert_eq!(
        row.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("variant_calling")
    );
    assert_eq!(row.get("report_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("variant_calling_metrics")
    );

    if row.get("smoke_ready").and_then(serde_json::Value::as_bool) == Some(true) {
        assert_eq!(
            row.get("smoke_command").and_then(serde_json::Value::as_str),
            Some("bijux-dna bench local run-vcf-call-pseudohaploid-smoke --tool-id bcftools")
        );
        assert_eq!(row.get("smoke_parseable").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            row.get("smoke_haploid_compatible").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(row.get("smoke_target_sites").and_then(serde_json::Value::as_u64), Some(3));
        assert_eq!(row.get("smoke_called_sites").and_then(serde_json::Value::as_u64), Some(3));
        assert_eq!(
            row.get("smoke_sampling_policy").and_then(serde_json::Value::as_str),
            Some("deterministic_first_allele_projection")
        );
        assert_eq!(
            row.get("smoke_deterministic_replay_match").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            row.get("coverage_status").and_then(serde_json::Value::as_str),
            Some("complete")
        );
        assert_eq!(
            row.get("missing_surfaces").and_then(serde_json::Value::as_array).map(Vec::len),
            Some(0)
        );
    } else {
        assert_eq!(
            row.get("coverage_status").and_then(serde_json::Value::as_str),
            Some("incomplete")
        );
        assert_eq!(
            row.get("missing_surfaces").and_then(serde_json::Value::as_array),
            Some(&vec![serde_json::Value::String(
                "local_vcf_call_pseudohaploid_smoke".to_string()
            )])
        );
    }
    let angsd = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd"))
        .expect("ANGSD row");
    assert_eq!(angsd.get("tool_status").and_then(serde_json::Value::as_str), Some("supported"));
    assert_eq!(angsd.get("command_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(angsd.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(angsd.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(angsd.get("expected_result_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(angsd.get("report_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(angsd.get("smoke_ready").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(
        angsd.get("coverage_status").and_then(serde_json::Value::as_str),
        Some("incomplete")
    );
    assert!(angsd.get("missing_surfaces").and_then(serde_json::Value::as_array).is_some_and(
        |values| values
            .iter()
            .any(|value| { value.as_str() == Some("local_vcf_call_pseudohaploid_smoke") })
    ));
}
