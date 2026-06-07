#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_local_vcf_population_structure_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-population-structure-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root
        .join("runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json");
    let source_stage_path = repo_root.join(
        "runs/bench/local-smoke/vcf.population_structure/plink2/source_population_structure.json",
    );
    let source_pruned_variants_path = repo_root
        .join("runs/bench/local-smoke/vcf.population_structure/plink2/source_pruned_variants.tsv");
    let source_logs_path =
        repo_root.join("runs/bench/local-smoke/vcf.population_structure/plink2/source_logs.txt");
    let source_pca_report_path =
        repo_root.join("runs/bench/local-smoke/vcf.population_structure/plink2/source_pca.json");
    let source_admixture_report_path = repo_root
        .join("runs/bench/local-smoke/vcf.population_structure/plink2/source_admixture.json");
    let stage_result_path =
        repo_root.join("runs/bench/local-smoke/vcf.population_structure/plink2/stage-result.json");

    for path in [
        &report_path,
        &source_stage_path,
        &source_pruned_variants_path,
        &source_logs_path,
        &source_pca_report_path,
        &source_admixture_report_path,
        &stage_result_path,
    ] {
        assert!(path.is_file(), "expected file at {}", path.display());
    }

    let report_raw = std::fs::read_to_string(&report_path).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&report_raw).expect("parse report");
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(
        report
            .get("distance_summary")
            .and_then(|row| row.get("pair_count"))
            .and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        report.get("sample_groups").and_then(serde_json::Value::as_array).map(|rows| rows.len()),
        Some(4)
    );

    let source_stage_raw = std::fs::read_to_string(&source_stage_path).expect("read stage report");
    let source_stage: serde_json::Value =
        serde_json::from_str(&source_stage_raw).expect("parse stage report");
    assert_eq!(
        source_stage.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.population_structure.v1")
    );
    assert_eq!(source_stage.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(
        source_stage.get("sample_ids").and_then(serde_json::Value::as_array).map(|rows| rows.len()),
        Some(4)
    );

    let source_pca_raw = std::fs::read_to_string(&source_pca_report_path).expect("read source pca");
    let source_pca: serde_json::Value = serde_json::from_str(&source_pca_raw).expect("parse pca");
    assert_eq!(
        source_pca.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_pca_smoke.v1")
    );

    let source_admixture_raw =
        std::fs::read_to_string(&source_admixture_report_path).expect("read source admixture");
    let source_admixture: serde_json::Value =
        serde_json::from_str(&source_admixture_raw).expect("parse admixture");
    assert_eq!(
        source_admixture.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_admixture_smoke.v1")
    );

    let pruned_variants =
        std::fs::read_to_string(&source_pruned_variants_path).expect("read pruned");
    assert_eq!(pruned_variants.lines().next(), Some("variant"));

    let manifest_raw = std::fs::read_to_string(&stage_result_path).expect("read stage result");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        manifest.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.population_structure")
    );
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("plink2")
    );
    let outputs = manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs");
    assert_eq!(outputs.len(), 6);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str)
            == Some("population_structure_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some(
                    "runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json",
                )
    }));
}
