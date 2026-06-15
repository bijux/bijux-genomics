#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_corpus_assignment_prints_governed_output_path() {
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
        .args(["bench", "readiness", "render-fastq-corpus-assignment"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "benchmarks/readiness/fastq-corpus-assignment.tsv"
    );
}

#[test]
fn bench_readiness_fastq_corpus_assignment_json_keeps_all_taxonomy_tools_on_corpus_02() {
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
        .args(["bench", "readiness", "render-fastq-corpus-assignment", "--json"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let taxonomy_rows = rows
        .iter()
        .filter(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
        })
        .collect::<Vec<_>>();
    assert_eq!(taxonomy_rows.len(), 4);
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(taxonomy_rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("assignment_status").and_then(serde_json::Value::as_str)
                    == Some("assigned")
                && row.get("corpus_family_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02")
                && row.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-02-edna-mini")
        }));
    }
}

#[test]
fn bench_readiness_fastq_corpus_assignment_json_keeps_all_amplicon_tools_on_corpus_03() {
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
        .args(["bench", "readiness", "render-fastq-corpus-assignment", "--json"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    for (tool_id, stage_id) in [
        ("cutadapt", "fastq.normalize_primers"),
        ("vsearch", "fastq.remove_chimeras"),
        ("dada2", "fastq.infer_asvs"),
        ("vsearch", "fastq.cluster_otus"),
        ("seqkit", "fastq.normalize_abundance"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("assignment_status").and_then(serde_json::Value::as_str)
                    == Some("assigned")
                && row.get("corpus_family_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-03")
                && row.get("fixture_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-03-amplicon-mini")
        }));
    }
}
