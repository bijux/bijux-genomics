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
fn bench_readiness_vcf_qc_ready_reports_complete_retained_qc_callers() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-qc-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_qc_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/qc-ready.json")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    assert_eq!(
        payload.get("required_metric_names").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("sample_missingness".to_string()),
            serde_json::Value::String("variant_missingness".to_string()),
            serde_json::Value::String("maf_summary".to_string()),
            serde_json::Value::String("heterozygosity".to_string()),
            serde_json::Value::String("hwe_summary".to_string()),
            serde_json::Value::String("excluded_samples".to_string()),
            serde_json::Value::String("excluded_variants".to_string()),
            serde_json::Value::String("sample_missingness_exclusion_threshold".to_string()),
            serde_json::Value::String("variant_missingness_exclusion_threshold".to_string()),
        ])
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 3);

    for tool_id in ["bcftools", "plink", "plink2"] {
        let row = rows
            .iter()
            .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id))
            .unwrap_or_else(|| panic!("missing {tool_id} qc row"));
        let expected_output_root = format!("runs/bench/local-smoke/vcf.qc/{tool_id}");
        let expected_qc_json_path = format!("runs/bench/local-smoke/vcf.qc/{tool_id}/qc.json");
        let expected_qc_summary_path =
            format!("runs/bench/local-smoke/vcf.qc/{tool_id}/qc_summary.json");
        let expected_metrics_path = format!("runs/bench/local-smoke/vcf.qc/{tool_id}/metrics.json");
        let expected_stage_result_path =
            format!("runs/bench/local-smoke/vcf.qc/{tool_id}/stage-result.json");

        assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.qc"));
        assert_eq!(
            row.get("retained_scope_state").and_then(serde_json::Value::as_str),
            Some("active")
        );
        assert_eq!(
            row.get("all_domain_active_row_present").and_then(serde_json::Value::as_bool),
            Some(true)
        );

        assert_eq!(row.get("command_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(row.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(row.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            row.get("expected_result_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(row.get("report_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(row.get("smoke_ready").and_then(serde_json::Value::as_bool), Some(true));

        assert_eq!(
            row.get("parser_fixture_schema_id").and_then(serde_json::Value::as_str),
            Some("bijux.vcf.qc.v1")
        );
        assert_eq!(
            row.get("report_section_id").and_then(serde_json::Value::as_str),
            Some("quality_control")
        );
        assert_eq!(
            row.get("summary_table_id").and_then(serde_json::Value::as_str),
            Some("quality_control_metrics")
        );
        assert_eq!(
            row.get("smoke_output_root").and_then(serde_json::Value::as_str),
            Some(expected_output_root.as_str())
        );
        assert_eq!(
            row.get("smoke_qc_json_path").and_then(serde_json::Value::as_str),
            Some(expected_qc_json_path.as_str())
        );
        assert_eq!(
            row.get("smoke_qc_summary_path").and_then(serde_json::Value::as_str),
            Some(expected_qc_summary_path.as_str())
        );
        assert_eq!(
            row.get("smoke_metrics_path").and_then(serde_json::Value::as_str),
            Some(expected_metrics_path.as_str())
        );
        assert_eq!(
            row.get("smoke_stage_result_manifest_path").and_then(serde_json::Value::as_str),
            Some(expected_stage_result_path.as_str())
        );
        assert_eq!(
            row.get("coverage_status").and_then(serde_json::Value::as_str),
            Some("complete")
        );
        assert_eq!(
            row.get("missing_surfaces").and_then(serde_json::Value::as_array).map(Vec::len),
            Some(0)
        );

        assert_eq!(
            row.get("smoke_sample_missingness_exclusion_threshold")
                .and_then(serde_json::Value::as_f64),
            Some(0.5)
        );
        assert_eq!(
            row.get("smoke_variant_missingness_exclusion_threshold")
                .and_then(serde_json::Value::as_f64),
            Some(0.5)
        );
        assert_eq!(
            row.get("smoke_maf_summary")
                .and_then(serde_json::Value::as_object)
                .and_then(|summary| summary.get("observed_variant_count"))
                .and_then(serde_json::Value::as_u64),
            Some(4)
        );
        assert_eq!(
            row.get("smoke_heterozygosity")
                .and_then(serde_json::Value::as_object)
                .and_then(|summary| summary.get("heterozygous_call_count"))
                .and_then(serde_json::Value::as_u64),
            Some(4)
        );
        assert_eq!(
            row.get("smoke_heterozygosity")
                .and_then(serde_json::Value::as_object)
                .and_then(|summary| summary.get("homozygous_alt_call_count"))
                .and_then(serde_json::Value::as_u64),
            Some(2)
        );
        assert_eq!(
            row.get("smoke_heterozygosity")
                .and_then(serde_json::Value::as_object)
                .and_then(|summary| summary.get("het_hom_ratio"))
                .and_then(serde_json::Value::as_f64),
            Some(2.0)
        );
        assert_eq!(
            row.get("smoke_hwe_summary")
                .and_then(serde_json::Value::as_object)
                .and_then(|summary| summary.get("tested_variant_count"))
                .and_then(serde_json::Value::as_u64),
            Some(3)
        );
        assert_eq!(
            row.get("smoke_hwe_summary")
                .and_then(serde_json::Value::as_object)
                .and_then(|summary| summary.get("pvalue_mean"))
                .and_then(serde_json::Value::as_f64),
            Some(0.825656)
        );
        assert_eq!(
            row.get("smoke_hwe_summary")
                .and_then(serde_json::Value::as_object)
                .and_then(|summary| summary.get("status"))
                .and_then(serde_json::Value::as_str),
            Some("computed_modern")
        );

        let excluded_samples = row
            .get("smoke_excluded_samples")
            .and_then(serde_json::Value::as_array)
            .expect("smoke excluded samples");
        assert_eq!(excluded_samples.len(), 1);
        assert_eq!(
            excluded_samples[0].get("sample_id").and_then(serde_json::Value::as_str),
            Some("qc_sparse")
        );
        assert_eq!(
            excluded_samples[0].get("missingness").and_then(serde_json::Value::as_f64),
            Some(0.75)
        );

        let excluded_variants = row
            .get("smoke_excluded_variants")
            .and_then(serde_json::Value::as_array)
            .expect("smoke excluded variants");
        assert_eq!(excluded_variants.len(), 1);
        assert_eq!(
            excluded_variants[0].get("variant_id").and_then(serde_json::Value::as_str),
            Some("chr1:30:G:A")
        );
        assert_eq!(
            excluded_variants[0].get("missingness").and_then(serde_json::Value::as_f64),
            Some(2.0 / 3.0)
        );
    }
}
