pub mod deltas;
pub mod spec;
#[allow(unused_imports)]
pub use deltas::{compute_delta, delta_from_counts, ratio_u64, FastqDelta};
#[allow(unused_imports)]
pub use spec::{metric_spec_for_stage, MetricClass, StageMetricSpec};
