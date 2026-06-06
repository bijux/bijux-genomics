#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_eigensoft_adapter_writes_governed_json_file() {
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
        .args(["bench", "readiness", "render-vcf-eigensoft-adapter"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = repo_root.join("target/bench-readiness/adapters/eigensoft.vcf.json");
    assert!(report_path.is_file(), "VCF eigensoft adapter JSON must exist");

    let payload = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(&report_path).expect("read VCF eigensoft adapter JSON"),
    )
    .expect("parse VCF eigensoft adapter JSON");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_eigensoft_adapter.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(2));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    for stage_id in ["vcf.pca", "vcf.population_structure"] {
        assert!(
            rows.iter().any(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
            }),
            "governed adapter JSON must retain stage row: {stage_id}"
        );
    }

    let pca_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca"))
        .expect("eigensoft pca row");
    assert!(
        pca_row.get("declared_outputs").and_then(serde_json::Value::as_array).is_some_and(
            |items| {
                items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("eigensoft_geno")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("eigensoft_snp")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("eigensoft_ind")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("smartpca_eigenvec")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("smartpca_eigenval")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("pca_report")
                })
            }
        ),
        "pca row must retain convertf outputs, smartpca outputs, and normalized pca report"
    );

    let population_structure_row = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
        })
        .expect("eigensoft population structure row");
    assert!(
        population_structure_row
            .get("declared_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| item.get("artifact_id").and_then(serde_json::Value::as_str) == Some("population_structure_report"))
                    && items.iter().any(|item| {
                        item.get("path").and_then(serde_json::Value::as_str)
                            == Some("target/bench-readiness/adapters/eigensoft/vcf.population_structure/population_structure_report.evec")
                    })
                    && items.iter().any(|item| {
                        item.get("path").and_then(serde_json::Value::as_str)
                            == Some("target/bench-readiness/adapters/eigensoft/vcf.population_structure/population_structure_report.eval")
                    })
            }),
        "population-structure row must retain smartpca outputs and normalized population-structure report"
    );
}
