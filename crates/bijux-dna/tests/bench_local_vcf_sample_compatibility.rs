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
fn bench_local_vcf_sample_compatibility_reports_cohort_label_parity() {
    let payload = run_cli_json(&["bench", "local", "validate-vcf-sample-compatibility", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_sample_compatibility.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/vcf/sample-compatibility.json")
    );
    assert_eq!(payload.get("corpus_id").and_then(serde_json::Value::as_str), Some("vcf-mini"));
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("compatible"));
    assert_eq!(
        payload
            .get("source_variant_roles")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["multisample", "phased"])
    );
    assert_eq!(
        payload
            .get("downstream_stage_ids")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "vcf.population_structure",
            "vcf.pca",
            "vcf.admixture",
            "vcf.roh",
            "vcf.ibd",
        ])
    );
    assert_eq!(
        payload
            .get("vcf_samples")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["sample_a", "sample_b", "sample_c", "sample_d"])
    );
    assert_eq!(
        payload
            .get("metadata_samples")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "panel_ref_1",
            "panel_ref_2",
            "sample_a",
            "sample_b",
            "sample_c",
            "sample_d",
        ])
    );
    assert!(
        payload
            .get("missing_metadata")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| values.is_empty())
    );
    assert_eq!(
        payload
            .get("extra_metadata")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["panel_ref_1", "panel_ref_2"])
    );
    assert_eq!(
        payload
            .get("population_labels")
            .and_then(serde_json::Value::as_object)
            .and_then(|labels| labels.get("sample_a"))
            .and_then(serde_json::Value::as_str),
        Some("Cohort Alpha")
    );
    assert_eq!(
        payload
            .get("population_labels")
            .and_then(serde_json::Value::as_object)
            .and_then(|labels| labels.get("sample_d"))
            .and_then(serde_json::Value::as_str),
        Some("Cohort Beta")
    );
    assert_eq!(
        payload
            .get("sex_labels")
            .and_then(serde_json::Value::as_object)
            .and_then(|labels| labels.get("sample_a"))
            .and_then(serde_json::Value::as_str),
        Some("female")
    );
    assert_eq!(
        payload
            .get("sex_labels")
            .and_then(serde_json::Value::as_object)
            .and_then(|labels| labels.get("sample_b"))
            .and_then(serde_json::Value::as_str),
        Some("male")
    );
    assert!(
        payload
            .get("missing_population_labels")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| values.is_empty())
    );
    assert!(
        payload
            .get("missing_sex_labels")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| values.is_empty())
    );
}
