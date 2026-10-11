//! Execute the native VCF stage with explicit scientific parameters.
use crate::postprocess::execution::inputs::PreparedInputs;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_corpus::panel_metadata::governed_vcf_panel_species_context;
use bijux_dna_stages_vcf::pipeline::run_postprocess_stage;
use bijux_dna_stages_vcf::pipeline::PostprocessStageOutputs;
use bijux_dna_stages_vcf::pipeline::PostprocessStageParams;

pub(crate) fn execute_stage(prepared: &PreparedInputs) -> Result<PostprocessStageOutputs> {
    let input_vcf = &prepared.input_vcf;
    let stage_root = &prepared.stage_root;
    let species_context = governed_vcf_panel_species_context();
    let stage_outputs = run_postprocess_stage(
        input_vcf,
        stage_root,
        &species_context,
        &PostprocessStageParams {
            species_id: species_context.species_id.clone(),
            build_id: species_context.build_id.clone(),
            per_chr_inputs: vec![],
            retain_info_fields: vec![],
            remove_info_fields: vec!["MQ".to_string()],
            compression_level: 6,
            compression_threads: 2,
            emit_bcf: false,
            normalize_indels: true,
            split_multiallelic: true,
            run_level_checksums_path: None,
        },
    )
    .with_context(|| format!("run governed VCF postprocess smoke from {}", input_vcf.display()))?;

    Ok(stage_outputs)
}
