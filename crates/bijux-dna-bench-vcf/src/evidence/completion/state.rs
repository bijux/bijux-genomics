use super::model::VcfBindingKey;
use std::collections::BTreeSet;
#[derive(Default)]
pub(super) struct CompletionState {
    pub retained_row: usize,
    pub retained_stage: usize,
    pub retained_tool: usize,
    pub active_row: usize,
    pub removed_row: usize,
    pub active_stage: usize,
    pub active_tool: usize,
    pub expected_result_row: usize,
    pub rendered_command_row: usize,
    pub parser_fixture_row: usize,
    pub local_smoke_row: usize,
    pub local_smoke_host_stage_row: usize,
    pub local_smoke_container_row: usize,
    pub local_smoke_external_artifact_row: usize,
    pub report_map_row: usize,
    pub retained_bindings: BTreeSet<VcfBindingKey>,
    pub active_bindings: BTreeSet<VcfBindingKey>,
}
