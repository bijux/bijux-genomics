pub mod deltas;
pub mod spec;
pub mod types;
#[allow(unused_imports)]
pub use deltas::{compute_delta, delta_from_counts, ratio_u64, FastqDelta};
#[allow(unused_imports)]
pub use spec::{metric_spec_for_stage, MetricClass, StageMetricSpec};
#[allow(unused_imports)]
pub use types::{
    FastqCorrectMetricsV1, FastqDeltaMetricsV1, FastqDetectAdaptersMetricsV1, FastqFilterMetricsV1,
    FastqMergeMetricsV1, FastqPreprocessMetricsV1, FastqQcPostMetricsV1, FastqTrimMetricsV1,
    FastqUmiMetricsV1, FastqValidateMetricsV1, RetentionReportMetricV1,
};
