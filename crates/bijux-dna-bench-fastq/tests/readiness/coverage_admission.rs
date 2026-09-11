use anyhow::Result;
use bijux_dna_bench_fastq::readiness::inputs::{
    FastqScopeRow, FastqServingRow, FastqServingSnapshot,
};
use bijux_dna_bench_fastq::readiness::{active, adapters, fixtures, parsers};
fn serving(parser: &str) -> FastqServingSnapshot {
    FastqServingSnapshot {
        stage_count: 1,
        tool_count: 1,
        rows: vec![FastqServingRow {
            stage_id: "fastq.correct_errors".into(),
            tool_id: "rcorrector".into(),
            support_status: "governed_execution".into(),
            adapter_status: "runnable".into(),
            parser_status: parser.into(),
            corpus_status: "fixture:small".into(),
        }],
    }
}
fn scope() -> FastqScopeRow {
    FastqScopeRow {
        domain: "fastq".into(),
        stage_id: "fastq.correct_errors".into(),
        tool_id: "rcorrector".into(),
        corpus_id: "small".into(),
        asset_profile_id: "small".into(),
        adapter_id: "rcorrector".into(),
        parser_id: "parse_correct_errors_report".into(),
        schema_id: bijux_dna_domain_fastq::CORRECT_ERRORS_REPORT_SCHEMA_VERSION.into(),
    }
}
#[test]
fn unsupported_parser_status_never_establishes_readiness_or_parser_coverage() -> Result<()> {
    for status in [
        "",
        "foreign_parser",
        "not_normalized",
        "comparable",
        "benchmark_normalized",
        "parse_normalized",
    ] {
        let expected = matches!(status, "comparable" | "benchmark_normalized" | "parse_normalized");
        let coverage = adapters::collect_fastq_command_adapter_coverage_rows(serving(status))?;
        assert_eq!(
            coverage.2[0].benchmark_status == adapters::FastqBenchmarkStatus::BenchmarkReady,
            expected
        );
        assert_eq!(
            coverage.2[0].readiness_gap == adapters::FastqReadinessGapKind::Parser,
            !expected
        );
        let parsed = parsers::collect_fastq_parser_coverage_rows(coverage)?;
        assert_eq!(
            parsed.2[0].parser_coverage == parsers::FastqParserCoverageKind::Covered,
            expected
        );
    }
    Ok(())
}
#[test]
fn duplicate_or_inconsistent_sources_preserve_an_existing_report() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("retained.tsv");
    std::fs::write(&output, b"retained evidence")?;
    let mut duplicate = serving("comparable");
    duplicate.rows.push(duplicate.rows[0].clone());
    let mut wrong_count = serving("comparable");
    wrong_count.tool_count = 2;
    for input in [duplicate, wrong_count] {
        assert!(
            adapters::render_fastq_command_adapter_coverage(root.path(), &output, input).is_err()
        );
        assert_eq!(std::fs::read(&output)?, b"retained evidence");
    }
    Ok(())
}
#[test]
fn active_binding_without_coverage_refuses_before_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("absent/active.tsv");
    assert!(active::render_fastq_active_stage_tool_matrix(
        root.path(),
        &output,
        ((0, 0, vec![]), vec![scope()])
    )
    .is_err());
    assert!(!output.exists());
    assert!(!root.path().join("absent").exists());
    Ok(())
}
#[test]
fn governed_fixture_flows_through_typed_admission_and_stable_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let coverage = adapters::collect_fastq_command_adapter_coverage_rows(serving("comparable"))?;
    let active = active::collect_fastq_active_stage_tool_matrix_rows(coverage, vec![scope()])?;
    let output = root.path().join("fixture.tsv");
    let report = fixtures::render_fastq_parser_fixture_coverage(root.path(), &output, active.rows)?;
    assert_eq!(report.covered_row_count, 1);
    assert_eq!(report.missing_row_count, 0);
    assert_eq!(report.parser_fixture_coverage_percent.to_bits(), 100.0_f64.to_bits());
    assert_eq!(report.rows[0].tool_id, "rcorrector");
    assert_eq!(report.rows[0].parser_fixture_canonical_tool_id, "lighter");
    let text = std::fs::read_to_string(output)?;
    assert!(text.starts_with("stage_id\ttool_id\t"));
    assert!(text.ends_with('\n'));
    Ok(())
}
