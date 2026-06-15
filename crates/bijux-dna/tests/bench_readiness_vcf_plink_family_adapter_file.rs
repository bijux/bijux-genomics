#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_file_command(command_name: &str) -> serde_json::Value {
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
        .args(["bench", "readiness", command_name])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join(String::from_utf8_lossy(&output.stdout).trim());
    assert!(report_path.is_file(), "adapter JSON must exist: {}", report_path.display());

    serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(&report_path).expect("read adapter JSON"),
    )
    .expect("parse adapter JSON")
}

#[test]
fn bench_readiness_vcf_plink_adapter_writes_governed_json_file() {
    let payload = run_file_command("render-vcf-plink-adapter");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_plink_adapter.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(2));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    for stage_id in ["vcf.qc", "vcf.admixture"] {
        assert!(
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
            }),
            "governed PLINK adapter JSON must retain stage row: {stage_id}"
        );
    }

    let admixture_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
        })
        .expect("plink admixture row");
    assert!(
        admixture_row.get("declared_outputs").and_then(serde_json::Value::as_array).is_some_and(
            |items| {
                items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("bed_matrix")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("admixture_report")
                })
            }
        ),
        "plink admixture row must retain cohort-preparation outputs and normalized report mapping"
    );
}

#[test]
fn bench_readiness_vcf_plink2_adapter_writes_governed_json_file() {
    let payload = run_file_command("render-vcf-plink2-adapter");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_plink2_adapter.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(5));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    for stage_id in ["vcf.qc", "vcf.pca", "vcf.admixture", "vcf.population_structure", "vcf.roh"] {
        assert!(
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
            }),
            "governed PLINK2 adapter JSON must retain stage row: {stage_id}"
        );
    }

    let pca_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca"))
        .expect("plink2 pca row");
    assert_eq!(
        pca_row.get("parser_output_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("pca_report".to_string())])
    );

    let roh_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh"))
        .expect("plink2 roh row");
    assert!(
        roh_row.get("declared_outputs").and_then(serde_json::Value::as_array).is_some_and(
            |items| {
                items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str) == Some("roh_hom")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("roh_report")
                })
            }
        ),
        "plink2 roh row must retain HOM output and normalized report declarations"
    );
}
