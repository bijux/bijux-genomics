use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::reports::{
    render_vcf_report_map, DEFAULT_VCF_REPORT_MAP_PATH, VCF_REPORT_MAP_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn render_vcf_report_map_reports_expected_result_sections() {
    let root = repo_root();
    let inputs = crate::support::inputs(&root);
    let observations =
        bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(&inputs, &root);
    let report = render_vcf_report_map(&observations, &PathBuf::from(DEFAULT_VCF_REPORT_MAP_PATH))
        .expect("render VCF report map");

    assert_eq!(report.schema_version, VCF_REPORT_MAP_SCHEMA_VERSION);
    assert_eq!(report.output_path, DEFAULT_VCF_REPORT_MAP_PATH);
    assert_eq!(
        report.row_count,
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations
        )
        .expect("collect VCF expected benchmark results")
        .len()
    );
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
    assert_eq!(report.section_count, 12);
    assert_eq!(report.summary_table_count, 12);
    assert_eq!(report.section_counts.get("variant_calling"), Some(&6));
    assert_eq!(report.section_counts.get("quality_control"), Some(&5));
    assert_eq!(report.section_counts.get("imputation"), Some(&8));
    assert_eq!(report.section_counts.get("population_structure"), Some(&7));
    assert_eq!(report.section_counts.get("runs_of_homozygosity"), Some(&1));
    assert_eq!(report.section_counts.get("identity_by_descent"), Some(&3));
    assert_eq!(report.section_counts.get("demography"), Some(&1));
    assert_eq!(report.section_counts.get("normalization"), Some(&1));
    assert_eq!(report.section_counts.get("reference_panel_preparation"), Some(&1));

    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.call"
            && row.tool_id == "bcftools"
            && row.section_id == "variant_calling"
            && row.summary_table == "variant_calling_metrics"
            && row.metric_columns
                == vec!["variant_count", "snp_count", "indel_count", "sample_count"]
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.gl_propagation"
            && row.section_id == "likelihood_postprocess"
            && row.summary_table == "likelihood_postprocess_metrics"
            && row.failure_columns.iter().any(|value| value == "observed_error")
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.admixture"
            && row.tool_id == "plink2"
            && row.section_id == "population_structure"
            && row.summary_table == "population_structure_metrics"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.population_structure"
            && row.tool_id == "plink2"
            && row.section_id == "population_structure"
            && row.summary_table == "population_structure_metrics"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.pca"
            && row.tool_id == "plink2"
            && row.section_id == "population_structure"
            && row.summary_table == "population_structure_metrics"
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.pca"
            && row.tool_id == "eigensoft"
            && row.section_id == "population_structure"
            && row.summary_table == "population_structure_metrics"
    }));
}

#[test]
fn report_placement_refuses_foreign_or_incomplete_expectations() -> anyhow::Result<()> {
    use bijux_dna_bench_vcf::evidence::{
        expectations::collect_vcf_expected_benchmark_result_rows,
        reports::collect_vcf_report_map_rows_from_expected,
    };
    let root = repo_root();
    let inputs = crate::support::inputs(&root);
    let observations =
        bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(&inputs, &root);
    let expected = collect_vcf_expected_benchmark_result_rows(&observations)?;
    let mut omitted = expected.clone();
    omitted.pop();
    assert!(collect_vcf_report_map_rows_from_expected(&inputs, omitted).is_err());
    let mut duplicate = expected.clone();
    duplicate.push(expected[0].clone());
    assert!(collect_vcf_report_map_rows_from_expected(&inputs, duplicate).is_err());
    for field in ["corpus", "asset", "metrics", "outputs", "section"] {
        let mut foreign = expected.clone();
        match field {
            "corpus" => foreign[0].corpus_id = "foreign-corpus".to_owned(),
            "asset" => foreign[0].asset_profile_id = "foreign-asset".to_owned(),
            "metrics" => foreign[0].expected_metrics = vec!["foreign-metric".to_owned()],
            "outputs" => foreign[0].expected_outputs = vec!["foreign-output".to_owned()],
            _ => foreign[0].report_section = "foreign-section".to_owned(),
        }
        assert!(collect_vcf_report_map_rows_from_expected(&inputs, foreign).is_err());
    }
    Ok(())
}

#[test]
fn malformed_parser_preserves_existing_report_map() -> anyhow::Result<()> {
    let root = repo_root();
    let base = crate::support::inputs(&root);
    let mut bindings = base.bindings().to_vec();
    bindings[0].parser_id = "vcf.parser.unknown".to_owned();
    let inputs = bijux_dna_bench_vcf::inputs::VcfAdapterInputs::new(
        base.catalog().to_vec(),
        bindings,
        |_, _| anyhow::bail!("unexpected reference materialization"),
    )?;
    let observations =
        bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(&inputs, &root);
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("existing.tsv");
    std::fs::write(&output, b"retained report evidence")?;
    assert!(render_vcf_report_map(&observations, &output).is_err());
    assert_eq!(std::fs::read(&output)?, b"retained report evidence");
    let absent = directory.path().join("unpublished/report.tsv");
    assert!(render_vcf_report_map(&observations, &absent).is_err());
    assert!(!absent.parent().is_some_and(std::path::Path::exists));
    Ok(())
}
