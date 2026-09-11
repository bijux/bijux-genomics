use super::models::FastqValidateReadsSmokeEvidence;
use crate::proofs::inputs::observations::CommandArgvObservation as RenderedCommandArgvRow;
use crate::proofs::inputs::observations::ComparableMetricObservation as FastqComparableMetricsRow;
use crate::proofs::inputs::observations::ExpectedResultObservation as ExpectedBenchmarkResultRow;
use crate::readiness::active::FastqActiveStageToolMatrixRow;
use crate::readiness::output_contract::FastqAdapterOutputContractRow;
use crate::readiness::parsers::FastqParserCoverageRow;
use crate::readiness::report_map::FastqReportMapRow;
pub(super) struct RowInputs<'a> {
    pub(super) active_row: FastqActiveStageToolMatrixRow,
    pub(super) output_row: &'a FastqAdapterOutputContractRow,
    pub(super) parser_row: &'a FastqParserCoverageRow,
    pub(super) expected_row: &'a ExpectedBenchmarkResultRow,
    pub(super) command_row: &'a RenderedCommandArgvRow,
    pub(super) comparable_row: &'a FastqComparableMetricsRow,
    pub(super) report_map_row: &'a FastqReportMapRow,
    pub(super) smoke: &'a FastqValidateReadsSmokeEvidence,
}
#[allow(clippy::struct_excessive_bools)]
pub(super) struct RowAssessment {
    pub(super) coverage_status: String,
    pub(super) validation_report_fields: Vec<String>,
    pub(super) validated_reads_manifest_fields: Vec<String>,
    pub(super) command_ready: bool,
    pub(super) output_ready: bool,
    pub(super) parser_ready: bool,
    pub(super) comparable_metrics_ready: bool,
    pub(super) expected_result_ready: bool,
    pub(super) report_ready: bool,
    pub(super) missing_surfaces: Vec<String>,
}
