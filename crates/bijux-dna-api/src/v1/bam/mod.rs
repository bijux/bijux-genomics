//! BAM domain helpers for v1.

pub use bijux_dna_planner_bam::stage_api as bam_banks;
pub use bijux_dna_planner_bam::stage_api::{bam_stage_completeness, BamStage};

pub use crate::surface::request_contracts::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};

pub mod domain;
pub mod feature_flags;
pub(crate) mod plan;
pub(crate) mod stage_planning;

pub use domain::{
    write_local_align_plan, write_local_complexity_smoke_report,
    write_local_coverage_smoke_summary, write_local_duplication_metrics_smoke_report,
    write_local_filter_smoke_report, write_local_length_filter_smoke_report,
    write_local_mapping_summary_smoke_summary, write_local_mapq_filter_smoke_report,
    write_local_markdup_smoke_report, write_local_qc_pre_smoke_report,
    write_local_validate_smoke_report,
};
pub use feature_flags::downstream_enabled;
