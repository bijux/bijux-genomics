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
fn bench_local_corpus_stage_compatibility_reports_governed_51_stage_slice() {
    let payload =
        run_cli_json(&["bench", "local", "validate-corpus-stage-compatibility", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_corpus_stage_compatibility_validation.v1")
    );
    assert_eq!(
        payload.get("matrix_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/corpus-stage-compatibility.toml")
    );
    assert_eq!(payload.get("fixture_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        payload.get("fixture_backed_stage_count").and_then(serde_json::Value::as_u64),
        Some(22)
    );
    assert_eq!(
        payload.get("planner_only_stage_count").and_then(serde_json::Value::as_u64),
        Some(29)
    );

    let stages = payload.get("stages").and_then(serde_json::Value::as_array).expect("stages array");
    assert_eq!(stages.len(), 51);
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "detect_duplicates_premerge must map to the governed general FASTQ corpus once duplicate-signal coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_adapters")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "detect_adapters must map to the governed general FASTQ corpus once adapter-hit coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_reads")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "filter_reads must map to the governed general FASTQ corpus once filter-signal coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_polyg_tails")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "trim_polyg_tails must map to the governed general FASTQ corpus once poly-G coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_terminal_damage")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-01-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "trim_terminal_damage must map to the governed general FASTQ corpus once aDNA-like fixture coverage is owned there"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
                && stage.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("fixture")
        }),
        "taxonomy stage must map to the governed eDNA corpus"
    );
    assert!(
        stages.iter().any(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
                && stage.get("compatibility_kind").and_then(serde_json::Value::as_str)
                    == Some("planner_only")
                && stage.get("fixture_id").is_some_and(serde_json::Value::is_null)
        }),
        "report_qc must stay explicit about its planner-only corpus gap"
    );
}
