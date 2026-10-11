#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

fn run_cli_report(args: &[&str], report_path: &str) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_guard =
        support::RepoProcessLock::acquire("benchmark-readiness-mutators").expect("repo lock");
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

    assert!(!output.status.success(), "unadmitted ANGSD evidence must keep the gate open");
    serde_json::from_slice(&std::fs::read(repo_root.join(report_path)).expect("read report"))
        .expect("parse report as json")
}

#[test]
fn bench_readiness_vcf_call_gl_ready_reports_unadmitted_angsd_evidence() {
    let payload = run_cli_report(
        &["bench", "readiness", "render-vcf-call-gl-ready", "--json"],
        "artifacts/benchmark-smoke/readiness/vcf/call-gl-ready.json",
    );

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_call_gl_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/vcf/call-gl-ready.json")
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
            serde_json::Value::String("likelihood_field".to_string()),
            serde_json::Value::String("sites_with_likelihoods".to_string()),
            serde_json::Value::String("samples_with_likelihoods".to_string()),
            serde_json::Value::String("missing_likelihoods".to_string()),
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
        Some("vcf:vcf_production_regression:vcf.call_gl:bam_bundle:bcftools")
    );
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.call_gl"));
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
            serde_json::Value::String("call_gl".to_string()),
            serde_json::Value::String("index_gl_sites_vcf".to_string()),
        ])
    );

    assert_eq!(row.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str),
        Some("parse_bcftools_call_gl_metrics")
    );
    assert_eq!(
        row.get("parser_fixture_schema_id").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.call_gl.v1")
    );
    assert_eq!(
        row.get("parser_fixture_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_gl")
    );

    assert_eq!(row.get("expected_result_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("expected_outputs").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("gl_sites_vcf".to_string())])
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
            Some("bijux-dna bench local run-vcf-call-gl-smoke --tool-id bcftools")
        );
        assert_eq!(
            row.get("smoke_output_vcf_path").and_then(serde_json::Value::as_str),
            Some("artifacts/runs/bench/local-smoke/vcf.call_gl/bcftools/gl.vcf.gz")
        );
        assert_eq!(row.get("smoke_parseable").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(row.get("smoke_gt_present").and_then(serde_json::Value::as_bool), Some(false));
        assert_eq!(row.get("smoke_gl_present").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            row.get("smoke_likelihood_field").and_then(serde_json::Value::as_str),
            Some("PL")
        );
        assert_eq!(
            row.get("smoke_sites_with_likelihoods").and_then(serde_json::Value::as_u64),
            Some(12)
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
            Some(&vec![serde_json::Value::String("local_vcf_call_gl_smoke".to_string())])
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
        |values| values.iter().any(|value| { value.as_str() == Some("local_vcf_call_gl_smoke") })
    ));
}
