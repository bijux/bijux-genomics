use bijux_dna_core::id_catalog::{
    BAM_AUTHENTICITY, BAM_CONTAMINATION, BAM_DAMAGE, BAM_ENDOGENOUS_CONTENT, BAM_GENOTYPING,
    BAM_HAPLOGROUPS, BAM_KINSHIP, BAM_OVERLAP_CORRECTION, BAM_RECALIBRATION, BAM_SEX,
};
pub const DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/BAM_ALL_RETAINED_TOOLS_COMPLETE.json";

pub const BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_all_retained_tools_complete.v1";

pub(super) const EXPECTED_CHECKED_GOAL_COUNT: usize = 19;

pub(super) const EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT: usize = 6;

pub(super) const EXPECTED_MICRO_SMOKE_FAMILY_COUNT: usize = 12;

pub(super) const EXPECTED_BAM_COMPARABLE_STAGE_COUNT: usize = 15;

pub(super) const EXPECTED_BAM_GOVERNED_METRIC_COUNT: usize = 51;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BamStageCompletionGoal {
    pub(super) goal_id: u32,
    pub(super) stage_id: &'static str,
    pub(super) output_path: &'static str,
    pub(super) expected_active_row_count: usize,
    pub(super) expected_checked_surface_count: usize,
}

pub(super) const BAM_STAGE_COMPLETION_GOALS: &[BamStageCompletionGoal] = &[
    BamStageCompletionGoal {
        goal_id: 431,
        stage_id: BAM_OVERLAP_CORRECTION,
        output_path: crate::overlap_correction::DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 11,
    },
    BamStageCompletionGoal {
        goal_id: 432,
        stage_id: BAM_ENDOGENOUS_CONTENT,
        output_path: crate::endogenous_content::DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 11,
    },
    BamStageCompletionGoal {
        goal_id: 433,
        stage_id: BAM_DAMAGE,
        output_path: crate::damage::DEFAULT_BAM_DAMAGE_COMPLETE_PATH,
        expected_active_row_count: 5,
        expected_checked_surface_count: 13,
    },
    BamStageCompletionGoal {
        goal_id: 434,
        stage_id: BAM_AUTHENTICITY,
        output_path: crate::authenticity::DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH,
        expected_active_row_count: 3,
        expected_checked_surface_count: 12,
    },
    BamStageCompletionGoal {
        goal_id: 435,
        stage_id: BAM_CONTAMINATION,
        output_path: crate::contamination::DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH,
        expected_active_row_count: 3,
        expected_checked_surface_count: 14,
    },
    BamStageCompletionGoal {
        goal_id: 436,
        stage_id: BAM_SEX,
        output_path: crate::sex::DEFAULT_BAM_SEX_COMPLETE_PATH,
        expected_active_row_count: 2,
        expected_checked_surface_count: 15,
    },
    BamStageCompletionGoal {
        goal_id: 437,
        stage_id: BAM_HAPLOGROUPS,
        output_path: crate::haplogroups::DEFAULT_BAM_HAPLOGROUPS_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 14,
    },
    BamStageCompletionGoal {
        goal_id: 438,
        stage_id: BAM_RECALIBRATION,
        output_path: crate::recalibration::DEFAULT_BAM_RECALIBRATION_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 13,
    },
    BamStageCompletionGoal {
        goal_id: 439,
        stage_id: BAM_GENOTYPING,
        output_path: crate::genotyping::DEFAULT_BAM_GENOTYPING_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 15,
    },
    BamStageCompletionGoal {
        goal_id: 440,
        stage_id: BAM_KINSHIP,
        output_path: crate::kinship::DEFAULT_BAM_KINSHIP_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 19,
    },
];
