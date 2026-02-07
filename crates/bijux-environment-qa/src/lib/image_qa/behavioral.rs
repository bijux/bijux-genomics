// split to keep module size manageable

use super::fs::temp_out_dir;
use bijux_domain_fastq::{
    STAGE_CORRECT, STAGE_FILTER, STAGE_MERGE, STAGE_QC_POST, STAGE_STATS_NEUTRAL, STAGE_TRIM,
    STAGE_UMI, STAGE_VALIDATE_PRE,
};

include!("behavioral/scenarios.rs");
include!("behavioral/runner.rs");
