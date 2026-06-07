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
fn bench_local_vcf_admixture_smoke_reports_complete_cohort_contract() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-admixture-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_admixture_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-admixture-smoke --tool-id plink2")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.admixture"));
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
        payload.get("fixture_manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/vcf-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("input_vcf_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/artifacts/input/admixture_input.vcf")
    );
    assert_eq!(
        payload.get("sample_metadata_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/artifacts/input/sample_metadata.tsv")
    );
    assert_eq!(
        payload.get("population_metadata_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/artifacts/input/population_metadata.tsv")
    );
    assert_eq!(
        payload.get("population_labels_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/artifacts/input/population_labels.json")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2")
    );
    assert_eq!(
        payload.get("admixture_tsv_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/admixture.tsv")
    );
    assert_eq!(
        payload.get("admixture_json_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/admixture.json")
    );
    assert_eq!(
        payload.get("source_q_matrix_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/source_admixture_q_matrix.tsv")
    );
    assert_eq!(
        payload.get("source_k_selection_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/source_admixture_k_selection.json")
    );
    assert_eq!(
        payload.get("source_logs_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/source_logs.txt")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.admixture/plink2/stage-result.json")
    );
    assert_eq!(payload.get("selected_k").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("population_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(payload.get("insufficient_data_reason").and_then(serde_json::Value::as_str), None);

    let execution_mode =
        payload.get("execution_mode").and_then(serde_json::Value::as_str).expect("execution_mode");
    assert!(matches!(execution_mode, "real_tool" | "fallback_proxy"));
    assert!(payload.get("tool_ok").and_then(serde_json::Value::as_bool).is_some());

    let cluster_headers = payload
        .get("cluster_headers")
        .and_then(serde_json::Value::as_array)
        .expect("cluster headers");
    assert_eq!(
        cluster_headers.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>(),
        vec!["cluster_1", "cluster_2"]
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows.iter()
            .filter_map(|row| row.get("sample_id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert!(rows.iter().all(|row| row.get("K").and_then(serde_json::Value::as_u64) == Some(2)));
    assert!(rows.iter().all(|row| row.get("k").is_none()), "row field must stay uppercase `K`");
    assert!(rows
        .iter()
        .all(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("complete")));
    for row in rows {
        let cluster_1 =
            row.get("cluster_1").and_then(serde_json::Value::as_f64).expect("cluster_1");
        let cluster_2 =
            row.get("cluster_2").and_then(serde_json::Value::as_f64).expect("cluster_2");
        assert!(((cluster_1 + cluster_2) - 1.0).abs() <= 1e-6);
    }

    let repo_root = support::repo_root().expect("repo root");
    let persisted_path = repo_root.join("target/local-smoke/vcf.admixture/plink2/admixture.json");
    let persisted_raw = std::fs::read_to_string(&persisted_path).expect("read persisted report");
    let persisted: serde_json::Value = serde_json::from_str(&persisted_raw).expect("parse report");
    assert_eq!(
        persisted.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_admixture_smoke.v1")
    );
    assert_eq!(persisted.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
}
