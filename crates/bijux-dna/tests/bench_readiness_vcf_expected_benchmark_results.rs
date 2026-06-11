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
fn bench_readiness_vcf_expected_benchmark_results_tracks_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-expected-benchmark-results", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_expected_benchmark_results.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-expected-benchmark-results.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(18));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("corpus_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("asset_profile_count").and_then(serde_json::Value::as_u64), Some(5));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 18);
    assert!(rows.iter().all(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("corpus_id").and_then(serde_json::Value::as_str)
                == Some("vcf_production_regression")
            && row
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| !items.is_empty())
            && row
                .get("expected_metrics")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| !items.is_empty())
            && row
                .get("report_section")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty())
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str) == Some("vcf_cohort")
            && row
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("qc_report")))
            && row
                .get("expected_metrics")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("hwe_summary")))
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("quality_control")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str) == Some("bam_bundle")
            && row
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("called_vcf")))
            && row.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("variant_count")),
            )
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("variant_calling")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("vcf.imputation_metrics")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_cohort_with_panel")
            && row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("imputation_metrics_json")),
            )
            && row.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("mean_info_score")),
            )
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("imputation")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_cohort_with_panel")
            && row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("imputed_vcf")),
            )
            && row.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
                |items| {
                    items.iter()
                        .any(|item| item.as_str() == Some("masked_truth_match_count"))
                },
            )
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("imputation")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_cohort")
            && row
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("pca_report")))
            && row
                .get("expected_metrics")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("eigenvalues")))
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("population_structure")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eigensoft")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_cohort")
            && row
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("pca_report")))
            && row
                .get("expected_metrics")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("eigenvalues")))
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("population_structure")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.gl_propagation")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_single_sample")
            && row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("gl_propagated_vcf")),
            )
            && row
                .get("expected_metrics")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("lost_fields")))
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("likelihood_postprocess")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_single_sample")
            && row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("postprocess_vcf")),
            )
            && row
                .get("expected_metrics")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("readable_vcf")))
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("normalization")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("vcf.prepare_reference_panel")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str)
                == Some("vcf_reference_panel")
            && row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("prepared_panel")),
            )
            && row.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|item| item.as_str() == Some("normalization_status")),
            )
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("reference_panel_preparation")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
            && row.get("asset_profile_id").and_then(serde_json::Value::as_str) == Some("vcf_cohort")
            && row
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("stats_json")))
            && row
                .get("expected_metrics")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("ti_tv")))
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("quality_control")
    }));
}
