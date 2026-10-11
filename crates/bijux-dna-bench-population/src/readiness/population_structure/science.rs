use super::{
    GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID, REQUIRED_CROSS_POPULATION_PAIR_COUNT,
    REQUIRED_PAIR_COUNT, REQUIRED_SAMPLE_COUNT, REQUIRED_SAMPLE_IDS, REQUIRED_SELECTED_K,
    REQUIRED_STATUS, REQUIRED_WITHIN_POPULATION_PAIR_COUNT,
};
use crate::population_structure::LocalVcfPopulationStructureSmokeReport;
use std::collections::BTreeSet;
pub(super) fn population_structure_smoke_matches_governed_contract(
    report: &LocalVcfPopulationStructureSmokeReport,
) -> bool {
    !report.output_root.trim().is_empty()
        && !report.population_structure_json_path.trim().is_empty()
        && !report.source_population_structure_path.trim().is_empty()
        && !report.source_pruned_variants_path.trim().is_empty()
        && !report.source_logs_path.trim().is_empty()
        && !report.source_pca_report_path.trim().is_empty()
        && !report.source_admixture_report_path.trim().is_empty()
        && !report.stage_result_manifest_path.trim().is_empty()
        && report.status == REQUIRED_STATUS
        && report.selected_pca_tool_id == GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID
        && report.selected_admixture_tool_id == GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID
        && report.consumed_pca.report_path == report.source_pca_report_path
        && report.consumed_pca.sample_count == REQUIRED_SAMPLE_COUNT
        && !report.consumed_pca.execution_mode.trim().is_empty()
        && report.consumed_admixture.report_path == report.source_admixture_report_path
        && report.consumed_admixture.sample_count == REQUIRED_SAMPLE_COUNT
        && report.consumed_admixture.selected_k == REQUIRED_SELECTED_K
        && !report.consumed_admixture.execution_mode.trim().is_empty()
        && report.consumed_admixture.status == REQUIRED_STATUS
        && report.sample_groups.len() == REQUIRED_SAMPLE_IDS.len()
        && report.sample_groups.iter().map(|row| row.sample_id.as_str()).collect::<BTreeSet<_>>()
            == REQUIRED_SAMPLE_IDS.into_iter().collect::<BTreeSet<_>>()
        && report.sample_groups.iter().all(|row| {
            !row.sample_id.trim().is_empty()
                && !row.population_id.trim().is_empty()
                && !row.population_label.trim().is_empty()
                && !row.sex.trim().is_empty()
                && !row.dominant_cluster.trim().is_empty()
                && row.dominant_fraction.is_finite()
                && row.dominant_fraction >= 0.0
                && row.dominant_fraction <= 1.0
                && row.pc1.is_finite()
                && row.pc2.is_finite()
                && row.status == REQUIRED_STATUS
        })
        && report.distance_summary.sample_count == REQUIRED_SAMPLE_COUNT
        && report.distance_summary.pair_count == REQUIRED_PAIR_COUNT
        && report.distance_summary.within_population_pair_count
            == REQUIRED_WITHIN_POPULATION_PAIR_COUNT
        && report.distance_summary.cross_population_pair_count
            == REQUIRED_CROSS_POPULATION_PAIR_COUNT
        && report.distance_summary.min_pc_distance.is_finite()
        && report.distance_summary.max_pc_distance.is_finite()
        && report.distance_summary.mean_pc_distance.is_finite()
        && report.distance_summary.min_pc_distance > 0.0
        && report.distance_summary.max_pc_distance >= report.distance_summary.min_pc_distance
        && report.distance_summary.mean_pc_distance >= report.distance_summary.min_pc_distance
        && report.distance_summary.mean_pc_distance <= report.distance_summary.max_pc_distance
}
