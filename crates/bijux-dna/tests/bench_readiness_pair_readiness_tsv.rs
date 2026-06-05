#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
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

fn parse_tsv_row<'a>(header: &[&'a str], row: &'a str) -> BTreeMap<&'a str, &'a str> {
    header.iter().copied().zip(row.split('\t')).collect()
}

#[test]
fn bench_readiness_pair_readiness_writes_gap_status_columns() {
    let output = run_cli(&["bench", "readiness", "render-pair-readiness"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "target/bench-readiness/pair-readiness.tsv");

    let repo_root = support::repo_root().expect("repo root");
    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read pair readiness TSV");
    let mut lines = tsv.lines();
    let header_line = lines.next().expect("header line");
    let header = header_line.split('\t').collect::<Vec<_>>();
    assert_eq!(
        header,
        vec![
            "domain",
            "stage_id",
            "tool_id",
            "benchmark_status",
            "readiness_gap",
            "support_status",
            "adapter_status",
            "parser_status",
            "corpus_status",
            "asset_status",
            "required_asset_roles",
            "assigned_asset_roles",
            "reason",
        ]
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 123);

    let taxonomy = rows
        .iter()
        .map(|row| parse_tsv_row(&header, row))
        .find(|row| {
            row.get("domain") == Some(&"fastq")
                && row.get("stage_id") == Some(&"fastq.screen_taxonomy")
                && row.get("tool_id") == Some(&"kraken2")
        })
        .expect("taxonomy readiness TSV row");
    assert_eq!(taxonomy.get("adapter_status"), Some(&"runnable"));
    assert_eq!(taxonomy.get("parser_status"), Some(&"benchmark_normalized"));
    assert_eq!(taxonomy.get("corpus_status"), Some(&"fixture:corpus-02-edna-mini"));
    assert_eq!(taxonomy.get("asset_status"), Some(&"assigned"));

    let index_reference = rows
        .iter()
        .map(|row| parse_tsv_row(&header, row))
        .find(|row| {
            row.get("domain") == Some(&"fastq")
                && row.get("stage_id") == Some(&"fastq.index_reference")
                && row.get("tool_id") == Some(&"bowtie2_build")
        })
        .expect("index-reference readiness TSV row");
    assert_eq!(index_reference.get("benchmark_status"), Some(&"not_benchmark_ready"));
    assert_eq!(index_reference.get("readiness_gap"), Some(&"corpus"));
    assert_eq!(index_reference.get("corpus_status"), Some(&"planner_only"));
    assert_eq!(index_reference.get("asset_status"), Some(&"assigned"));
}
