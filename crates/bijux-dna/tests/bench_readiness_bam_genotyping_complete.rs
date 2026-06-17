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
        .args(["bench", "readiness", "render-bam-genotyping-complete", "--json"])
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
fn bench_readiness_bam_genotyping_complete_reports_governed_row() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_genotyping_complete.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/stages/bam.genotyping.complete.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("toolset_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("bam_pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-genotyping")
    );
    assert_eq!(
        payload.get("cross_pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-genotyping-to-vcf-downstream")
    );
    assert_eq!(
        payload.get("expected_tool_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::json!("angsd")])
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];

    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("bam.genotyping"));
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("angsd"));
    assert_eq!(
        row.get("local_plan_sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_genotyping_candidate_panel")
    );
    assert_eq!(
        row.get("local_plan_reference_fasta").and_then(serde_json::Value::as_str),
        Some(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        )
    );
    assert_eq!(
        row.get("local_plan_sites_vcf").and_then(serde_json::Value::as_str),
        Some(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
        )
    );
    assert_eq!(
        row.get("local_plan_regions").and_then(serde_json::Value::as_str),
        Some(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
        )
    );
    assert_eq!(row.get("local_plan_min_posterior").and_then(serde_json::Value::as_f64), Some(0.9));
    assert_eq!(row.get("local_plan_min_call_rate").and_then(serde_json::Value::as_f64), Some(0.5));
    assert_eq!(
        row.get("parser_smoke_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.genotyping.v1")
    );
    assert_eq!(row.get("parser_smoke_call_rate").and_then(serde_json::Value::as_f64), Some(1.0));
    assert_eq!(
        row.get("parser_smoke_mean_posterior").and_then(serde_json::Value::as_f64),
        Some(0.99)
    );
    assert_eq!(
        row.get("cross_pipeline_vcf_filter_inputs").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::json!("genotyping_vcf_gz"),
            serde_json::json!("genotyping_vcf_tbi"),
            serde_json::json!("genotyping_report_json")
        ])
    );
    assert_eq!(row.get("downstream_variant_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(row.get("downstream_sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        row.get("downstream_sample_missingness_exclusion_threshold")
            .and_then(serde_json::Value::as_f64),
        Some(0.5)
    );
    assert_eq!(
        row.get("downstream_variant_missingness_exclusion_threshold")
            .and_then(serde_json::Value::as_f64),
        Some(0.5)
    );
    assert_eq!(row.get("active_scope_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("command_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("expected_result_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("report_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("schema_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("local_plan_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("producer_contract_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("parser_smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("bam_pipeline_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(row.get("cross_pipeline_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        row.get("downstream_handoff_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        row.get("downstream_variant_metrics_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        row.get("downstream_missingness_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(row.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));
}
