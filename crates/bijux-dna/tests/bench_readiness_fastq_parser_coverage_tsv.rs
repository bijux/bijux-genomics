#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_parser_coverage_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-parser-coverage"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/fastq-parser-coverage.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ parser coverage TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tparser_coverage\tparser_status\tsupport_status\tadapter_status\tcorpus_status\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 67);
    assert!(
        rows.iter().any(|row| {
            row == &"fastqc\tfastq.validate_reads\tcovered\tcomparable\tobserver_specialized_benchmark\trunnable\tfixture:corpus-01-mini\trow `fastq.validate_reads` / `fastqc` has governed support, adapter-backed command rendering, fixture-backed corpus coverage, and normalized parser output"
        }),
        "TSV must retain the governed FASTQ validation parser row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bijux_dna\tfastq.detect_duplicates_premerge\tcovered\tparse_normalized\tgoverned_execution\trunnable\tfixture:corpus-01-mini\trow `fastq.detect_duplicates_premerge` / `bijux_dna` has governed support, adapter-backed command rendering, fixture-backed corpus coverage, and normalized parser output"
        }),
        "TSV must retain the governed duplicate-signal parser row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"kraken2\tfastq.screen_taxonomy\tcovered\tbenchmark_normalized\tgoverned_benchmark_cohort\trunnable\tfixture:corpus-02-edna-mini\trow `fastq.screen_taxonomy` / `kraken2` has governed support, adapter-backed command rendering, fixture-backed corpus coverage, and normalized parser output"
        }),
        "TSV must retain the governed taxonomy parser row"
    );
}
