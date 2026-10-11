//! Execute the native VCF stage with explicit scientific parameters.
use crate::filter::execution::inputs::PreparedInputs;
use crate::filter::model::GovernedVcfFilterSmokeContract;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_vcf::params::VcfFilterParams;
use bijux_dna_stages_vcf::pipeline::run_filter_stage_real;
use bijux_dna_stages_vcf::pipeline::FilterStageOutputs;

pub(crate) fn execute_stage(
    prepared: &PreparedInputs,
    contract: &GovernedVcfFilterSmokeContract,
) -> Result<FilterStageOutputs> {
    let input_vcf = &prepared.input_vcf;
    let stage_root = &prepared.stage_root;
    let stage_outputs = run_filter_stage_real(
        input_vcf,
        stage_root,
        &VcfFilterParams {
            sample_name: contract.sample_name.clone(),
            min_qual: 30.0,
            require_pass: false,
            normalize: true,
            require_bgzip_tabix: true,
            production_profile: false,
            ..VcfFilterParams::default()
        },
    )
    .with_context(|| format!("run governed VCF filter smoke from {}", input_vcf.display()))?;

    Ok(stage_outputs)
}
