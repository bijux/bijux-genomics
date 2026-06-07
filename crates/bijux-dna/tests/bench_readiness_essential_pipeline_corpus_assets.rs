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
fn bench_readiness_essential_pipeline_corpus_assets_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-essential-pipeline-corpus-assets", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipeline_corpus_assets.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/essential-pipeline-corpus-assets.tsv")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(payload.get("resolved_row_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(payload.get("corpus_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(payload.get("asset_profile_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(
        payload
            .get("pipeline_row_counts")
            .and_then(|value| value.get("diploid-small-fastq-bam-vcf"))
            .and_then(serde_json::Value::as_u64),
        Some(16)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("vcf"))
            .and_then(serde_json::Value::as_u64),
        Some(33)
    );
    assert_eq!(
        payload
            .get("status_counts")
            .and_then(|value| value.get("resolved_fixture_bound"))
            .and_then(serde_json::Value::as_u64),
        Some(51)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 93);

    let amplicon_validate = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("amplicon-asv-otu-no-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.validate_reads")
        })
        .expect("amplicon validate row");
    assert_eq!(
        amplicon_validate.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-03-amplicon-mini")
    );
    assert_eq!(
        amplicon_validate.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("corpus_only")
    );
    assert_eq!(
        amplicon_validate.get("status").and_then(serde_json::Value::as_str),
        Some("resolved_pipeline_bound")
    );

    let edna_taxonomy = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("edna-taxonomy-no-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
        })
        .expect("edna taxonomy row");
    assert_eq!(
        edna_taxonomy.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-02")
    );
    assert_eq!(
        edna_taxonomy.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("taxonomy_database.root+taxonomy_expected_truth_table")
    );
    assert!(
        edna_taxonomy
            .get("input_paths")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.contains("external:taxonomy_database.root")),
        "eDNA taxonomy row must keep explicit taxonomy database bindings"
    );

    let reference_panel = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("reference-panel-imputation")
                && row.get("node_id").and_then(serde_json::Value::as_str)
                    == Some("vcf.prepare_reference_panel")
        })
        .expect("reference panel preparation row");
    assert_eq!(
        reference_panel.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        reference_panel.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some(
            "genetic_map_contract+reference_dict_contract+reference_fai_contract+reference_fasta_contract+reference_panel_id_contract+reference_panel_lock_contract"
        )
    );
    assert!(
        reference_panel
            .get("input_paths")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.contains("external:corpus.reference_panel_vcf")),
        "reference-panel preparation row must keep explicit panel corpus bindings"
    );
}
