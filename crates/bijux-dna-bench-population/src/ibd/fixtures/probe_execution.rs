//! Owner: bijux-dna-bench-population
//! Execute the insufficient-overlap case and its productive ROH control.
use crate::ibd::model::PROBE_IBD_SEGMENT_MARKER_MINIMUM;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_ibd_stage;
use bijux_dna_stages_vcf::pipeline::run_roh_stage;
use bijux_dna_stages_vcf::pipeline::IbdStageParams;
use bijux_dna_stages_vcf::pipeline::RohStageParams;
use std::path::Path;

pub(crate) struct ProbeOutputs {
    pub(crate) ibd: bijux_dna_stages_vcf::pipeline::IbdStageOutputs,
    pub(crate) roh: bijux_dna_stages_vcf::pipeline::RohStageOutputs,
}
pub(crate) fn execute_probe(
    input_vcf_path: &Path,
    ibd_stage_root: &Path,
    roh_stage_root: &Path,
    tool_id: &str,
) -> Result<ProbeOutputs> {
    let ibd_outputs = run_ibd_stage(
        input_vcf_path,
        ibd_stage_root,
        &IbdStageParams {
            toolchain: tool_id.to_string(),
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 2,
            min_segment_cm: 1.0,
            min_markers_per_segment: PROBE_IBD_SEGMENT_MARKER_MINIMUM,
            ..IbdStageParams::default()
        },
    )
    .with_context(|| format!("run sparse-overlap IBD probe from {}", input_vcf_path.display()))?;
    let roh_outputs = run_roh_stage(
        input_vcf_path,
        roh_stage_root,
        &RohStageParams {
            toolchain: "plink2".to_string(),
            min_snp_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_segment_kb: 0,
            max_gap_bp: 10_000_000,
            ..RohStageParams::default()
        },
    )
    .with_context(|| format!("run sparse-overlap ROH probe from {}", input_vcf_path.display()))?;

    Ok(ProbeOutputs { ibd: ibd_outputs, roh: roh_outputs })
}
