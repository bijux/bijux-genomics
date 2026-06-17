#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
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
        .args(["bench", "readiness", "render-bam-kinship-complete", "--json"])
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
fn bench_readiness_bam_kinship_complete_reports_governed_rows() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_kinship_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/stages/bam.kinship.complete.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(payload.get("local_smoke_case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("bam_pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-kinship")
    );
    assert_eq!(
        payload.get("vcf_pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-genotyping-to-vcf-downstream")
    );
    assert_eq!(
        payload.get("expected_tool_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::json!("angsd"), serde_json::json!("king")])
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 2);

    for row in rows {
        assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.kinship"));
        assert_eq!(row.get("ready_case_status").and_then(serde_json::Value::as_str), Some("ok"));
        assert_eq!(
            row.get("insufficient_case_status").and_then(serde_json::Value::as_str),
            Some("insufficient")
        );
        assert_eq!(
            row.get("insufficient_case_insufficiency_reason").and_then(serde_json::Value::as_str),
            Some("insufficient_overlap_snps")
        );
        assert_eq!(
            row.get("parser_smoke_schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.kinship_summary.v1")
        );
        assert_eq!(
            row.get("parser_smoke_method").and_then(serde_json::Value::as_str),
            Some("king")
        );
        assert_eq!(row.get("parser_smoke_status").and_then(serde_json::Value::as_str), Some("ok"));
        assert_eq!(row.get("parser_smoke_pair_count").and_then(serde_json::Value::as_u64), Some(1));
        assert_eq!(
            row.get("parser_smoke_observed_max_overlap_snps").and_then(serde_json::Value::as_u64),
            Some(6)
        );
        assert_eq!(
            row.get("ready_case_pair_sample_a").and_then(serde_json::Value::as_str),
            Some("sample_a")
        );
        assert_eq!(
            row.get("ready_case_pair_sample_b").and_then(serde_json::Value::as_str),
            Some("sample_b")
        );
        assert_eq!(
            row.get("ready_case_pair_overlap_snps").and_then(serde_json::Value::as_u64),
            Some(6)
        );
        assert_eq!(
            row.get("ready_case_kinship_coefficient").and_then(serde_json::Value::as_f64),
            Some(0.416667)
        );
        assert_eq!(
            row.get("ready_case_relationship_label").and_then(serde_json::Value::as_str),
            Some("first_degree")
        );
        assert_eq!(
            row.get("bam_pipeline_upstream_inputs").and_then(serde_json::Value::as_array),
            Some(&vec![
                serde_json::json!("overlap_corrected_bam"),
                serde_json::json!("overlap_corrected_bai"),
                serde_json::json!("overlap_correction_summary_json"),
                serde_json::json!("genotyping_report_json")
            ])
        );
        assert_eq!(
            row.get("bam_pipeline_external_inputs").and_then(serde_json::Value::as_array),
            Some(&vec![
                serde_json::json!("kinship_reference_panel_contract"),
                serde_json::json!("kinship_population_scope_contract"),
                serde_json::json!("kinship_pairing_contract")
            ])
        );
        assert_eq!(
            row.get("bam_pipeline_outputs").and_then(serde_json::Value::as_array),
            Some(&vec![
                serde_json::json!("kinship_report_json"),
                serde_json::json!("kinship_segments_tsv"),
                serde_json::json!("kinship_summary_json"),
                serde_json::json!("kinship_stage_metrics")
            ])
        );
        assert_eq!(row.get("local_smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            row.get("ready_case_report_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            row.get("ready_case_summary_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            row.get("ready_case_pairwise_table_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            row.get("ready_case_stage_metrics_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            row.get("insufficient_case_report_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            row.get("insufficient_case_summary_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            row.get("insufficient_case_stage_metrics_ready").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(row.get("parser_smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(row.get("bam_pipeline_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(row.get("bam_locality_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(row.get("vcf_isolation_ready").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            row.get("coverage_status").and_then(serde_json::Value::as_str),
            Some("complete")
        );
    }
}
