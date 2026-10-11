use super::models::BindingKey;
use super::models::FastqValidateReadsReadyRow;
use super::models::FastqValidateReadsSmokeEvidence;
use crate::proofs::inputs::observations::CommandArgvObservation;
use crate::proofs::inputs::observations::ComparableMetricObservation;
use crate::proofs::inputs::observations::ExpectedResultObservation;
use crate::readiness::active::FastqActiveStageToolMatrixRow;
use crate::readiness::output_contract::FastqAdapterOutputContractRow;
use crate::readiness::parsers::FastqParserCoverageRow;
use crate::readiness::report_map::FastqReportMapRow;
use std::collections::BTreeMap;
pub(super) struct SourceEvidence {
    pub(super) active_matrix: Vec<FastqActiveStageToolMatrixRow>,
    pub(super) output_contract: Vec<FastqAdapterOutputContractRow>,
    pub(super) parser_coverage: Vec<FastqParserCoverageRow>,
    pub(super) comparable_metrics: Vec<ComparableMetricObservation>,
    pub(super) report_map: Vec<FastqReportMapRow>,
    pub(super) expected_results: Vec<ExpectedResultObservation>,
    pub(super) command_argv: Vec<CommandArgvObservation>,
    pub(super) smoke: FastqValidateReadsSmokeEvidence,
}
pub(super) struct AdmittedBindings {
    pub(super) active_rows: Vec<FastqActiveStageToolMatrixRow>,
    pub(super) output_by_binding: BTreeMap<BindingKey, FastqAdapterOutputContractRow>,
    pub(super) parser_by_binding: BTreeMap<BindingKey, FastqParserCoverageRow>,
    pub(super) expected_by_binding: BTreeMap<BindingKey, ExpectedResultObservation>,
    pub(super) command_by_binding: BTreeMap<BindingKey, CommandArgvObservation>,
    pub(super) report_map_by_binding: BTreeMap<BindingKey, FastqReportMapRow>,
    pub(super) comparable_row: ComparableMetricObservation,
    pub(super) smoke: FastqValidateReadsSmokeEvidence,
}
pub(super) struct EvaluatedRows {
    pub(super) rows: Vec<FastqValidateReadsReadyRow>,
    pub(super) smoke: FastqValidateReadsSmokeEvidence,
}
