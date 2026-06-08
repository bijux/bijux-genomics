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
fn bench_readiness_vcf_call_diploid_ready_reports_complete_retained_callers() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-call-diploid-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_call_diploid_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/call-diploid-ready.json")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let required_metric_names = payload
        .get("required_metric_names")
        .and_then(serde_json::Value::as_array)
        .expect("required metric names");
    assert_eq!(
        required_metric_names,
        &vec![
            serde_json::Value::String("ploidy".to_string()),
            serde_json::Value::String("called_genotypes".to_string()),
            serde_json::Value::String("heterozygous_count".to_string()),
            serde_json::Value::String("homozygous_ref_count".to_string()),
            serde_json::Value::String("homozygous_alt_count".to_string()),
            serde_json::Value::String("missing_count".to_string()),
            serde_json::Value::String("sample_count".to_string()),
        ]
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);
    let row = rows.first().expect("first row");

    assert_eq!(
        row.get("result_id").and_then(serde_json::Value::as_str),
        Some("vcf:vcf_production_regression:vcf.call_diploid:bam_bundle:bcftools")
    );
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.call_diploid"));
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
            serde_json::Value::String("call_diploid".to_string()),
            serde_json::Value::String("index_diploid_vcf".to_string()),
        ])
    );

    assert_eq!(row.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str),
        Some("parse_bcftools_call_diploid_metrics")
    );
    assert_eq!(
        row.get("parser_fixture_schema_id").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.call_diploid.v1")
    );
    assert_eq!(
        row.get("parser_fixture_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_diploid")
    );

    assert_eq!(row.get("expected_result_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("expected_outputs").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("diploid_vcf".to_string())])
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

    assert_eq!(row.get("smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("smoke_command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-call-diploid-smoke --tool-id bcftools")
    );
    assert_eq!(
        row.get("smoke_output_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz")
    );
    assert_eq!(
        row.get("smoke_output_tbi_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz.tbi")
    );
    assert_eq!(
        row.get("smoke_metrics_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_diploid/bcftools/metrics.json")
    );
    assert_eq!(
        row.get("smoke_stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call_diploid/bcftools/stage-result.json")
    );
    assert_eq!(row.get("smoke_parseable").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("smoke_diploid_compatible").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(row.get("smoke_gt_present").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("smoke_gl_present").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("smoke_ploidy").and_then(serde_json::Value::as_str), Some("diploid"));
    assert_eq!(row.get("smoke_called_genotypes").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(row.get("smoke_heterozygous_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(row.get("smoke_homozygous_ref_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(row.get("smoke_homozygous_alt_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(row.get("smoke_missing_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(row.get("smoke_sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(row.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(
        row.get("missing_surfaces").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
}
