#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_corpus_centric_report_tracks_governed_corpus_coverage() {
    let payload = run_cli_json(&["bench", "readiness", "render-corpus-centric-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.corpus_centric_report.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/corpus-centric-report.md")
    );
    assert_eq!(payload.get("corpus_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(50));
    assert_eq!(payload.get("tool_row_count").and_then(serde_json::Value::as_u64), Some(121));
    assert_eq!(
        payload.get("benchmark_ready_tool_row_count").and_then(serde_json::Value::as_u64),
        Some(118)
    );
    assert_eq!(payload.get("blocked_tool_row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("blocked_corpus_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(24)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(26)
    );
    assert_eq!(
        payload
            .get("corpus_stage_counts")
            .and_then(|value| value.get("reference-index-assets"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("corpus_stage_counts")
            .and_then(|value| value.get("corpus-02"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("corpus_stage_counts")
            .and_then(|value| value.get("corpus-03"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        payload
            .get("corpus_stage_counts")
            .and_then(|value| value.get("corpus-01-adna-bam"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        payload
            .get("corpus_stage_counts")
            .and_then(|value| value.get("corpus-01-genotyping"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("corpus_stage_counts")
            .and_then(|value| value.get("corpus-01-kinship"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let corpora =
        payload.get("corpora").and_then(serde_json::Value::as_array).expect("corpora array");
    assert_eq!(corpora.len(), 8);

    let corpus_02 = corpora
        .iter()
        .find(|corpus| {
            corpus.get("corpus_family_id").and_then(serde_json::Value::as_str) == Some("corpus-02")
        })
        .expect("corpus-02");
    assert_eq!(
        corpus_02.get("fixture_ids").and_then(serde_json::Value::as_array).map(|values| {
            values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
        }),
        Some(vec!["corpus-02-edna-mini"])
    );
    assert_eq!(corpus_02.get("stage_count").and_then(serde_json::Value::as_u64), Some(1));
    let corpus_02_stages =
        corpus_02.get("stages").and_then(serde_json::Value::as_array).expect("corpus-02 stages");
    let taxonomy = corpus_02_stages
        .iter()
        .find(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
        })
        .expect("taxonomy stage");
    assert_eq!(taxonomy.get("tool_count").and_then(serde_json::Value::as_u64), Some(4));

    let reference_index = corpora
        .iter()
        .find(|corpus| {
            corpus.get("corpus_family_id").and_then(serde_json::Value::as_str)
                == Some("reference-index-assets")
        })
        .expect("reference index corpus");
    assert_eq!(
        reference_index.get("fixture_ids").and_then(serde_json::Value::as_array).map(|values| {
            values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
        }),
        Some(vec!["reference-index-assets"])
    );
    assert_eq!(reference_index.get("stage_count").and_then(serde_json::Value::as_u64), Some(1));

    let corpus_03 = corpora
        .iter()
        .find(|corpus| {
            corpus.get("corpus_family_id").and_then(serde_json::Value::as_str) == Some("corpus-03")
        })
        .expect("corpus-03");
    assert_eq!(
        corpus_03.get("blocked_stage_ids").and_then(serde_json::Value::as_array).map(|values| {
            values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
        }),
        Some(vec![])
    );
    let corpus_03_stages =
        corpus_03.get("stages").and_then(serde_json::Value::as_array).expect("corpus-03 stages");
    for (stage_id, tool_count) in [
        ("fastq.cluster_otus", 1_u64),
        ("fastq.infer_asvs", 1_u64),
        ("fastq.normalize_abundance", 1_u64),
        ("fastq.remove_chimeras", 1_u64),
    ] {
        let stage = corpus_03_stages
            .iter()
            .find(|stage| {
                stage.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
            })
            .expect("amplicon stage");
        assert_eq!(stage.get("tool_count").and_then(serde_json::Value::as_u64), Some(tool_count));
    }

    let adna = corpora
        .iter()
        .find(|corpus| {
            corpus.get("corpus_family_id").and_then(serde_json::Value::as_str)
                == Some("corpus-01-adna-bam")
        })
        .expect("adna corpus");
    let adna_stages =
        adna.get("stages").and_then(serde_json::Value::as_array).expect("adna stages");
    let damage = adna_stages
        .iter()
        .find(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
        })
        .expect("damage stage");
    assert_eq!(damage.get("tool_count").and_then(serde_json::Value::as_u64), Some(6));
    let contamination = adna_stages
        .iter()
        .find(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
        })
        .expect("contamination stage");
    assert_eq!(contamination.get("tool_count").and_then(serde_json::Value::as_u64), Some(3));

    let genotyping = corpora
        .iter()
        .find(|corpus| {
            corpus.get("corpus_family_id").and_then(serde_json::Value::as_str)
                == Some("corpus-01-genotyping")
        })
        .expect("genotyping corpus");
    assert_eq!(
        genotyping.get("stages").and_then(serde_json::Value::as_array).expect("genotyping stages")
            [0]
        .get("stage_id")
        .and_then(serde_json::Value::as_str),
        Some("bam.genotyping")
    );

    let kinship = corpora
        .iter()
        .find(|corpus| {
            corpus.get("corpus_family_id").and_then(serde_json::Value::as_str)
                == Some("corpus-01-kinship")
        })
        .expect("kinship corpus");
    assert_eq!(
        kinship.get("stages").and_then(serde_json::Value::as_array).expect("kinship stages")[0]
            .get("stage_id")
            .and_then(serde_json::Value::as_str),
        Some("bam.kinship")
    );
}
