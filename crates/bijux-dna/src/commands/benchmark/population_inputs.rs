//! Acquire only declarations required by the selected population service.
use anyhow::Result;
use bijux_dna_bench_population::inputs::PopulationBenchmarkInputs;
use bijux_dna_core::id_catalog as stage;
use std::path::Path;

pub(super) fn supported(root: &Path) -> Result<PopulationBenchmarkInputs> {
    let rows = super::vcf_benchmark_bindings::collect_vcf_benchmark_binding_rows_at(root)?;
    PopulationBenchmarkInputs::new(
        rows.into_iter()
            .filter(|row| {
                [stage::VCF_PCA, stage::VCF_ADMIXTURE, stage::VCF_POPULATION_STRUCTURE]
                    .contains(&row.stage_id.as_str())
            })
            .collect(),
        Vec::new(),
        Vec::new(),
    )
}
pub(super) fn governed(_root: &Path) -> Result<PopulationBenchmarkInputs> {
    let rows = super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?;
    PopulationBenchmarkInputs::new(
        Vec::new(),
        rows.into_iter()
            .filter(|row| {
                [stage::VCF_ROH, stage::VCF_IBD, stage::VCF_DEMOGRAPHY]
                    .contains(&row.stage_id.as_str())
            })
            .collect(),
        Vec::new(),
    )
}
pub(super) fn ibd(root: &Path) -> Result<PopulationBenchmarkInputs> {
    let rows = super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?;
    let active = super::readiness::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows(root)?;
    PopulationBenchmarkInputs::new(
        Vec::new(),
        rows.into_iter()
            .filter(|row| {
                [stage::VCF_ROH, stage::VCF_IBD, stage::VCF_DEMOGRAPHY]
                    .contains(&row.stage_id.as_str())
            })
            .collect(),
        active
            .into_iter()
            .filter(|row| row.domain == "vcf" && row.stage_id == stage::VCF_IBD)
            .collect(),
    )
}
