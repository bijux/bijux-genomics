use anyhow::Result;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_stage_contract::StagePlanV1;

use crate::api::StagePlanRequest;

pub mod adna;
pub mod downstream;
pub mod pre;
pub mod post;

/// # Errors
/// Returns an error if the stage family planner cannot build a valid stage plan.
pub fn plan(stage: BamStage, request: &StagePlanRequest<'_>) -> Result<StagePlanV1> {
    match stage {
        BamStage::Align
        | BamStage::Validate
        | BamStage::QcPre
        | BamStage::MappingSummary
        | BamStage::Filter
        | BamStage::MapqFilter
        | BamStage::LengthFilter
        | BamStage::OverlapCorrection => pre::plan(stage, request),
        BamStage::Markdup
        | BamStage::DuplicationMetrics
        | BamStage::Complexity
        | BamStage::Coverage
        | BamStage::InsertSize
        | BamStage::GcBias
        | BamStage::EndogenousContent
        | BamStage::Recalibration => post::plan(stage, request),
        BamStage::Damage | BamStage::Authenticity | BamStage::Contamination | BamStage::Sex => {
            adna::plan(stage, request)
        }
        BamStage::BiasMitigation
        | BamStage::Haplogroups
        | BamStage::Genotyping
        | BamStage::Kinship => downstream::plan(stage, request),
    }
}
