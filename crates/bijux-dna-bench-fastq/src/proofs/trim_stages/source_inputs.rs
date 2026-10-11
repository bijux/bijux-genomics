use super::models::BindingKey;
use super::models::TrimStageSchemaContract;
use crate::proofs::inputs::observations::ExpectedResultObservation;
use crate::proofs::inputs::observations::PublishedSchema;
use crate::readiness::active::FastqActiveStageToolMatrixRow;
use crate::readiness::adapters::FastqCommandAdapterCoverageRow;
use crate::readiness::output_contract::FastqAdapterOutputContractRow;
use crate::readiness::parsers::FastqParserCoverageRow;
use crate::readiness::report_map::FastqReportMapRow;
use std::collections::BTreeMap;
pub(super) struct SourceEvidence {
    pub(super) active_matrix: Vec<FastqActiveStageToolMatrixRow>,
    pub(super) output_contract: Vec<FastqAdapterOutputContractRow>,
    pub(super) parser_coverage: Vec<FastqParserCoverageRow>,
    pub(super) expected_results: Vec<ExpectedResultObservation>,
    pub(super) report_map: Vec<FastqReportMapRow>,
    pub(super) command_coverage: Vec<FastqCommandAdapterCoverageRow>,
    pub(super) schema_report: PublishedSchema,
    pub(super) schema_contracts: BTreeMap<String, TrimStageSchemaContract>,
}
pub(super) struct AdmittedBindings {
    pub(super) active_rows: Vec<FastqActiveStageToolMatrixRow>,
    pub(super) output_by_binding: BTreeMap<BindingKey, FastqAdapterOutputContractRow>,
    pub(super) parser_by_binding: BTreeMap<BindingKey, FastqParserCoverageRow>,
    pub(super) expected_by_binding: BTreeMap<BindingKey, ExpectedResultObservation>,
    pub(super) command_by_binding: BTreeMap<BindingKey, FastqCommandAdapterCoverageRow>,
    pub(super) report_map_by_binding: BTreeMap<BindingKey, FastqReportMapRow>,
    pub(super) schema_report: PublishedSchema,
    pub(super) schema_contracts: BTreeMap<String, TrimStageSchemaContract>,
}
