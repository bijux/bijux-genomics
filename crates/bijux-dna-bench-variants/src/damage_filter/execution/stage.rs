//! Execute the native VCF stage with explicit scientific parameters.
use crate::damage_filter::execution::inputs::PreparedInputs;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_damage_filter_stage_with_policy;
use bijux_dna_stages_vcf::pipeline::DamageFilterOutputs;
use bijux_dna_stages_vcf::pipeline::DamageFilterPolicy;
use bijux_dna_stages_vcf::pipeline::DamageFilterStageParams;
use bijux_dna_stages_vcf::pipeline::DamageUdgRegime;

pub(crate) fn execute_stage(prepared: &PreparedInputs) -> Result<DamageFilterOutputs> {
    let input_vcf = &prepared.input_vcf;
    let stage_root = &prepared.stage_root;
    let params = DamageFilterStageParams {
        udg_regime: DamageUdgRegime::NonUdg,
        strict_regime: true,
        min_qual: 30.0,
        max_damage_ratio: 0.35,
    };
    let policy = DamageFilterPolicy::recommended(DamageUdgRegime::NonUdg);
    let stage_outputs =
        run_damage_filter_stage_with_policy(input_vcf, stage_root, &params, &policy).with_context(
            || format!("run governed VCF damage_filter smoke from {}", input_vcf.display()),
        )?;

    Ok(stage_outputs)
}
