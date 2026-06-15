#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_report_map_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-report-map"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/fastq/fastq-report-map.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ report map TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "result_row_id\tstage_id\ttool_id\tcorpus_family_id\tfixture_id\tsample_scope\tcanonical_stage_rank\treadiness_kind\tstage_kind\tcriticality\tsupport_status\treport_section_id\treport_section_title\tsummary_table_id\tsummary_table_title\tmetric_classes\tmutates_fastq\tproduces_reports_only\treport_focus\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 69, "TSV must retain every FASTQ expected-result reporting row");
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "fastq:reference-index-assets:fastq.index_reference:asset-set:bowtie2_build\tfastq.index_reference\tbowtie2_build\tasset_backed\treference-index-assets\tasset-set\t27\tdry_run\tmeta\toptional\tsupported\treference_preparation\tReference Preparation\treference_index_assets\tReference Index Assets\t\tfalse\tfalse\treference provenance and benchmark index availability\t"
            )
        }),
        "TSV must retain the governed reference-preparation row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "fastq:corpus-01-mini:fastq.estimate_library_complexity_prealign:sample-set:bijux_dna\tfastq.estimate_library_complexity_prealign\tbijux_dna\tcorpus-01\tcorpus-01-mini\tsample-set\t5\tsmoke\toptional\toptional\tsupported\tquality_profiling\tQuality Profiling\tpremerge_complexity\tPre-merge Complexity\tintegrity\tfalse\ttrue\tduplicate burden and pre-alignment library complexity evidence\t"
            )
        }),
        "TSV must retain the governed premerge-complexity row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats\tfastq.profile_reads\tseqkit_stats\tcorpus-01\tcorpus-01-mini\tsample-set\t11\tsmoke\tcore\tessential\tsupported\tquality_profiling\tQuality Profiling\tqc_signal_profiles\tQC Signal Profiles\tintegrity,composition\tfalse\ttrue\tbaseline read quality, composition, and aggregated QC evidence\t"
            )
        }),
        "TSV must retain the governed quality-profiling row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "fastq:corpus-03-amplicon-mini:fastq.infer_asvs:sample-set:dada2\tfastq.infer_asvs\tdada2\tcorpus-03\tcorpus-03-amplicon-mini\tsample-set\t22\tsmoke\tamplicon\texperimental\tsupported\tamplicon_interpretation\tAmplicon Interpretation\tamplicon_features\tAmplicon Feature Tables\tcomposition\tfalse\tfalse\tamplicon cleanup, feature inference, and abundance normalization outputs\t"
            )
        }),
        "TSV must retain the governed amplicon feature row"
    );
}
