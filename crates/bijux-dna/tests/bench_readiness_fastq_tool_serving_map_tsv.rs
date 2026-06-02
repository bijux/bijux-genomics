#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_tool_serving_map_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-tool-serving-map"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/fastq-tool-serving-map.tsv");
    assert!(tsv_path.is_file(), "FASTQ tool serving map TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read FASTQ tool serving map");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status")
    );
    let rows = lines.collect::<Vec<_>>();
    assert!(!rows.is_empty(), "TSV must contain FASTQ tool rows");
    assert!(
        rows.iter().any(|row| {
            row == &"fastqc\tfastq.validate_reads\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini"
        }),
        "TSV must retain the governed fastqc validation row"
    );
    for tool_id in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.validate_reads\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed validation row for {tool_id}"
        );
    }
    for tool_id in ["seqfu", "seqkit", "seqkit_stats"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.profile_reads\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed profile-reads row for {tool_id}"
        );
    }
    for tool_id in ["fastp", "prinseq", "seqfu", "seqkit_stats"] {
        assert!(
            rows.iter().any(|row| {
                row == &format!(
                    "{tool_id}\tfastq.profile_read_lengths\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini"
                )
            }),
            "TSV must retain the governed profile-read-lengths row for {tool_id}"
        );
    }
}
