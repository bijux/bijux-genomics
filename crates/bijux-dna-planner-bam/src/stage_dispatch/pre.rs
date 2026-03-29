use anyhow::{anyhow, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_stage_contract::StagePlanV1;

use crate::api::StagePlanRequest;
use crate::params;
use crate::tool_adapters;

/// # Errors
/// Returns an error if the selected pre-alignment stage cannot be planned.
pub fn plan(stage: BamStage, request: &StagePlanRequest<'_>) -> Result<StagePlanV1> {
    match stage {
        BamStage::Align => {
            let r1 = request.r1.ok_or_else(|| anyhow!("align requires r1"))?;
            let reference = request
                .reference
                .ok_or_else(|| anyhow!("align requires reference"))?;
            let sample_id = request
                .sample_id
                .ok_or_else(|| anyhow!("align requires sample_id"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Align(params) = params else {
                return Err(anyhow!("align params mismatch"));
            };
            tool_adapters::stages_pre::align::plan(
                request.tool,
                r1,
                request.r2,
                reference,
                sample_id,
                &params,
                request.out_dir,
            )
        }
        BamStage::Validate => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("validate requires bam"))?;
            tool_adapters::stages_pre::validate::plan(
                request.tool,
                bam,
                request.bam_index,
                request.reference,
                request.out_dir,
            )
        }
        BamStage::QcPre => {
            let bam = request.bam.ok_or_else(|| anyhow!("qc_pre requires bam"))?;
            tool_adapters::stages_pre::qc_pre::plan(request.tool, bam, request.out_dir)
        }
        BamStage::MappingSummary => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("mapping_summary requires bam"))?;
            tool_adapters::stages_pre::mapping_summary::plan(request.tool, bam, request.out_dir)
        }
        BamStage::Filter => {
            let bam = request.bam.ok_or_else(|| anyhow!("filter requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Filter(params) = params else {
                return Err(anyhow!("filter params mismatch"));
            };
            let mut plan = tool_adapters::stages_pre::filter::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        BamStage::MapqFilter => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("mapq_filter requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::MapqFilter(params) = params else {
                return Err(anyhow!("mapq_filter params mismatch"));
            };
            let mut mapq_params = params;
            mapq_params.min_length = 0;
            tool_adapters::stages_pre::mapq_filter::plan(
                request.tool,
                bam,
                request.out_dir,
                &mapq_params,
            )
        }
        BamStage::LengthFilter => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("length_filter requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::LengthFilter(params) = params else {
                return Err(anyhow!("length_filter params mismatch"));
            };
            tool_adapters::stages_pre::length_filter::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        BamStage::OverlapCorrection => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("overlap_correction requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::OverlapCorrection(params) = params else {
                return Err(anyhow!("overlap_correction params mismatch"));
            };
            let mut plan = tool_adapters::stages_pre::overlap_correction::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        _ => Err(anyhow!(
            "stage {} is not handled by the pre-alignment dispatcher",
            stage.as_str()
        )),
    }
}
