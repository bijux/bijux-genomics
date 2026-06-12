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
fn bench_readiness_vcf_pca_ready_reports_complete_active_retained_callers() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-pca-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_pca_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/pca-ready.json")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    assert_eq!(
        payload.get("required_metric_names").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("sample_count".to_string()),
            serde_json::Value::String("variant_count".to_string()),
            serde_json::Value::String("excluded_samples".to_string()),
            serde_json::Value::String("unexpected_samples".to_string()),
            serde_json::Value::String("eigenvalues".to_string()),
        ])
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 2);

    let plink2 = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2"))
        .expect("plink2 PCA row");
    assert_eq!(
        plink2.get("result_id").and_then(serde_json::Value::as_str),
        Some("vcf:vcf_production_regression:vcf.pca:vcf_cohort:plink2")
    );
    assert_eq!(plink2.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.pca"));
    assert_eq!(plink2.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(plink2.get("command_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(plink2.get("output_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(plink2.get("parser_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        plink2.get("expected_result_ready").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(plink2.get("report_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(plink2.get("smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        plink2.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_plink_family_adapter")
    );
    assert_eq!(
        plink2.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("population_structure")
    );
    assert_eq!(
        plink2.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("population_structure_metrics")
    );
    assert_eq!(
        plink2.get("smoke_command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-pca-smoke --tool-id plink2")
    );
    assert_eq!(
        plink2.get("smoke_output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.pca/plink2")
    );
    assert_eq!(
        plink2.get("smoke_input_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.pca/plink2/artifacts/input/pca_input.vcf")
    );
    assert_eq!(
        plink2
            .get("smoke_population_labels_manifest_path")
            .and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.pca/plink2/artifacts/input/population_labels.json")
    );
    assert_eq!(plink2.get("smoke_variant_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(plink2.get("smoke_sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert!(plink2.get("raw_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|value| {
            value.as_str()
                == Some("pca_eigenvec=benchmarks/readiness/adapters/plink2/vcf.pca/pca.eigenvec")
        })
    ));
    assert!(plink2.get("smoke_rows").and_then(serde_json::Value::as_array).is_some_and(|items| {
        items.len() == 4
            && items.iter().all(|item| {
                item.get("population_label")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| !value.is_empty())
                    && item.get("pc1").and_then(serde_json::Value::as_f64).is_some()
                    && item.get("pc2").and_then(serde_json::Value::as_f64).is_some()
            })
    }));

    let eigensoft = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("eigensoft"))
        .expect("eigensoft PCA row");
    assert_eq!(
        eigensoft.get("result_id").and_then(serde_json::Value::as_str),
        Some("vcf:vcf_production_regression:vcf.pca:vcf_cohort:eigensoft")
    );
    assert_eq!(eigensoft.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.pca"));
    assert_eq!(
        eigensoft.get("coverage_status").and_then(serde_json::Value::as_str),
        Some("complete")
    );
    assert_eq!(
        eigensoft.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_eigensoft_adapter")
    );
    assert_eq!(eigensoft.get("smoke_ready").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        eigensoft.get("smoke_command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-pca-smoke --tool-id eigensoft")
    );
    assert_eq!(
        eigensoft.get("smoke_output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.pca/eigensoft")
    );
    assert_eq!(
        eigensoft.get("smoke_input_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.pca/eigensoft/artifacts/input/pca_input.vcf")
    );
    assert_eq!(
        eigensoft
            .get("smoke_population_labels_manifest_path")
            .and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.pca/eigensoft/artifacts/input/population_labels.json")
    );
    assert_eq!(eigensoft.get("smoke_variant_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(eigensoft.get("smoke_sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert!(eigensoft.get("raw_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|value| {
            value.as_str()
                == Some(
                    "smartpca_eigenvec=benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.evec",
                )
        })
    ));
    let execution_mode = eigensoft
        .get("smoke_execution_mode")
        .and_then(serde_json::Value::as_str)
        .expect("eigensoft execution mode");
    assert!(matches!(execution_mode, "real_tool" | "fallback_proxy"));
    assert_eq!(
        eigensoft.get("smoke_tool_ok").and_then(serde_json::Value::as_bool),
        Some(execution_mode == "real_tool")
    );
    assert!(eigensoft
        .get("smoke_eigenvalues")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| items.len() >= 2));
    assert_eq!(
        eigensoft.get("missing_surfaces").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
}
