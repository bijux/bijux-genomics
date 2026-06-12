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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_corpus_incompatibility_reports_governed_blockers() {
    let payload = run_cli_json(&["bench", "readiness", "render-corpus-incompatibility", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.corpus_incompatibility.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/corpus-incompatibility.tsv")
    );
    assert_eq!(payload.get("fixture_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        payload.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64),
        Some(115)
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(48));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(330));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(198)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(132)
    );
    assert_eq!(
        payload
            .get("incompatibility_kind_counts")
            .and_then(|value| value.get("missing_amplicon_asv_contract"))
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        payload
            .get("incompatibility_kind_counts")
            .and_then(|value| value.get("missing_taxonomy_database_bundle"))
            .and_then(serde_json::Value::as_u64),
        Some(8)
    );
    assert_eq!(
        payload
            .get("incompatibility_kind_counts")
            .and_then(|value| value.get("missing_kinship_pair_manifest"))
            .and_then(serde_json::Value::as_u64),
        Some(8)
    );
    assert_eq!(
        payload
            .get("incompatibility_kind_counts")
            .and_then(|value| value.get("wrong_corpus_family"))
            .and_then(serde_json::Value::as_u64),
        Some(312)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");

    let asv = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.infer_asvs")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("dada2")
                && row.get("incompatible_fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        })
        .expect("ASV incompatibility row");
    assert_eq!(
        asv.get("required_fixture_id").and_then(serde_json::Value::as_str),
        Some("corpus-03-amplicon-mini")
    );
    assert_eq!(
        asv.get("incompatibility_kind").and_then(serde_json::Value::as_str),
        Some("missing_amplicon_asv_contract")
    );
    assert!(
        asv.get("required_contract")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.contains("expected_asvs_path=")),
        "ASV row must carry the governed expected_asvs contract"
    );

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
                && row.get("incompatible_fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        })
        .expect("taxonomy incompatibility row");
    assert_eq!(
        taxonomy.get("incompatibility_kind").and_then(serde_json::Value::as_str),
        Some("missing_taxonomy_database_bundle")
    );
    assert!(
        taxonomy.get("required_assets").and_then(serde_json::Value::as_str).is_some_and(|value| {
            value.contains("database_artifact_id=taxonomy_db")
                && value.contains("taxonomy_database_root=taxonomy_reference")
        }),
        "taxonomy row must carry the governed taxonomy asset bundle"
    );

    let kinship = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("king")
                && row.get("incompatible_fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-bam-mini")
        })
        .expect("kinship incompatibility row");
    assert_eq!(
        kinship.get("incompatibility_kind").and_then(serde_json::Value::as_str),
        Some("missing_kinship_pair_manifest")
    );
    assert!(
        kinship
            .get("required_assets")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.contains("reference_panel=human_like_relatedness_panel")),
        "kinship row must carry the governed relatedness panel asset"
    );
    assert!(
        kinship.get("required_contract").and_then(serde_json::Value::as_str).is_some_and(|value| {
            value.contains("cases=") && value.contains("reference_build=grch38")
        }),
        "kinship row must carry the governed pair-manifest contract"
    );
}
