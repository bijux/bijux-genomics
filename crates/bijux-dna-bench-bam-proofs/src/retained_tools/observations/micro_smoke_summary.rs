/// Nonserialized canonical observations consumed by retained BAM assessment.
#[derive(Debug, Clone)]
pub struct MicroSmokeObservation {
    pub container_needed_count: usize,
    pub family_count: usize,
    pub local_smoke_count: usize,
    pub passes_behavior_test: bool,
    pub unavailable_count: usize,
}
