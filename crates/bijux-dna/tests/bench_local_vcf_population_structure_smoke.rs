#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_local_vcf_population_structure_smoke_reports_consumed_upstream_contract() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-population-structure-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_population_structure_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-population-structure-smoke --tool-id plink2")
    );
    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.population_structure")
    );
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("plink2"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("vcf_mini_multisample_cohort")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2")
    );
    assert_eq!(
        payload.get("population_structure_json_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json")
    );
    assert_eq!(
        payload.get("source_population_structure_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/source_population_structure.json")
    );
    assert_eq!(
        payload.get("source_pruned_variants_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/source_pruned_variants.tsv")
    );
    assert_eq!(
        payload.get("source_logs_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/source_logs.txt")
    );
    assert_eq!(
        payload.get("source_pca_report_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/source_pca.json")
    );
    assert_eq!(
        payload.get("source_admixture_report_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/source_admixture.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/stage-result.json")
    );
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("complete"));

    let consumed_pca = payload.get("consumed_pca").expect("consumed pca");
    assert_eq!(
        consumed_pca.get("report_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/source_pca.json")
    );
    assert_eq!(consumed_pca.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert!(matches!(
        consumed_pca.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("real_tool" | "fallback_proxy")
    ));

    let consumed_admixture = payload.get("consumed_admixture").expect("consumed admixture");
    assert_eq!(
        consumed_admixture.get("report_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.population_structure/plink2/source_admixture.json")
    );
    assert_eq!(consumed_admixture.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(consumed_admixture.get("selected_k").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        consumed_admixture.get("status").and_then(serde_json::Value::as_str),
        Some("complete")
    );

    let sample_groups =
        payload.get("sample_groups").and_then(serde_json::Value::as_array).expect("sample groups");
    assert_eq!(sample_groups.len(), 4);
    assert_eq!(
        sample_groups
            .iter()
            .filter_map(|row| row.get("sample_id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert!(sample_groups
        .iter()
        .all(|row| row.get("pc1").and_then(serde_json::Value::as_f64).is_some()));
    assert!(sample_groups
        .iter()
        .all(|row| row.get("pc2").and_then(serde_json::Value::as_f64).is_some()));
    assert!(sample_groups
        .iter()
        .all(|row| row.get("dominant_cluster").and_then(serde_json::Value::as_str).is_some()));

    let distance_summary = payload.get("distance_summary").expect("distance summary");
    assert_eq!(distance_summary.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(distance_summary.get("pair_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        distance_summary.get("within_population_pair_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        distance_summary.get("cross_population_pair_count").and_then(serde_json::Value::as_u64),
        Some(4)
    );

    let repo_root = support::repo_root().expect("repo root");
    let persisted_path = repo_root
        .join("runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json");
    let persisted_raw = std::fs::read_to_string(&persisted_path).expect("read persisted report");
    let persisted: serde_json::Value = serde_json::from_str(&persisted_raw).expect("parse report");
    assert_eq!(
        persisted.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_population_structure_smoke.v1")
    );
    assert_eq!(
        persisted
            .get("distance_summary")
            .and_then(|row| row.get("pair_count"))
            .and_then(serde_json::Value::as_u64),
        Some(6)
    );
}
