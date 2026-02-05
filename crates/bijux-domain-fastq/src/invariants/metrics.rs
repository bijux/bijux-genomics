use crate::metrics::{
    FastqFilterMetricsV1, FastqMergeMetricsV1, FastqTrimMetricsV1, FastqValidateMetricsV1,
};
use bijux_core::{InvariantStatusV1, StageVerdictV1};

use crate::invariants::core::{
    result, retention_thresholds_for, worst_status, InvariantEvaluation, InvariantThresholds,
};
use crate::parse_effective_params;

include!("metrics/evaluate.rs");
