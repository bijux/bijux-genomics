use crate::metrics::{
    FastqFilterMetricsV1, FastqMergeMetricsV1, FastqTrimMetricsV1, FastqValidateMetricsV1,
};
use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::invariants::{InvariantStatusV1, StageVerdictV1};

use crate::invariants::evaluation::{
    result, retention_thresholds_for, worst_status, InvariantEvaluation, InvariantThresholds,
};
use crate::parse_effective_params;
use crate::stages::ids::{
    STAGE_CORRECT_ERRORS, STAGE_FILTER_READS, STAGE_MERGE_PAIRS, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY,
    STAGE_PROFILE_READS, STAGE_TRIM_READS, STAGE_EXTRACT_UMIS, STAGE_VALIDATE_READS,
};

include!("metrics/evaluate.rs");
