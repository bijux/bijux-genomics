pub mod registry;

#[must_use]
pub fn tools_for_stage(stage_id: &bijux_core::ids::StageId) -> Vec<String> {
    registry::allowed_tools_for_stage(stage_id)
}

#[must_use]
pub fn available_tools() -> Vec<String> {
    let mut all = Vec::new();
    for stage in crate::fastq::registry() {
        all.extend(tools_for_stage(&stage.id));
    }
    all.sort();
    all.dedup();
    all
}
