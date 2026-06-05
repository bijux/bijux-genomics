#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_corpus_assignment_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-corpus-assignment"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/bam-corpus-assignment.tsv");
    assert!(tsv_path.is_file(), "BAM corpus assignment TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read BAM corpus assignment");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_family_id\tfixture_id\treason"
        )
    );
    let rows = lines.map(|line| line.split('\t').collect::<Vec<_>>()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 49, "TSV must retain the governed BAM row count");
    assert!(rows.iter().any(|row| {
        row[0] == "authenticct"
            && row[1] == "bam.authenticity"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-adna-bam"
            && row[7] == "corpus-01-adna-damage-mini"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "angsd"
            && row[1] == "bam.genotyping"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-genotyping"
            && row[7] == "corpus-01-genotyping-mini"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "king"
            && row[1] == "bam.kinship"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-kinship"
            && row[7] == "corpus-01-kinship-mini"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "samtools"
            && row[1] == "bam.qc_pre"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01-bam"
            && row[7] == "corpus-01-bam-mini"
    }));
    assert!(rows.iter().any(|row| {
        row[0] == "bwa"
            && row[1] == "bam.align"
            && row[2] == "benchmark_ready"
            && row[3] == "supported"
            && row[4] == "runnable"
            && row[5] == "parser_fixture_validated"
            && row[6] == "corpus-01"
            && row[7] == "corpus-01-mini"
    }));
}
