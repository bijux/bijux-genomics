//! BAM domain helpers for v1.

pub use bijux_dna_planner_bam::stage_api as bam_banks;
pub use bijux_dna_planner_bam::stage_api::{bam_stage_completeness, BamStage};

pub use crate::surface::request_contracts::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};

pub mod domain;
pub mod feature_flags;
pub(crate) mod plan;
pub(crate) mod stage_planning;

pub use domain::{
    write_local_align_plan, write_local_qc_pre_smoke_report, write_local_validate_smoke_report,
};
pub use feature_flags::downstream_enabled;
