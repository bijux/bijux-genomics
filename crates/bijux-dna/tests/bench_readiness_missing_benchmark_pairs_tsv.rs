#![allow(clippy::expect_used, clippy::too_many_lines)]

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

    let tsv_path = repo_root.join("benchmarks/readiness/missing-benchmark-pairs.tsv");
    assert!(tsv_path.is_file(), "missing benchmark pairs TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read missing benchmark pairs");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("domain\tstage_id\ttool_id\tsupport_status\tregistered_tool_ids\treason")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 0, "TSV must retain the governed missing-pair row count");
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.overlap_correction\t")),
        "TSV must not retain a missing benchmark-pair row for bam.overlap_correction once samtools is no longer admitted for that stage"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.align\t")),
        "TSV must not retain a missing benchmark-pair row for bam.align once the admitted aligner set is fully covered"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.damage\t")),
        "TSV must not retain a missing benchmark-pair row for bam.damage once all declared tool rows are covered"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.authenticity\t")),
        "TSV must not retain a missing benchmark-pair row for bam.authenticity once damageprofiler is covered"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.haplogroups\t")),
        "TSV must not retain a missing benchmark-pair row for bam.haplogroups in the current governed matrix"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.endogenous_content\t")),
        "TSV must not retain a missing benchmark-pair row for bam.endogenous_content"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.filter\t")),
        "TSV must not retain a missing benchmark-pair row for bam.filter"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.mapq_filter\t")),
        "TSV must not retain a missing benchmark-pair row for bam.mapq_filter"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.length_filter\t")),
        "TSV must not retain a missing benchmark-pair row for bam.length_filter"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.duplication_metrics\t")),
        "TSV must not retain a missing benchmark-pair row for bam.duplication_metrics"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.insert_size\t")),
        "TSV must not retain a missing benchmark-pair row for bam.insert_size"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.gc_bias\t")),
        "TSV must not retain a missing benchmark-pair row for bam.gc_bias"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.complexity\t")),
        "TSV must not retain a missing benchmark-pair row for bam.complexity"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.markdup\t")),
        "TSV must not retain a missing benchmark-pair row for bam.markdup"
    );
}
