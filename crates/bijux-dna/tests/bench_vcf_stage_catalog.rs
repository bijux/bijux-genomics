#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::process::Command;

use bijux_dna_domain_vcf::VCF_STAGE_ORDER_DOWNSTREAM;
use bijux_dna_stages_vcf::stage_specs::vcf_stage_catalog;

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
fn bench_vcf_stage_catalog_matches_code_catalog() {
    let payload = run_cli_json(&["bench", "local", "render-vcf-stage-catalog", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_stage_catalog.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/vcf-stage-catalog.toml")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("supported_stage_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("planned_stage_count").and_then(serde_json::Value::as_u64), Some(12));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), vcf_stage_catalog().len());

    let observed_stage_ids = rows
        .iter()
        .map(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str).expect("stage_id").to_string()
        })
        .collect::<Vec<_>>();
    let expected_order = VCF_STAGE_ORDER_DOWNSTREAM
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        observed_stage_ids, expected_order,
        "rendered VCF stage catalog must preserve governed downstream order"
    );

    let observed_stage_set = observed_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    let expected_stage_set =
        vcf_stage_catalog().iter().map(|spec| spec.stage_id.to_string()).collect::<BTreeSet<_>>();
    assert_eq!(
        observed_stage_set, expected_stage_set,
        "rendered VCF stage catalog must cover the authoritative stage specs exactly"
    );

    let prepare_reference_panel = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
        })
        .expect("prepare reference panel row");
    assert_eq!(
        prepare_reference_panel
            .get("required_assets")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "genetic_map",
            "reference_dict",
            "reference_fai",
            "reference_fasta",
            "reference_panel_lock",
            "vcf_index",
        ])
    );
    assert_eq!(
        prepare_reference_panel.get("benchmark_category").and_then(serde_json::Value::as_str),
        Some("reference_panel_preparation")
    );
    assert_eq!(
        prepare_reference_panel.get("local_smoke_mode").and_then(serde_json::Value::as_str),
        Some("vcf_reference_panel")
    );

    let phasing = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing"))
        .expect("phasing row");
    assert_eq!(
        phasing.get("local_smoke_mode").and_then(serde_json::Value::as_str),
        Some("vcf_cohort_with_panel")
    );
    assert!(phasing.get("required_assets").and_then(serde_json::Value::as_array).is_some_and(
        |assets| {
            assets.iter().any(|value| value.as_str() == Some("genetic_map"))
                && assets.iter().any(|value| value.as_str() == Some("reference_panel_lock"))
        }
    ));

    let population_structure = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
        })
        .expect("population structure row");
    assert_eq!(
        population_structure
            .get("output_types")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["json"])
    );
    assert!(population_structure
        .get("required_assets")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|assets| {
            assets.iter().any(|value| value.as_str() == Some("sample_metadata_manifest"))
                && assets.iter().any(|value| value.as_str() == Some("vcf_index"))
        }));

    let pca = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca"))
        .expect("pca row");
    assert_eq!(
        pca.get("metrics_schema_id").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.pca.v1")
    );

    let admixture = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
        })
        .expect("admixture row");
    assert_eq!(
        admixture.get("metrics_schema_id").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.admixture.v1")
    );
}
