use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::serving::{
    render_vcf_tool_serving_map, VcfToolServingMapReport, DEFAULT_VCF_TOOL_SERVING_MAP_PATH,
    VCF_TOOL_SERVING_MAP_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_tool_serving_map_tracks_owned_matrix_rows() {
    let root = repo_root();
    let report = render_vcf_tool_serving_map(
        &crate::support::inputs(&root),
        &root,
        &PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH),
    )
    .expect("render VCF tool serving map");

    assert_eq!(report.schema_version, VCF_TOOL_SERVING_MAP_SCHEMA_VERSION);
    assert_eq!(report.domain, "vcf");
    assert_eq!(report.row_count, crate::support::inputs(&root).bindings().len());
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
    assert_eq!(report.benchmark_ready_row_count, report.row_count);
    assert_eq!(report.not_benchmark_ready_row_count, 0);
    assert_genotype_contracts(&report);
    assert_panel_contracts(&report);
    assert_population_contracts(&report);
}

fn assert_genotype_contracts(report: &VcfToolServingMapReport) {
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "bcftools"
            && row.stage_id == "vcf.call"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "bcftools"
            && row.stage_id == "vcf.qc"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "plink"
            && row.stage_id == "vcf.qc"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
}

fn assert_panel_contracts(report: &VcfToolServingMapReport) {
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "bcftools"
            && row.stage_id == "vcf.prepare_reference_panel"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "shapeit5"
            && row.stage_id == "vcf.phasing"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "beagle"
            && row.stage_id == "vcf.imputation_metrics"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "beagle"
            && row.stage_id == "vcf.impute"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
}

fn assert_population_contracts(report: &VcfToolServingMapReport) {
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "plink2"
            && row.stage_id == "vcf.admixture"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "plink2"
            && row.stage_id == "vcf.population_structure"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "plink2"
            && row.stage_id == "vcf.pca"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "eigensoft"
            && row.stage_id == "vcf.pca"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "assigned"
            && row.benchmark_status == "benchmark_ready"
    }));
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "ibdne"
            && row.stage_id == "vcf.demography"
            && row.support_status == "supported"
            && row.adapter_status == "runnable"
            && row.parser_status == "parse_normalized"
            && row.corpus_status == "fixture:vcf_production_regression"
            && row.asset_status == "not_required"
            && row.benchmark_status == "benchmark_ready"
    }));
}

fn refuse_materialization(
    _source: &std::path::Path,
    _destination: &std::path::Path,
) -> anyhow::Result<(PathBuf, PathBuf)> {
    anyhow::bail!("serving observations must not materialize reference inputs")
}

#[test]
fn serving_projection_uses_admitted_declarations_without_fixture_effects() -> anyhow::Result<()> {
    let base = crate::support::inputs(&repo_root());
    let inputs = bijux_dna_bench_vcf::inputs::VcfAdapterInputs::new(
        base.catalog().to_vec(),
        base.bindings().to_vec(),
        refuse_materialization,
    )?;
    let rows = bijux_dna_bench_vcf::evidence::serving::collect_vcf_tool_serving_map_rows(&inputs)?;
    assert_eq!(rows.len(), inputs.bindings().len());
    assert!(rows.iter().all(|row| row.benchmark_status == "benchmark_ready"));
    Ok(())
}

#[test]
fn unknown_parser_contract_refuses_before_publication() -> anyhow::Result<()> {
    let root = repo_root();
    let base = crate::support::inputs(&root);
    let mut bindings = base.bindings().to_vec();
    bindings[0].parser_id = "vcf.parser.unknown".into();
    let inputs = bijux_dna_bench_vcf::inputs::VcfAdapterInputs::new(
        base.catalog().to_vec(),
        bindings,
        refuse_materialization,
    )?;
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("unpublished/serving.tsv");
    let error =
        render_vcf_tool_serving_map(&inputs, &root, &output).err().map(|error| error.to_string());
    assert!(error.is_some_and(|error| error.contains("unknown parser contract")));
    assert!(!output.exists());
    assert!(!directory.path().join("unpublished").exists());
    let existing = directory.path().join("existing.tsv");
    std::fs::write(&existing, b"retained evidence")?;
    assert!(render_vcf_tool_serving_map(&inputs, &root, &existing).is_err());
    assert_eq!(std::fs::read(&existing)?, b"retained evidence");
    Ok(())
}
