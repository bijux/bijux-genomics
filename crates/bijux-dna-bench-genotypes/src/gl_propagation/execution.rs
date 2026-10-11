//! Likelihood fixture preparation, native execution and artifact materialization.
use crate::gl_propagation::fixtures::inputs::write_governed_gl_propagation_input_vcf;
use crate::gl_propagation::model::LikelihoodFieldSummary;
use crate::gl_propagation::model::DEFAULT_INPUT_VCF_NAME;
use crate::gl_propagation::model::DEFAULT_OUTPUT_BCF_NAME;
use crate::gl_propagation::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::gl_propagation::model::DEFAULT_OUTPUT_VCF_NAME;
use crate::gl_propagation::observations::variants::summarize_likelihood_fields;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_gl_propagation_stage;
use bijux_dna_stages_vcf::pipeline::GlPropagationOutputs;
use bijux_dna_stages_vcf::pipeline::GlPropagationStageParams;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct PreparedInputs {
    pub(crate) vcf: PathBuf,
    pub(crate) stage: PathBuf,
    pub(crate) summary: LikelihoodFieldSummary,
}
impl PreparedInputs {
    pub(crate) fn prepare(output_root: &Path) -> Result<Self> {
        let artifacts_root = output_root.join("artifacts");
        let input_root = artifacts_root.join("input");
        let stage_root = artifacts_root.join("stage");
        bijux_dna_infra::ensure_dir(&input_root)
            .with_context(|| format!("create {}", input_root.display()))?;
        bijux_dna_infra::ensure_dir(&stage_root)
            .with_context(|| format!("create {}", stage_root.display()))?;

        let input_vcf = input_root.join(DEFAULT_INPUT_VCF_NAME);
        write_governed_gl_propagation_input_vcf(&input_vcf)?;
        let summary = summarize_likelihood_fields(&input_vcf)?;

        Ok(Self { vcf: input_vcf, stage: stage_root, summary })
    }
    pub(crate) fn execute(&self) -> Result<GlPropagationOutputs> {
        let stage_outputs = run_gl_propagation_stage(
            &self.vcf,
            &self.stage,
            &GlPropagationStageParams::recommended(),
        )
        .with_context(|| {
            format!("run governed VCF gl_propagation smoke from {}", self.vcf.display())
        })?;

        Ok(stage_outputs)
    }
}
pub(crate) struct StageArtifacts {
    pub(crate) vcf: PathBuf,
    pub(crate) index: PathBuf,
    pub(crate) bcf: PathBuf,
    pub(crate) bcf_index: PathBuf,
    pub(crate) report: PathBuf,
}
impl StageArtifacts {
    pub(crate) fn materialize(
        stage_outputs: &GlPropagationOutputs,
        output_root: &Path,
    ) -> Result<Self> {
        let output_vcf = output_root.join(DEFAULT_OUTPUT_VCF_NAME);
        fs::copy(&stage_outputs.normalized_vcf, &output_vcf).with_context(|| {
            format!("copy {} to {}", stage_outputs.normalized_vcf.display(), output_vcf.display())
        })?;
        let output_tbi = PathBuf::from(format!("{}.tbi", output_vcf.display()));
        fs::copy(&stage_outputs.normalized_tbi, &output_tbi).with_context(|| {
            format!("copy {} to {}", stage_outputs.normalized_tbi.display(), output_tbi.display())
        })?;
        let bcf_path = output_root.join(DEFAULT_OUTPUT_BCF_NAME);
        let normalized_bcf = stage_outputs
            .normalized_bcf
            .as_ref()
            .ok_or_else(|| anyhow!("gl_propagation smoke expected normalized_bcf output"))?;
        fs::copy(normalized_bcf, &bcf_path).with_context(|| {
            format!("copy {} to {}", normalized_bcf.display(), bcf_path.display())
        })?;
        let bcf_index_path = PathBuf::from(format!("{}.csi", bcf_path.display()));
        let normalized_bcf_csi = stage_outputs
            .normalized_bcf_csi
            .as_ref()
            .ok_or_else(|| anyhow!("gl_propagation smoke expected normalized_bcf_csi output"))?;
        fs::copy(normalized_bcf_csi, &bcf_index_path).with_context(|| {
            format!("copy {} to {}", normalized_bcf_csi.display(), bcf_index_path.display())
        })?;
        let report_path = output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
        fs::copy(&stage_outputs.gl_propagation_report_json, &report_path).with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.gl_propagation_report_json.display(),
                report_path.display()
            )
        })?;

        Ok(Self {
            vcf: output_vcf,
            index: output_tbi,
            bcf: bcf_path,
            bcf_index: bcf_index_path,
            report: report_path,
        })
    }
}
