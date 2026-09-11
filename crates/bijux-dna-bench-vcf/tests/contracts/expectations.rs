use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::serving::collect_vcf_tool_serving_map_rows,
};
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultsReport;

use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::expectations::{
    render_vcf_expected_benchmark_results, DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
    VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn render_vcf_expected_benchmark_results_tracks_benchmark_ready_rows() {
    let root = repo_root();
    let inputs = crate::support::inputs(&root);
    let observations = VcfAdapterObservations::new(&inputs, &root);
    let report = render_vcf_expected_benchmark_results(
        &observations,
        &PathBuf::from(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH),
    )
    .expect("render VCF expected benchmark results");

    assert_eq!(report.schema_version, VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION);
    assert_eq!(report.output_path, DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH);
    let expected_row_count = collect_vcf_tool_serving_map_rows(&inputs)
        .expect("collect VCF tool-serving rows")
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .count();
    assert_eq!(report.row_count, expected_row_count);
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
    assert_eq!(report.corpus_count, 1);
    assert_eq!(report.asset_profile_count, 6);
    assert_eq!(report.report_section_counts.get("damage_aware_filtering"), Some(&1));
    assert_eq!(report.report_section_counts.get("imputation"), Some(&8));
    assert_eq!(report.report_section_counts.get("likelihood_postprocess"), Some(&1));
    assert_eq!(report.report_section_counts.get("population_structure"), Some(&7));
    assert_eq!(report.report_section_counts.get("runs_of_homozygosity"), Some(&1));
    assert_eq!(report.report_section_counts.get("identity_by_descent"), Some(&3));
    assert_eq!(report.report_section_counts.get("demography"), Some(&1));
    assert_eq!(report.report_section_counts.get("phasing"), Some(&4));
    assert_eq!(report.report_section_counts.get("variant_calling"), Some(&6));
    assert_eq!(report.report_section_counts.get("quality_control"), Some(&5));
    assert_eq!(report.report_section_counts.get("normalization"), Some(&1));
    assert_eq!(report.report_section_counts.get("reference_panel_preparation"), Some(&1));

    assert!(report.rows.iter().all(|row| {
        row.domain == "vcf"
            && row.corpus_id == "vcf_production_regression"
            && !row.expected_outputs.is_empty()
            && !row.expected_metrics.is_empty()
            && !row.report_section.is_empty()
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.stats"
            && row.asset_profile_id == "vcf_cohort"
            && row.expected_outputs == vec!["stats_json".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "ti_tv")
            && row.report_section == "quality_control"
    }));
    assert_genotypes_expectations(&report);
    assert_population_expectations(&report);
    assert_panels_expectations(&report);
}

fn assert_genotypes_expectations(report: &VcfExpectedBenchmarkResultsReport) {
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.call"
            && row.asset_profile_id == "bam_bundle"
            && row.expected_outputs == vec!["called_vcf".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "variant_count")
            && row.report_section == "variant_calling"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.gl_propagation"
            && row.asset_profile_id == "vcf_single_sample"
            && row.expected_outputs == vec!["gl_propagated_vcf".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "lost_fields")
            && row.report_section == "likelihood_postprocess"
    }));
}

fn assert_population_expectations(report: &VcfExpectedBenchmarkResultsReport) {
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.admixture"
            && row.tool_id == "plink2"
            && row.asset_profile_id == "vcf_cohort"
            && row.expected_outputs == vec!["admixture_report".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "selected_k")
            && row.report_section == "population_structure"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.population_structure"
            && row.tool_id == "plink2"
            && row.asset_profile_id == "vcf_cohort"
            && row.expected_outputs == vec!["population_structure_report".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "sample_groups")
            && row.report_section == "population_structure"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.roh"
            && row.tool_id == "plink2"
            && row.asset_profile_id == "vcf_cohort"
            && row.expected_outputs == vec!["roh_report".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "segment_count")
            && row.report_section == "runs_of_homozygosity"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.pca"
            && row.tool_id == "plink2"
            && row.asset_profile_id == "vcf_cohort"
            && row.expected_outputs == vec!["pca_report".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "eigenvalues")
            && row.report_section == "population_structure"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.pca"
            && row.tool_id == "eigensoft"
            && row.asset_profile_id == "vcf_cohort"
            && row.expected_outputs == vec!["pca_report".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "eigenvalues")
            && row.report_section == "population_structure"
    }));
}

fn assert_panels_expectations(report: &VcfExpectedBenchmarkResultsReport) {
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.imputation_metrics"
            && row.tool_id == "beagle"
            && row.asset_profile_id == "vcf_cohort_with_panel"
            && row.expected_outputs == vec!["imputation_metrics_json".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "mean_info_score")
            && row.report_section == "imputation"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.impute"
            && row.tool_id == "beagle"
            && row.asset_profile_id == "vcf_cohort_with_panel"
            && row.expected_outputs == vec!["imputed_vcf".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "masked_truth_match_count")
            && row.report_section == "imputation"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.phasing"
            && row.tool_id == "shapeit5"
            && row.asset_profile_id == "vcf_cohort_with_panel"
            && row.expected_outputs == vec!["phased_vcf".to_string()]
            && row.expected_metrics.iter().any(|metric| metric == "phase_set_count")
            && row.report_section == "phasing"
    }));
}

fn refuse_reference(
    _: &std::path::Path,
    _: &std::path::Path,
) -> anyhow::Result<(PathBuf, PathBuf)> {
    anyhow::bail!("unexpected reference effect")
}

#[test]
fn malformed_parser_refuses_before_expected_result_publication() -> anyhow::Result<()> {
    let root = repo_root();
    let base = crate::support::inputs(&root);
    let mut bindings = base.bindings().to_vec();
    bindings[0].parser_id = "vcf.parser.unknown".to_owned();
    let inputs = bijux_dna_bench_vcf::inputs::VcfAdapterInputs::new(
        base.catalog().to_vec(),
        bindings,
        refuse_reference,
    )?;
    let observations = VcfAdapterObservations::new(&inputs, &root);
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("unpublished/expected.tsv");
    let error =
        render_vcf_expected_benchmark_results(&observations, &output).err().map(|e| e.to_string());
    assert!(error.is_some_and(|message| message.contains("unknown parser contract")));
    assert!(!output.parent().is_some_and(std::path::Path::exists));
    let output = directory.path().join("existing.tsv");
    std::fs::write(&output, b"retained expectation evidence")?;
    assert!(render_vcf_expected_benchmark_results(&observations, &output).is_err());
    assert_eq!(std::fs::read(&output)?, b"retained expectation evidence");
    Ok(())
}
