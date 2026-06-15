#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_parser_fixture_coverage_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-parser-fixture-coverage"])
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
    assert_eq!(
        rendered_path.trim(),
        "benchmarks/readiness/fastq/fastq-parser-fixture-coverage.tsv"
    );

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ parser fixture coverage TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_reference_kind\tparser_fixture_reference\tparser_fixture_surface\tparser_fixture_canonical_tool_id\tcoverage_status\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 69);
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "fastq.trim_reads\ttrimmomatic\tcorpus-01-mini\tcorpus_only\tfastq.adapter.trim_reads\tfastq.parser.trim_reads\tfastq_trim_reads_v2\tparse_trim_reads_report\tbijux.fastq.trim_reads.report.v2\tfixture_case\tfastq.trim_reads.report_json\treport_json\tfastp\tcovered\tactive row `fastq.trim_reads` / `trimmomatic` is governed by FASTQ parser fixture case `fastq.trim_reads.report_json` using parser `parse_trim_reads_report` over canonical `fastp` report_json data"
        )
    }));
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "fastq.detect_duplicates_premerge\tbijux_dna\tcorpus-01-mini\tcorpus_only\tfastq.adapter.detect_duplicates_premerge\tfastq.parser.detect_duplicates_premerge\tfastq_detect_duplicates_premerge_v1\tparse_detect_duplicates_premerge_report\tbijux.fastq.detect_duplicates_premerge.report.v1\tfixture_case\tfastq.detect_duplicates_premerge.report_json\treport_json\tbijux_dna\tcovered\tactive row `fastq.detect_duplicates_premerge` / `bijux_dna` is governed by FASTQ parser fixture case `fastq.detect_duplicates_premerge.report_json` using parser `parse_detect_duplicates_premerge_report` over canonical `bijux_dna` report_json data"
        )
    }));
    assert!(rows.iter().any(|row| {
        row.starts_with(
            "fastq.screen_taxonomy\tkraken2\tcorpus-02-edna-mini\tdatabase_artifact_id+taxonomy_database_root\tfastq.adapter.screen_taxonomy\tfastq.parser.screen_taxonomy\tfastq_screen_taxonomy_v1\tparse_screen_taxonomy_report\tbijux.fastq.screen_taxonomy.report.v2\tfixture_case\tfastq.screen_taxonomy.classification_report_json\tclassification_report_json\tkraken2\tcovered\tactive row `fastq.screen_taxonomy` / `kraken2` is governed by FASTQ parser fixture case `fastq.screen_taxonomy.classification_report_json` using parser `parse_screen_taxonomy_report` over canonical `kraken2` classification_report_json data"
        )
    }));
}
