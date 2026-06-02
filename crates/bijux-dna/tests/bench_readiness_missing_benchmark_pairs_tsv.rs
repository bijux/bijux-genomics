#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_missing_benchmark_pairs_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-missing-benchmark-pairs"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/missing-benchmark-pairs.tsv");
    assert!(tsv_path.is_file(), "missing benchmark pairs TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read missing benchmark pairs");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("domain\tstage_id\ttool_id\tsupport_status\tregistered_tool_ids\treason")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 6, "TSV must retain the governed missing-pair row count");
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.align\tsamtools\tplanned\tbowtie2,bwa\tdomain-compatible pair `bam.align` / `samtools` is admitted by governed contracts but absent from the benchmark matrix; current registered tools: bowtie2, bwa"
        }),
        "TSV must retain the governed bam.align / samtools gap"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.damage\taddeam\tsupported\tmapdamage2,pmdtools,pydamage\tdomain-compatible pair `bam.damage` / `addeam` is admitted by governed contracts but absent from the benchmark matrix; current registered tools: mapdamage2, pmdtools, pydamage"
        }),
        "TSV must retain the governed bam.damage / addeam gap"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.filter\t")),
        "TSV must not retain a missing benchmark-pair row for bam.filter"
    );
}
