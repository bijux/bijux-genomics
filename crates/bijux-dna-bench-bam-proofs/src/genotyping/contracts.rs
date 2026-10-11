use super::declarations::EXPECTED_VCF_QC_TOOL_ID;
use super::declarations::EXPECTED_VCF_STATS_TOOL_ID;
use super::observations::LocalGenotypingArtifact;
use crate::downstream::observations::PipelineNode;
use crate::downstream::observations::PipelineObservation;
use crate::downstream::observations::PublishedVariantCounts;
use crate::downstream::observations::PublishedVariantMissingness;
use crate::downstream::observations::VariantCounts;
use crate::downstream::observations::VariantMissingness;
use anyhow::anyhow;
use anyhow::Result;
#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::{VCF_QC, VCF_STATS};
#[cfg(feature = "bam_downstream")]
#[cfg(feature = "bam_downstream")]
pub(super) fn find_vcf_stats_row(report: &PublishedVariantCounts) -> Result<&VariantCounts> {
    report
        .rows
        .iter()
        .find(|row| row.stage_id == VCF_STATS && row.tool_id == EXPECTED_VCF_STATS_TOOL_ID)
        .ok_or_else(|| anyhow!("missing vcf.stats readiness row for `bcftools`"))
}

#[cfg(feature = "bam_downstream")]
pub(super) fn find_vcf_qc_row(report: &PublishedVariantMissingness) -> Result<&VariantMissingness> {
    report
        .rows
        .iter()
        .find(|row| row.stage_id == VCF_QC && row.tool_id == EXPECTED_VCF_QC_TOOL_ID)
        .ok_or_else(|| anyhow!("missing vcf.qc readiness row for `bcftools`"))
}

#[cfg(feature = "bam_downstream")]
pub(super) fn find_pipeline_node<'a>(
    report: &'a PipelineObservation,
    stage_id: &str,
) -> Result<&'a PipelineNode> {
    report
        .nodes
        .iter()
        .find(|node| node.stage_id == stage_id)
        .ok_or_else(|| anyhow!("missing `{stage_id}` node in pipeline `{}`", report.pipeline_id))
}

#[cfg(feature = "bam_downstream")]
pub(super) fn find_plan_output(outputs: &[LocalGenotypingArtifact], name: &str) -> Result<String> {
    outputs
        .iter()
        .find(|artifact| artifact.name == name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("missing bam.genotyping local-plan output `{name}`"))
}

#[cfg(feature = "bam_downstream")]
pub(super) fn declared_output_names(outputs: &[LocalGenotypingArtifact]) -> Vec<String> {
    let mut names = outputs.iter().map(|artifact| artifact.name.clone()).collect::<Vec<_>>();
    names.sort();
    names
}

pub(super) fn sorted_output_names(output_ids: &[&str]) -> Vec<String> {
    let mut names = output_ids.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    names.sort();
    names
}

pub(super) fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
