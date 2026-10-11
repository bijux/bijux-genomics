/// Nonserialized canonical observations consumed by retained BAM assessment.
#[derive(Debug, Clone)]
pub struct ScienceThresholdsObservation {
    pub comparable_stage_count: usize,
    pub governed_metric_count: usize,
    pub missing_threshold_metric_count: usize,
    pub missing_threshold_stage_count: usize,
    pub stage_row_count: usize,
    pub threshold_declared_metric_count: usize,
    pub threshold_declared_stage_count: usize,
}
