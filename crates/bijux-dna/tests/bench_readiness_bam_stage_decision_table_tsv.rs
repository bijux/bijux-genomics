#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_stage_decision_table_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-stage-decision-table"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/bam-stage-decision-table.tsv");
    assert!(tsv_path.is_file(), "BAM stage decision table TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read BAM stage decision table");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("stage_id\tdecision\tprimary_tool_id\tselected_tool_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\treason")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 24, "TSV must retain one governed BAM stage decision row per stage");
    assert!(
        rows.iter().any(|row| {
            row == &"bam.validate\tbenchmark_ready\tsamtools\tsamtools\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.validate` is benchmark_ready via `samtools` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.validate row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.damage\tbenchmark_ready\tmapdamage2\tmapdamage2\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-adna-damage-mini\tstage `bam.damage` is benchmark_ready via `mapdamage2` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.damage row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.authenticity\tbenchmark_ready\tauthenticct\tauthenticct\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.authenticity` is benchmark_ready via `authenticct` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.authenticity row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.qc_pre\tbenchmark_ready\tmultiqc\tmultiqc\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.qc_pre` is benchmark_ready via `multiqc` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.qc_pre row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.mapping_summary\tbenchmark_ready\tsamtools\tsamtools\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.mapping_summary` is benchmark_ready via `samtools` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.mapping_summary row"
    );
    assert!(
        rows.iter().any(|row| {
            let columns = row.split('\t').collect::<Vec<_>>();
            columns.len() == 9
                && columns[0] == "bam.sex"
                && columns[1] == "benchmark_ready"
                && columns[2] == "rxy"
                && columns[3] == "rxy"
                && columns[4] == "supported"
                && columns[5] == "runnable"
                && columns[6] == "parser_fixture_validated"
                && columns[7] == "fixture:corpus-01-bam-mini"
                && columns[8].contains("benchmark_ready via `rxy`")
        }),
        "TSV must retain the governed benchmark-ready bam.sex row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.mapq_filter\tbenchmark_ready\tsamtools\tsamtools\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.mapq_filter` is benchmark_ready via `samtools` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.mapq_filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.filter\tbenchmark_ready\tsamtools\tsamtools\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.filter` is benchmark_ready via `samtools` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.markdup\tbenchmark_ready\tpicard\tpicard\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.markdup` is benchmark_ready via `picard` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.markdup row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.length_filter\tbenchmark_ready\tsamtools\tsamtools\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.length_filter` is benchmark_ready via `samtools` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.length_filter row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.duplication_metrics\tbenchmark_ready\tsamtools\tsamtools\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.duplication_metrics` is benchmark_ready via `samtools` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.duplication_metrics row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.coverage\tbenchmark_ready\tmosdepth\tmosdepth\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.coverage` is benchmark_ready via `mosdepth` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.coverage row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.gc_bias\tbenchmark_ready\tpicard\tpicard\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.gc_bias` is benchmark_ready via `picard` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.gc_bias row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.insert_size\tbenchmark_ready\tpicard\tpicard\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.insert_size` is benchmark_ready via `picard` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.insert_size row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.endogenous_content\tbenchmark_ready\tsamtools\tsamtools\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.endogenous_content` is benchmark_ready via `samtools` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.endogenous_content row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.overlap_correction\tbenchmark_ready\tbamutil\tbamutil\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.overlap_correction` is benchmark_ready via `bamutil` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.overlap_correction row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.haplogroups\tbenchmark_ready\tyleaf\tyleaf\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.haplogroups` is benchmark_ready via `yleaf` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.haplogroups row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.kinship\tbenchmark_ready\tking\tking\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.kinship` is benchmark_ready via `king` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.kinship row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.align\tbenchmark_ready\tbwa\tbwa\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-mini\tstage `bam.align` is benchmark_ready via `bwa` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.align row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.bias_mitigation\tbenchmark_ready\tmapdamage2\tmapdamage2\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.bias_mitigation` is benchmark_ready via `mapdamage2` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.bias_mitigation row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.recalibration\tbenchmark_ready\tgatk\tgatk\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.recalibration` is benchmark_ready via `gatk` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must retain the governed benchmark-ready bam.recalibration row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.complexity\tbenchmark_ready\tpreseq\tpreseq\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.complexity` is benchmark_ready via `preseq` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must publish the governed benchmark-ready classification for bam.complexity"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.genotyping\tbenchmark_ready\tangsd\tangsd\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.genotyping` is benchmark_ready via `angsd` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must publish the governed benchmark-ready classification for bam.genotyping"
    );
}
