use super::VcfStageReadinessBindingKey;
use crate::model::{
    commands::VcfRenderedCommandRow, expectations::VcfExpectedBenchmarkResultRow,
    outputs::VcfAdapterOutputCoverageRow, parsers::VcfParserFixtureCoverageRow,
    reports::VcfReportMapRow, scope::VcfActiveStageToolMatrixRow,
};

/// The identity and contracts actually needed from the all-domain active authority.
#[derive(Debug, Clone)]
pub struct ActiveVcfBinding {
    pub key: VcfStageReadinessBindingKey,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
}

/// Untrusted observations supplied by acquisition adapters. Missing surfaces are valid evidence.
#[derive(Debug, Clone, Default)]
pub struct VcfEvidenceRows {
    pub retained: Vec<VcfActiveStageToolMatrixRow>,
    pub active: Vec<ActiveVcfBinding>,
    pub commands: Vec<VcfRenderedCommandRow>,
    pub outputs: Vec<VcfAdapterOutputCoverageRow>,
    pub parsers: Vec<VcfParserFixtureCoverageRow>,
    pub expected: Vec<VcfExpectedBenchmarkResultRow>,
    pub reports: Vec<VcfReportMapRow>,
}

/// A joined observation. Presence records coverage, not native or scientific qualification.
#[derive(Debug, Clone)]
pub struct VcfStageReadinessBinding {
    pub retained_row: VcfActiveStageToolMatrixRow,
    pub active_row: Option<ActiveVcfBinding>,
    pub command_row: Option<VcfRenderedCommandRow>,
    pub output_row: Option<VcfAdapterOutputCoverageRow>,
    pub parser_row: Option<VcfParserFixtureCoverageRow>,
    pub expected_row: Option<VcfExpectedBenchmarkResultRow>,
    pub report_row: Option<VcfReportMapRow>,
}

pub(super) fn retained_key(row: &VcfActiveStageToolMatrixRow) -> VcfStageReadinessBindingKey {
    VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

pub(super) fn expected_key(row: &VcfExpectedBenchmarkResultRow) -> VcfStageReadinessBindingKey {
    VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

pub(super) fn parser_key(row: &VcfParserFixtureCoverageRow) -> VcfStageReadinessBindingKey {
    VcfStageReadinessBindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}
