use super::bindings::BindingObservation;
/// Nonserialized canonical observations consumed by retained BAM assessment.
#[derive(Debug, Clone)]
pub struct LocalSmokeObservation {
    pub container_smoke_row_count: usize,
    pub external_artifact_row_count: usize,
    pub host_stage_smoke_row_count: usize,
    pub row_count: usize,
    pub rows: Vec<BindingObservation>,
    pub stage_count: usize,
    pub tool_count: usize,
}
