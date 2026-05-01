use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterScientificBoundary {
    RemovesWholeReads,
    TrimsTerminalBases,
    PreservesReadCount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FilterPolicyEntryV1 {
    pub stage_id: String,
    pub filter_family: String,
    pub removal_surface: FilterScientificBoundary,
    pub changed_metrics: Vec<String>,
    pub scientific_caveats: Vec<String>,
}

#[must_use]
pub fn governed_filter_policy_matrix() -> Vec<FilterPolicyEntryV1> {
    vec![
        FilterPolicyEntryV1 {
            stage_id: "fastq.filter_reads".to_string(),
            filter_family: "quality_and_ambiguous_base_filter".to_string(),
            removal_surface: FilterScientificBoundary::RemovesWholeReads,
            changed_metrics: vec![
                "reads_removed_by_n".to_string(),
                "reads_removed_by_length".to_string(),
                "reads_dropped".to_string(),
                "mean_q_after".to_string(),
            ],
            scientific_caveats: vec![
                "may enrich for higher-quality fragments and distort empirical error profiles"
                    .to_string(),
                "paired-end rejection can remove intact mates to preserve synchronization"
                    .to_string(),
            ],
        },
        FilterPolicyEntryV1 {
            stage_id: "fastq.filter_low_complexity".to_string(),
            filter_family: "low_complexity_and_polyx_filter".to_string(),
            removal_surface: FilterScientificBoundary::RemovesWholeReads,
            changed_metrics: vec![
                "reads_removed_low_complexity".to_string(),
                "bases_out".to_string(),
                "pairs_out".to_string(),
            ],
            scientific_caveats: vec![
                "may remove true biological low-complexity molecules in repetitive or biased assays"
                    .to_string(),
                "polyx thresholds are technology-sensitive and should not be compared across platforms without calibration"
                    .to_string(),
            ],
        },
        FilterPolicyEntryV1 {
            stage_id: "fastq.trim_polyg_tails".to_string(),
            filter_family: "poly_g_tail_trim".to_string(),
            removal_surface: FilterScientificBoundary::TrimsTerminalBases,
            changed_metrics: vec![
                "reads_out".to_string(),
                "bases_out".to_string(),
                "mean_q_after".to_string(),
            ],
            scientific_caveats: vec![
                "targets platform-specific tail artifacts and should remain explicit in cross-run comparisons"
                    .to_string(),
                "base trimming can shorten authentic inserts and alter downstream mergeability"
                    .to_string(),
            ],
        },
        FilterPolicyEntryV1 {
            stage_id: "fastq.trim_terminal_damage".to_string(),
            filter_family: "terminal_damage_trim".to_string(),
            removal_surface: FilterScientificBoundary::TrimsTerminalBases,
            changed_metrics: vec![
                "reads_out".to_string(),
                "bases_out".to_string(),
                "mean_q_after".to_string(),
            ],
            scientific_caveats: vec![
                "can erase authentic terminal damage signals and must not be presented as neutral preprocessing"
                    .to_string(),
                "damage trimming boundaries are assay-specific and especially consequential for ancient-dna interpretation"
                    .to_string(),
            ],
        },
    ]
}
