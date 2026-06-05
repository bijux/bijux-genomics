#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_parser_coverage_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-parser-coverage"])
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
    assert_eq!(rendered_path.trim(), "target/bench-readiness/bam-parser-coverage.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM parser coverage TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tparser_coverage\tparser_status\tsupport_status\tadapter_status\tcorpus_status\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 49);
    assert!(
        rows.iter().any(|row| {
            row == &"samtools\tbam.validate\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-bam-mini\trow `bam.validate` / `samtools` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output"
        }),
        "TSV must retain the governed BAM validation parser row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"mapdamage2\tbam.damage\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-adna-damage-mini\trow `bam.damage` / `mapdamage2` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output"
        }),
        "TSV must retain the governed BAM damage parser row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"angsd\tbam.genotyping\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-genotyping-mini\trow `bam.genotyping` / `angsd` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output"
        }),
        "TSV must retain the governed BAM genotyping parser row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bwa\tbam.align\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-mini\trow `bam.align` / `bwa` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output"
        }),
        "TSV must retain the governed BAM bwa alignment parser row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bowtie2\tbam.align\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-mini\trow `bam.align` / `bowtie2` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output"
        }),
        "TSV must retain the governed BAM bowtie2 alignment parser row"
    );
}
