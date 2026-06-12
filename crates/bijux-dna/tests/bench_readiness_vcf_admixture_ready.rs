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
fn bench_readiness_vcf_admixture_ready_reports_complete_active_retained_caller() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-admixture-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_admixture_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf/admixture-ready.json")
    );
    assert_eq!(payload.get("retained_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    assert_eq!(
        payload.get("required_metric_names").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("selected_k".to_string()),
            serde_json::Value::String("sample_count".to_string()),
            serde_json::Value::String("population_count".to_string()),
            serde_json::Value::String("status".to_string()),
        ])
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 1);

    let row = &rows[0];
    assert_eq!(
        row.get("result_id").and_then(serde_json::Value::as_str),
        Some("vcf:vcf_production_regression:vcf.admixture:vcf_cohort:plink2")
    );
    assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.admixture"));
    assert_eq!(row.get("tool_id").and_then(serde_json::Value::as_str), Some("plink2"));
    assert_eq!(row.get("coverage_status").and_then(serde_json::Value::as_str), Some("complete"));
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
        row.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_plink_family_adapter")
    );
    assert_eq!(
        row.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("population_structure")
    );
    assert_eq!(
        row.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("population_structure_metrics")
    );
    assert_eq!(
        row.get("smoke_command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-admixture-smoke --tool-id plink2")
    );
    assert_eq!(
        row.get("smoke_output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.admixture/plink2")
    );
    assert_eq!(
        row.get("smoke_input_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.admixture/plink2/artifacts/input/admixture_input.vcf")
    );
    assert_eq!(
        row.get("smoke_population_labels_manifest_path")
            .and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.admixture/plink2/artifacts/input/population_labels.json")
    );
    assert_eq!(row.get("smoke_selected_k").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(row.get("smoke_sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(row.get("smoke_population_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        row.get("smoke_cluster_headers").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("cluster_1".to_string()),
            serde_json::Value::String("cluster_2".to_string()),
        ])
    );
    assert_eq!(
        row.get("missing_surfaces").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
    assert!(row.get("raw_outputs").and_then(serde_json::Value::as_array).is_some_and(|outputs| {
        outputs.iter().any(|value| {
            value.as_str()
                == Some(
                    "admixture_proxy_eigenvec=benchmarks/readiness/adapters/plink2/vcf.admixture/admixture.eigenvec",
                )
        })
    }));
    assert!(row.get("smoke_rows").and_then(serde_json::Value::as_array).is_some_and(|items| {
        items.len() == 4
            && items.iter().all(|item| {
                item.get("K").and_then(serde_json::Value::as_u64) == Some(2)
                    && item.get("status").and_then(serde_json::Value::as_str) == Some("complete")
                    && item
                        .get("population_label")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| !value.is_empty())
                    && item.get("cluster_1").and_then(serde_json::Value::as_f64).is_some()
                    && item.get("cluster_2").and_then(serde_json::Value::as_f64).is_some()
                    && ((item
                        .get("cluster_1")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or_default()
                        + item
                            .get("cluster_2")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or_default())
                        - 1.0)
                        .abs()
                        <= 1e-6
            })
    }));
}
