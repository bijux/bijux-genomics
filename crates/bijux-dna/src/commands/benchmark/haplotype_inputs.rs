//! Acquire governed declarations for the haplotype benchmark owner.
use anyhow::Result;
use bijux_dna_bench_haplotypes::inputs::HaplotypeBenchmarkInputs;
use bijux_dna_core::id_catalog as stage;
pub(super) fn governed() -> Result<HaplotypeBenchmarkInputs> {
    let rows = super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?;
    HaplotypeBenchmarkInputs::new(
        rows.into_iter()
            .filter(|row| {
                [
                    stage::VCF_PREPARE_REFERENCE_PANEL,
                    stage::VCF_PHASING,
                    stage::VCF_IMPUTE,
                    stage::VCF_IMPUTATION_METRICS,
                ]
                .contains(&row.stage_id.as_str())
            })
            .collect(),
        bijux_dna_stage_contract::canonical_tool_bindings()?,
    )
}
