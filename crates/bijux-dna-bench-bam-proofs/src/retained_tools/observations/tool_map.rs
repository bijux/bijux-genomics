use super::bindings::BindingObservation;
/// Nonserialized canonical observations consumed by retained BAM assessment.
#[derive(Debug, Clone)]
pub struct ToolMapObservation {
    pub row_count: usize,
    pub rows: Vec<BindingObservation>,
    pub stage_count: usize,
    pub tool_count: usize,
}
