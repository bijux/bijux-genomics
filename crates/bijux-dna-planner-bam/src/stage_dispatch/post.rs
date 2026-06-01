use anyhow::{anyhow, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_stage_contract::StagePlanV1;

use crate::api::StagePlanRequest;
use crate::params;
use crate::tool_adapters;

/// # Errors
/// Returns an error if the selected post-alignment stage cannot be planned.
pub fn plan(stage: BamStage, request: &StagePlanRequest<'_>) -> Result<StagePlanV1> {
    match stage {
        BamStage::Markdup => {
            let bam = request.bam.ok_or_else(|| anyhow!("markdup requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Markdup(params) = params else {
                return Err(anyhow!("markdup params mismatch"));
            };
            let mut plan = tool_adapters::stages_post::markdup::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        BamStage::DuplicationMetrics => {
            let bam = request.bam.ok_or_else(|| anyhow!("duplication_metrics requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::DuplicationMetrics(params) = params else {
                return Err(anyhow!("duplication_metrics params mismatch"));
            };
            let mut plan = tool_adapters::stages_post::duplication_metrics::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        BamStage::Complexity => {
            let bam = request.bam.ok_or_else(|| anyhow!("complexity requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Complexity(params) = params else {
                return Err(anyhow!("complexity params mismatch"));
            };
            tool_adapters::stages_post::complexity::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        BamStage::Coverage => {
            let bam = request.bam.ok_or_else(|| anyhow!("coverage requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Coverage(params) = params else {
                return Err(anyhow!("coverage params mismatch"));
            };
            tool_adapters::stages_post::coverage::plan(request.tool, bam, request.out_dir, &params)
        }
        BamStage::InsertSize => {
            let bam = request.bam.ok_or_else(|| anyhow!("insert_size requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::InsertSize(params) = params else {
                return Err(anyhow!("insert_size params mismatch"));
            };
            tool_adapters::stages_post::insert_size::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        BamStage::GcBias => {
            let bam = request.bam.ok_or_else(|| anyhow!("gc_bias requires bam"))?;
            let reference =
                request.reference.ok_or_else(|| anyhow!("gc_bias requires reference"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::GcBias(params) = params else {
                return Err(anyhow!("gc_bias params mismatch"));
            };
            tool_adapters::stages_post::gc_bias::plan(
                request.tool,
                bam,
                reference,
                request.out_dir,
                &params,
            )
        }
        BamStage::EndogenousContent => {
            let bam = request.bam.ok_or_else(|| anyhow!("endogenous_content requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::EndogenousContent(params) = params else {
                return Err(anyhow!("endogenous_content params mismatch"));
            };
            let mut plan = tool_adapters::stages_post::endogenous_content::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        BamStage::Recalibration => {
            let bam = request.bam.ok_or_else(|| anyhow!("recalibration requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let BamEffectiveParams::Recalibration(params) = params else {
                return Err(anyhow!("recalibration params mismatch"));
            };
            tool_adapters::stages_post::recalibration::plan(
                request.tool,
                bam,
                request.reference,
                request.out_dir,
                &params,
            )
        }
        _ => {
            Err(anyhow!("stage {} is not handled by the post-alignment dispatcher", stage.as_str()))
        }
    }
}
