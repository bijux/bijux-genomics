#[cfg(feature = "bam_downstream")]
#[test]
fn render_adapter_missing_input_tests_reports_governed_probe_rows() {
    use std::path::PathBuf;

    use super::adapter_missing_input_tests::{
        render_adapter_missing_input_tests, DEFAULT_ADAPTER_MISSING_INPUT_TESTS_PATH,
    };

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root");
    let report = render_adapter_missing_input_tests(
        &root,
        PathBuf::from(DEFAULT_ADAPTER_MISSING_INPUT_TESTS_PATH),
    )
    .expect("render adapter missing-input tests");

    assert_eq!(report.schema_version, "bijux.bench.readiness.adapter_missing_input_tests.v1");
    assert_eq!(report.output_path, "benchmarks/readiness/adapter-missing-input-tests.json");
    assert_eq!(report.row_count, 33);
    assert_eq!(report.passed_row_count, 33);
    assert_eq!(report.failed_row_count, 0);
    assert_eq!(report.missing_input_class_counts.get("fastq").copied(), Some(7));
    assert_eq!(report.missing_input_class_counts.get("bam").copied(), Some(6));
    assert_eq!(report.missing_input_class_counts.get("reference").copied(), Some(16));
    assert_eq!(report.missing_input_class_counts.get("database").copied(), Some(4));
    assert!(report.rows.iter().all(|row| row.passed));

    let taxonomy_row = report
        .rows
        .iter()
        .find(|row| {
            row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.missing_input_role == "database_root"
        })
        .expect("taxonomy missing database_root row");
    assert!(taxonomy_row.observed_error.contains("taxonomy database root is missing"));

    let contamination_row = report
        .rows
        .iter()
        .find(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "verifybamid2"
                && row.missing_input_role == "reference_panel"
        })
        .expect("contamination missing reference_panel row");
    assert!(contamination_row.observed_error.contains("reference panel is missing"));

    let recalibration_row = report
        .rows
        .iter()
        .find(|row| {
            row.stage_id == "bam.recalibration"
                && row.tool_id == "gatk"
                && row.missing_input_role == "known_sites"
        })
        .expect("recalibration missing known_sites row");
    assert!(recalibration_row.observed_error.contains("known-sites fixture is missing"));
}
