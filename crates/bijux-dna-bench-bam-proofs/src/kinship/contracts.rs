use super::declarations::EXPECTED_METHOD;
use super::declarations::EXPECTED_STAGE_ID;
use super::observations::LocalKinshipSmokeCaseReport;
use super::observations::LocalKinshipSmokeReport;
use crate::downstream::observations::ParserObservation;
use crate::downstream::observations::ParserObservations;
use crate::downstream::observations::PipelineNode;
use crate::downstream::observations::PipelineObservation;
use anyhow::anyhow;
use anyhow::Result;
#[cfg(feature = "bam_downstream")]
pub(super) fn find_case<'a>(
    report: &'a LocalKinshipSmokeReport,
    sample_id: &str,
) -> Result<&'a LocalKinshipSmokeCaseReport> {
    report
        .cases
        .iter()
        .find(|case| case.sample_id == sample_id)
        .ok_or_else(|| anyhow!("missing bam.kinship local-smoke case `{sample_id}`"))
}

#[cfg(feature = "bam_downstream")]
pub(super) fn find_parser_smoke_row(report: &ParserObservations) -> Result<&ParserObservation> {
    report
        .rows
        .iter()
        .find(|row| {
            row.representative_stage_id == EXPECTED_STAGE_ID
                && row.representative_tool_id == EXPECTED_METHOD
        })
        .ok_or_else(|| anyhow!("missing real-output parser-smoke row for `bam.kinship` / `king`"))
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

pub(super) fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

pub(super) fn sorted_strings_from_slice(values: &[&str]) -> Vec<String> {
    let mut sorted = values.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    sorted.sort();
    sorted
}

pub(super) fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
