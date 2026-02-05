use crate::metrics::{
    FastqFilterMetricsV1, FastqMergeMetricsV1, FastqTrimMetricsV1, FastqValidateMetricsV1,
};
use bijux_core::ids::StageId;
use bijux_core::primitives::invariants::{InvariantStatusV1, StageVerdictV1};

use crate::invariants::core::{
    result, retention_thresholds_for, worst_status, InvariantEvaluation, InvariantThresholds,
};
use crate::parse_effective_params;
use crate::stage_registry::{
    STAGE_CORRECT, STAGE_FILTER, STAGE_MERGE, STAGE_PREPROCESS, STAGE_QC_POST, STAGE_SCREEN,
    STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};

include!("metrics/evaluate.rs");
