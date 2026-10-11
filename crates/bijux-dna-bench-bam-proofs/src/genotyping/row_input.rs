use super::observations::LocalGenotypingPlan;
use crate::downstream::observations::ParserObservation;
use crate::downstream::observations::PipelineNode;
use crate::downstream::observations::PipelineObservation;
use crate::downstream::observations::PublishedVariantCounts;
use crate::downstream::observations::PublishedVariantMissingness;
use crate::downstream::observations::VariantCounts;
use crate::downstream::observations::VariantMissingness;
use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping::BamRecalibrationGenotypingReadyRow;
use std::path::Path;
pub(super) struct RowInputs<'a> {
    pub repo_root: &'a Path,
    pub local_plan_path: &'a Path,
    pub readiness_row: &'a BamRecalibrationGenotypingReadyRow,
    pub local_plan: &'a LocalGenotypingPlan,
    pub parser_row: &'a ParserObservation,
    pub bam_pipeline: &'a PipelineObservation,
    pub bam_pipeline_node: &'a PipelineNode,
    pub cross_pipeline: &'a PipelineObservation,
    pub cross_pipeline_genotyping_node: &'a PipelineNode,
    pub cross_pipeline_vcf_filter_node: &'a PipelineNode,
    pub vcf_stats: &'a PublishedVariantCounts,
    pub stats_row: &'a VariantCounts,
    pub vcf_qc: &'a PublishedVariantMissingness,
    pub qc_row: &'a VariantMissingness,
}
