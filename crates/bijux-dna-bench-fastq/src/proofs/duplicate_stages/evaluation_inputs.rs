use super::models::DuplicateStageSchemaContract;
use super::models::DuplicateStageSpec;
use crate::proofs::inputs::observations::ExpectedResultObservation as ExpectedBenchmarkResultRow;
use crate::readiness::active::FastqActiveStageToolMatrixRow;
use crate::readiness::adapters::FastqCommandAdapterCoverageRow;
use crate::readiness::output_contract::FastqAdapterOutputContractRow;
use crate::readiness::parsers::FastqParserCoverageRow;
use crate::readiness::report_map::FastqReportMapRow;
pub(super) struct RowInputs<'a> {
    pub(super) active_row: FastqActiveStageToolMatrixRow,
    pub(super) output_row: &'a FastqAdapterOutputContractRow,
    pub(super) parser_row: &'a FastqParserCoverageRow,
    pub(super) expected_row: &'a ExpectedBenchmarkResultRow,
    pub(super) command_row: &'a FastqCommandAdapterCoverageRow,
    pub(super) report_map_row: &'a FastqReportMapRow,
    pub(super) schema_contract: &'a DuplicateStageSchemaContract,
    pub(super) spec: DuplicateStageSpec,
    pub(super) schema_proof_path: &'a str,
}
#[allow(clippy::struct_excessive_bools)]
pub(super) struct RowAssessment {
    pub(super) active_scope_ready: bool,
    pub(super) command_ready: bool,
    pub(super) output_ready: bool,
    pub(super) parser_ready: bool,
    pub(super) expected_result_ready: bool,
    pub(super) report_ready: bool,
    pub(super) schema_ready: bool,
    pub(super) coverage_status: String,
    pub(super) missing_surfaces: Vec<String>,
    pub(super) reason: String,
}
