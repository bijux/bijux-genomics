//! Typed phasing artifact projection from native stage outputs.
use crate::phasing::model::DEFAULT_OUTPUT_LOGS_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_MANIFEST_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_QC_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_SWITCH_PROXY_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_VCF_NAME;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::PhasingStageOutputs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct PhasingArtifacts {
    pub(crate) output_vcf: PathBuf,
    pub(crate) output_tbi: PathBuf,
    pub(crate) phasing_qc: PathBuf,
    pub(crate) phasing_manifest: PathBuf,
    pub(crate) phase_block_stats: PathBuf,
    pub(crate) switch_error_proxy: PathBuf,
    pub(crate) logs: PathBuf,
}

pub(crate) fn materialize_outputs(
    stage_outputs: &PhasingStageOutputs,
    output_root: &Path,
) -> Result<PhasingArtifacts> {
    let output_vcf_path = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.phased_vcf, &output_vcf_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.phased_vcf.display(), output_vcf_path.display())
    })?;
    let output_tbi_path = PathBuf::from(format!("{}.tbi", output_vcf_path.display()));
    bijux_dna_infra::copy_file(&stage_outputs.phased_tbi, &output_tbi_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.phased_tbi.display(), output_tbi_path.display())
    })?;
    let phasing_qc_path = output_root.join(DEFAULT_OUTPUT_QC_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.phasing_qc_json, &phasing_qc_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.phasing_qc_json.display(),
                phasing_qc_path.display()
            )
        },
    )?;
    let phasing_manifest_path = output_root.join(DEFAULT_OUTPUT_MANIFEST_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.phasing_manifest_json, &phasing_manifest_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.phasing_manifest_json.display(),
                phasing_manifest_path.display()
            )
        })?;
    let phase_block_stats_path = output_root.join(DEFAULT_OUTPUT_PHASE_BLOCK_STATS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.phase_block_stats_tsv, &phase_block_stats_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.phase_block_stats_tsv.display(),
                phase_block_stats_path.display()
            )
        })?;
    let switch_error_proxy_path = output_root.join(DEFAULT_OUTPUT_SWITCH_PROXY_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.switch_error_proxy_tsv, &switch_error_proxy_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.switch_error_proxy_tsv.display(),
                switch_error_proxy_path.display()
            )
        })?;
    let logs_path = output_root.join(DEFAULT_OUTPUT_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), logs_path.display())
    })?;

    Ok(PhasingArtifacts {
        output_vcf: output_vcf_path,
        output_tbi: output_tbi_path,
        phasing_qc: phasing_qc_path,
        phasing_manifest: phasing_manifest_path,
        phase_block_stats: phase_block_stats_path,
        switch_error_proxy: switch_error_proxy_path,
        logs: logs_path,
    })
}
