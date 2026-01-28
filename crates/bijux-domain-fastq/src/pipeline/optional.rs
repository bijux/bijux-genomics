#[derive(Debug, Clone, Copy)]
pub struct OptionalStage {
    pub stage_id: &'static str,
    pub prerequisites: &'static [&'static str],
}

#[must_use]
pub fn optional_stages() -> Vec<OptionalStage> {
    crate::domain::OPTIONAL_BRANCHES
        .iter()
        .map(|(stage_id, prerequisites)| OptionalStage {
            stage_id,
            prerequisites,
        })
        .collect()
}
