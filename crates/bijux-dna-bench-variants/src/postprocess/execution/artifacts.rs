//! Project native output files into the execution report layout.
use crate::postprocess::model::DEFAULT_OUTPUT_CHECKSUMS_NAME;
use crate::postprocess::model::DEFAULT_OUTPUT_LOGS_NAME;
use crate::postprocess::model::DEFAULT_OUTPUT_MANIFEST_NAME;
use crate::postprocess::model::DEFAULT_OUTPUT_NORMALIZATION_NAME;
use crate::postprocess::model::DEFAULT_OUTPUT_VALIDATE_NAME;
use crate::postprocess::model::DEFAULT_OUTPUT_VCF_NAME;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::PostprocessStageOutputs;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct StageArtifacts {
    pub(crate) output_vcf: PathBuf,
    pub(crate) output_tbi: PathBuf,
    pub(crate) validate_outputs_path: PathBuf,
    pub(crate) final_manifest_path: PathBuf,
    pub(crate) normalization_contract_path: PathBuf,
    pub(crate) artifact_checksums_path: PathBuf,
    pub(crate) logs_path: PathBuf,
}

pub(crate) fn materialize_outputs(
    stage_outputs: &PostprocessStageOutputs,
    output_root: &Path,
) -> Result<StageArtifacts> {
    let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.merged_vcf, &output_vcf).with_context(|| {
        format!("copy {} to {}", stage_outputs.merged_vcf.display(), output_vcf.display())
    })?;
    let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    fs::copy(&stage_outputs.merged_tbi, &output_tbi).with_context(|| {
        format!("copy {} to {}", stage_outputs.merged_tbi.display(), output_tbi.display())
    })?;
    let validate_outputs_path = output_root.join(DEFAULT_OUTPUT_VALIDATE_NAME);
    fs::copy(&stage_outputs.validate_outputs_json, &validate_outputs_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.validate_outputs_json.display(),
            validate_outputs_path.display()
        )
    })?;
    let final_manifest_path = output_root.join(DEFAULT_OUTPUT_MANIFEST_NAME);
    fs::copy(&stage_outputs.final_manifest_json, &final_manifest_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.final_manifest_json.display(),
            final_manifest_path.display()
        )
    })?;
    let normalization_contract_path = output_root.join(DEFAULT_OUTPUT_NORMALIZATION_NAME);
    fs::copy(&stage_outputs.normalization_contract_json, &normalization_contract_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.normalization_contract_json.display(),
                normalization_contract_path.display()
            )
        })?;
    let artifact_checksums_path = output_root.join(DEFAULT_OUTPUT_CHECKSUMS_NAME);
    fs::copy(&stage_outputs.artifact_checksums_json, &artifact_checksums_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.artifact_checksums_json.display(),
                artifact_checksums_path.display()
            )
        },
    )?;
    let logs_path = output_root.join(DEFAULT_OUTPUT_LOGS_NAME);
    fs::copy(&stage_outputs.logs_txt, &logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), logs_path.display())
    })?;

    Ok(StageArtifacts {
        output_vcf,
        output_tbi,
        validate_outputs_path,
        final_manifest_path,
        normalization_contract_path,
        artifact_checksums_path,
        logs_path,
    })
}
