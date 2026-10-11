//! Project native output files into the execution report layout.
use crate::filter::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::filter::model::FILTER_BREAKDOWN_JSON_NAME;
use crate::filter::model::FILTER_BREAKDOWN_TSV_NAME;
use crate::filter::model::FILTER_EXPLAIN_JSON_NAME;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::FilterStageOutputs;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct StageArtifacts {
    pub(crate) output_vcf: PathBuf,
    pub(crate) output_tbi: PathBuf,
    pub(crate) filter_breakdown_path: PathBuf,
    pub(crate) filter_breakdown_tsv_path: PathBuf,
    pub(crate) filter_explain_path: PathBuf,
}

pub(crate) fn materialize_outputs(
    stage_outputs: &FilterStageOutputs,
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
    let filter_breakdown_path = output_root.join(FILTER_BREAKDOWN_JSON_NAME);
    fs::copy(&stage_outputs.filter_breakdown_json, &filter_breakdown_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.filter_breakdown_json.display(),
            filter_breakdown_path.display()
        )
    })?;
    let filter_breakdown_tsv_path = output_root.join(FILTER_BREAKDOWN_TSV_NAME);
    fs::copy(&stage_outputs.filter_breakdown_tsv, &filter_breakdown_tsv_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.filter_breakdown_tsv.display(),
                filter_breakdown_tsv_path.display()
            )
        },
    )?;
    let filter_explain_path = output_root.join(FILTER_EXPLAIN_JSON_NAME);
    fs::copy(&stage_outputs.filter_explain_json, &filter_explain_path).with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.filter_explain_json.display(),
            filter_explain_path.display()
        )
    })?;

    Ok(StageArtifacts {
        output_vcf,
        output_tbi,
        filter_breakdown_path,
        filter_breakdown_tsv_path,
        filter_explain_path,
    })
}
