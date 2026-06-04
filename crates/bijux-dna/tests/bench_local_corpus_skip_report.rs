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
fn bench_local_corpus_skip_report_writes_governed_skip_manifest() {
    let payload = run_cli_json(&["bench", "local", "render-corpus-skip-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_corpus_skip_report.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/corpus-skip-report.json")
    );
    assert_eq!(payload.get("fixture_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("skip_count").and_then(serde_json::Value::as_u64), Some(88));
    assert_eq!(
        payload.get("planner_only_stage_count").and_then(serde_json::Value::as_u64),
        Some(29)
    );

    let skips = payload.get("skips").and_then(serde_json::Value::as_array).expect("skips array");
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed detect-duplicates skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
        }),
        "incompatible corpora must name their governed replacement corpus"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed filter-reads skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_polyg_tails")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed trim-polyg skips must name the governed FASTQ corpus replacement"
    );
    assert!(
        skips.iter().any(|skip| {
            skip.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_terminal_damage")
                && skip.get("corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && skip.get("replacement_corpus_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
        }),
        "fixture-backed trim-terminal-damage skips must name the governed FASTQ corpus replacement"
    );

    let planner_only = payload
        .get("planner_only_stages")
        .and_then(serde_json::Value::as_array)
        .expect("planner_only_stages array");
    assert!(
        planner_only.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        }),
        "planner-only stages must stay explicit instead of disappearing"
    );
}
