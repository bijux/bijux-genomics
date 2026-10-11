//! Project native output files into the execution report layout.
use crate::damage_filter::model::DAMAGE_FILTER_COUNTS_NAME;
use crate::damage_filter::model::DAMAGE_FILTER_MANIFEST_NAME;
use crate::damage_filter::model::DAMAGE_FILTER_SUMMARY_NAME;
use crate::damage_filter::model::DAMAGE_FILTER_WARNINGS_NAME;
use crate::damage_filter::model::DEFAULT_OUTPUT_VCF_NAME;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::DamageFilterOutputs;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct StageArtifacts {
    pub(crate) output_vcf: PathBuf,
    pub(crate) output_tbi: PathBuf,
    pub(crate) summary_path: PathBuf,
    pub(crate) counts_path: PathBuf,
    pub(crate) warnings_path: PathBuf,
    pub(crate) damage_manifest_path: PathBuf,
}

pub(crate) fn materialize_outputs(
    stage_outputs: &DamageFilterOutputs,
    output_root: &Path,
) -> Result<StageArtifacts> {
    let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
    fs::copy(&stage_outputs.filtered_vcf, &output_vcf).with_context(|| {
        format!("copy {} to {}", stage_outputs.filtered_vcf.display(), output_vcf.display())
    })?;
    let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
    fs::copy(&stage_outputs.filtered_tbi, &output_tbi).with_context(|| {
        format!("copy {} to {}", stage_outputs.filtered_tbi.display(), output_tbi.display())
    })?;
    let summary_path = output_root.join(DAMAGE_FILTER_SUMMARY_NAME);
    fs::copy(&stage_outputs.damage_filter_summary_json, &summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.damage_filter_summary_json.display(),
            summary_path.display()
        )
    })?;
    let counts_path = output_root.join(DAMAGE_FILTER_COUNTS_NAME);
    fs::copy(&stage_outputs.damage_filter_counts_json, &counts_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.damage_filter_counts_json.display(),
            counts_path.display()
        )
    })?;
    let warnings_path = output_root.join(DAMAGE_FILTER_WARNINGS_NAME);
    fs::copy(&stage_outputs.warnings_json, &warnings_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.warnings_json.display(), warnings_path.display())
    })?;
    let damage_manifest_path = output_root.join(DAMAGE_FILTER_MANIFEST_NAME);
    fs::copy(&stage_outputs.damage_genotype_manifest_json, &damage_manifest_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.damage_genotype_manifest_json.display(),
                damage_manifest_path.display()
            )
        },
    )?;

    Ok(StageArtifacts {
        output_vcf,
        output_tbi,
        summary_path,
        counts_path,
        warnings_path,
        damage_manifest_path,
    })
}
