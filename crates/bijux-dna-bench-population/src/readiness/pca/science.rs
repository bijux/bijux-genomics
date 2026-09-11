use super::{REQUIRED_SAMPLE_COUNT, REQUIRED_SAMPLE_IDS, REQUIRED_VARIANT_COUNT};
use crate::pca::LocalVcfPcaSmokeReport;
use std::collections::BTreeSet;
pub(super) fn pca_smoke_matches_governed_contract(report: &LocalVcfPcaSmokeReport) -> bool {
    let expected_samples = REQUIRED_SAMPLE_IDS.iter().copied().collect::<BTreeSet<_>>();
    let observed_samples =
        report.rows.iter().map(|row| row.sample_id.as_str()).collect::<BTreeSet<_>>();

    !report.input_vcf_path.trim().is_empty()
        && !report.sample_metadata_path.trim().is_empty()
        && !report.population_metadata_path.trim().is_empty()
        && !report.population_labels_manifest_path.trim().is_empty()
        && !report.output_root.trim().is_empty()
        && !report.pca_tsv_path.trim().is_empty()
        && !report.pca_json_path.trim().is_empty()
        && !report.source_eigenvec_path.trim().is_empty()
        && !report.source_eigenval_path.trim().is_empty()
        && !report.source_pca_manifest_path.trim().is_empty()
        && !report.source_logs_path.trim().is_empty()
        && !report.stage_result_manifest_path.trim().is_empty()
        && report.variant_count == REQUIRED_VARIANT_COUNT
        && report.sample_count == REQUIRED_SAMPLE_COUNT
        && report.excluded_samples.is_empty()
        && report.unexpected_samples.is_empty()
        && report.eigenvalues.len() >= 2
        && report.rows.len() == REQUIRED_SAMPLE_IDS.len()
        && observed_samples == expected_samples
        && report.rows.iter().all(|row| {
            !row.population_id.trim().is_empty()
                && !row.population_label.trim().is_empty()
                && !row.sex.trim().is_empty()
                && row.pc1.is_finite()
                && row.pc2.is_finite()
        })
}
