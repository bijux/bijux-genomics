//! BAM domain helpers for v1.

pub use bijux_dna_planner_bam::stage_api as bam_banks;
pub use bijux_dna_planner_bam::stage_api::{bam_stage_completeness, BamStage};

pub use crate::surface::request_contracts::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};

pub mod domain;
pub mod feature_flags;
pub(crate) mod plan;
pub(crate) mod stage_planning;

#[cfg(feature = "bam_downstream")]
pub use domain::write_local_bias_mitigation_smoke_report;
#[cfg(feature = "bam_downstream")]
pub use domain::write_local_genotyping_plan;
#[cfg(feature = "bam_downstream")]
pub use domain::write_local_haplogroups_plan;
pub use domain::{
    write_local_align_plan, write_local_authenticity_smoke_report,
    write_local_complexity_smoke_report, write_local_contamination_plan,
    write_local_coverage_smoke_summary, write_local_damage_smoke_report,
    write_local_duplication_metrics_smoke_report, write_local_endogenous_content_smoke_report,
    write_local_filter_smoke_report, write_local_gc_bias_smoke_summary,
    write_local_insert_size_smoke_report, write_local_length_filter_smoke_report,
    write_local_mapping_summary_smoke_summary, write_local_mapq_filter_smoke_report,
    write_local_markdup_smoke_report, write_local_overlap_correction_smoke_report,
    write_local_qc_pre_smoke_report, write_local_recalibration_smoke_report,
    write_local_sex_smoke_report, write_local_validate_smoke_report,
};
pub use feature_flags::downstream_enabled;
