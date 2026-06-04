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
            row == &"bam.coverage\tneeds_corpus\tmosdepth\tmosdepth\tsupported\tplannable\tparser_fixture_validated\tplanner_only\tstage `bam.coverage` has parser-validated BAM benchmark tooling via `mosdepth` but still resolves only planner-only corpus coverage"
        }),
        "TSV must retain the governed corpus blocker for bam.coverage"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.align\tneeds_parser\tbwa\tbwa\tsupported\trunnable\tartifact_contract_only\tfixture:corpus-01-mini\tstage `bam.align` has a supported adapter-backed BAM benchmark row via `bwa` but no parser-fixture-validated result normalizer"
        }),
        "TSV must retain the governed parser blocker for bam.align"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.bias_mitigation\tneeds_parser\tsamtools\tmapdamage2\tsupported\trunnable\tartifact_contract_only\tplanner_only\tstage `bam.bias_mitigation` has a supported adapter-backed BAM benchmark row via `mapdamage2` but no parser-fixture-validated result normalizer; primary `samtools` is not currently the strongest eligible row"
        }),
        "TSV must retain the governed fallback parser blocker for bam.bias_mitigation"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.complexity\tbenchmark_ready\tpreseq\tpreseq\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\tstage `bam.complexity` is benchmark_ready via `preseq` with a fixture-backed parser-validated BAM benchmark row"
        }),
        "TSV must publish the governed benchmark-ready classification for bam.complexity"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.genotyping\tfuture_not_in_hpc_round\t\tangsd\tplanned\trunnable\tartifact_contract_only\tplanner_only\tstage `bam.genotyping` is not yet in the governed BAM benchmark registry; strongest admitted row `angsd` remains `planned`"
        }),
        "TSV must retain the governed future classification for bam.genotyping"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.recalibration\tfuture_not_in_hpc_round\t\tgatk\tplanned\tplannable\tartifact_contract_only\tplanner_only\tstage `bam.recalibration` is not yet in the governed BAM benchmark registry; strongest admitted row `gatk` remains `planned`"
        }),
        "TSV must retain the governed future classification for bam.recalibration"
    );
}
