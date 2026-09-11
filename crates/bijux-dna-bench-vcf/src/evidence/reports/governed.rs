use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::model::reports::VcfReportMapRow;
pub(super) fn require_governed_mappings(rows: &[VcfReportMapRow]) -> Result<()> {
    require_row_mapping(
        rows,
        "vcf.call",
        "bcftools",
        "variant_calling",
        "variant_calling_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.damage_filter",
        "bcftools",
        "damage_aware_filtering",
        "damage_filtering_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.filter",
        "bcftools",
        "quality_control",
        "quality_control_metrics",
    )?;
    require_row_mapping(rows, "vcf.qc", "bcftools", "quality_control", "quality_control_metrics")?;
    require_row_mapping(
        rows,
        "vcf.gl_propagation",
        "bcftools",
        "likelihood_postprocess",
        "likelihood_postprocess_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.postprocess",
        "bcftools",
        "normalization",
        "normalization_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.prepare_reference_panel",
        "bcftools",
        "reference_panel_preparation",
        "reference_panel_readiness",
    )?;
    require_row_mapping(rows, "vcf.impute", "beagle", "imputation", "imputation_metrics")?;
    require_row_mapping(
        rows,
        "vcf.admixture",
        "plink2",
        "population_structure",
        "population_structure_metrics",
    )?;
    require_row_mapping(rows, "vcf.roh", "plink2", "runs_of_homozygosity", "roh_metrics")?;
    require_row_mapping(
        rows,
        "vcf.population_structure",
        "plink2",
        "population_structure",
        "population_structure_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.pca",
        "plink2",
        "population_structure",
        "population_structure_metrics",
    )?;
    require_row_mapping(
        rows,
        "vcf.pca",
        "eigensoft",
        "population_structure",
        "population_structure_metrics",
    )?;

    Ok(())
}
fn require_row_mapping(
    rows: &[VcfReportMapRow],
    stage_id: &str,
    tool_id: &str,
    section_id: &str,
    summary_table: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| anyhow!("VCF report map is missing `{stage_id}` / `{tool_id}`"))?;
    if row.section_id != section_id || row.summary_table != summary_table {
        return Err(anyhow!(
            "VCF report map row `{stage_id}` / `{tool_id}` drifted from `{section_id}` / `{summary_table}`"
        ));
    }
    Ok(())
}
