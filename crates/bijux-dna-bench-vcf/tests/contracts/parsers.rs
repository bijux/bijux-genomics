use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::{
        parsers::{
            coverage_status_label, render_vcf_parser_fixture_coverage,
            DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH, VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
        },
        serving::collect_vcf_tool_serving_map_rows,
    },
    inputs::VcfAdapterInputs,
};
use bijux_dna_bench_vcf_evidence::{
    model::{
        parsers::{VcfParserFixtureCoverageReport, VcfParserFixtureCoverageStatus},
        scope::VcfActiveStageToolMatrixRow,
    },
    snapshot::VcfStageReadinessBindingKey,
};
use std::path::{Path, PathBuf};
fn retained_rows(inputs: &VcfAdapterInputs, root: &Path) -> Vec<VcfActiveStageToolMatrixRow> {
    let observations = VcfAdapterObservations::new(inputs, root);
    let expected =
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations,
        )
        .expect("expected bindings");
    let active = expected
        .iter()
        .map(|row| VcfStageReadinessBindingKey {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
        })
        .collect::<Vec<_>>();
    bijux_dna_bench_vcf::evidence::scope::collect_vcf_active_stage_tool_matrix_rows_from_expected(
        inputs, root, &active, &expected,
    )
    .expect("retained scope")
}
#[test]
fn render_vcf_parser_fixture_coverage_reports_active_rows() {
    let root = crate::support::repo_root();
    let inputs = crate::support::inputs(&root);
    let retained = retained_rows(&inputs, &root);
    let report = render_vcf_parser_fixture_coverage(
        &inputs,
        &root,
        &retained,
        &PathBuf::from(DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH),
    )
    .expect("render VCF parser fixture coverage");

    assert_eq!(report.schema_version, VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION);
    assert_eq!(report.output_path, DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH);
    assert_eq!(report.stage_count, 20);
    assert_eq!(
        report.tool_count,
        report
            .rows
            .iter()
            .map(|row| row.tool_id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    let benchmark_ready_row_count = collect_vcf_tool_serving_map_rows(&inputs)
        .expect("collect VCF tool-serving rows")
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .count();
    assert_eq!(report.row_count, benchmark_ready_row_count);
    assert_eq!(report.covered_row_count, benchmark_ready_row_count);
    assert_eq!(report.missing_row_count, 0);
    assert!((report.parser_fixture_coverage_percent - 100.0).abs() < f64::EPSILON);
    assert_eq!(report.coverage_status_counts.get("covered"), Some(&benchmark_ready_row_count));
    assert!(report.rows.iter().all(|row| {
        row.schema_id.starts_with("bijux.schemas.bench.vcf-normalized-metrics.")
            && row.parser_fixture_schema_id.starts_with("bijux.vcf.")
            && row.expected_normalized_path.ends_with("expected.normalized.json")
            && row.raw_fixture_count > 0
            && !row.raw_fixture_paths.is_empty()
            && coverage_status_label(row.coverage_status) == "covered"
    }));
    assert_governed_parser_rows(&report);
}

#[test]
fn coverage_status_labels_are_stable() {
    assert_eq!(coverage_status_label(VcfParserFixtureCoverageStatus::Covered), "covered");
    assert_eq!(
        coverage_status_label(VcfParserFixtureCoverageStatus::MissingFixtureInventory),
        "missing_fixture_inventory"
    );
    assert_eq!(
        coverage_status_label(VcfParserFixtureCoverageStatus::MissingFixtureDirectory),
        "missing_fixture_directory"
    );
    assert_eq!(
        coverage_status_label(VcfParserFixtureCoverageStatus::MissingExpectedNormalizedJson),
        "missing_expected_normalized_json"
    );
    assert_eq!(
        coverage_status_label(VcfParserFixtureCoverageStatus::MissingRawFixtures),
        "missing_raw_fixtures"
    );
    assert_eq!(
        coverage_status_label(VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson),
        "invalid_expected_normalized_json"
    );
}

fn assert_governed_parser_rows(report: &VcfParserFixtureCoverageReport) {
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.admixture"
            && row.tool_id == "plink2"
            && row.parser_fixture_parser_id == "parse_plink2_admixture_metrics"
            && row.parser_fixture_schema_id == "bijux.vcf.admixture.v1"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.call"
            && row.tool_id == "bcftools"
            && row.parser_fixture_parser_id == "parse_bcftools_call_metrics"
            && row.parser_fixture_schema_id == "bijux.vcf.call.v1"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.roh"
            && row.tool_id == "plink2"
            && row.parser_fixture_parser_id == "parse_plink2_roh_segment_metrics"
            && row.parser_fixture_schema_id == "bijux.vcf.roh.v1"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.imputation_metrics"
            && row.tool_id == "beagle"
            && row.parser_fixture_parser_id == "parse_beagle_imputation_metrics"
            && row.expected_normalized_path.ends_with(
                "benchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.imputation_metrics/expected.normalized.json"
            )
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.postprocess"
            && row.tool_id == "bcftools"
            && row.parser_fixture_parser_id == "parse_bcftools_postprocess_metrics"
            && row.raw_fixture_paths.iter().any(|path| path.ends_with("raw.postprocess.vcf"))
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.pca"
            && row.tool_id == "plink2"
            && row.parser_fixture_parser_id == "parse_plink2_pca_metrics"
            && row.raw_fixture_count >= 3
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.phasing"
            && row.tool_id == "shapeit5"
            && row.coverage_status == VcfParserFixtureCoverageStatus::Covered
            && row.reason.contains("expected normalized JSON")
    }));
}
