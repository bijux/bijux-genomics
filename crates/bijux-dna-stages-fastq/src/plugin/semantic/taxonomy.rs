use super::*;

pub(super) fn observed_taxonomy_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if plan.stage_id.as_str() == "fastq.screen_taxonomy" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "classification_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_screen_taxonomy_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "classifier": report.classifier,
                        "report_format": report.report_format,
                        "assignment_format": report.assignment_format,
                        "database_catalog_id": report.database_catalog_id,
                        "database_artifact_id": report.database_artifact_id,
                        "database_digest": report.database_digest,
                        "minimum_confidence": report.minimum_confidence,
                        "emit_unclassified": report.emit_unclassified,
                        "contamination_rate": report.contamination_rate,
                        "classified_fraction": report.classified_fraction,
                        "unclassified_fraction": report.unclassified_fraction,
                        "summary_entry_count": report.summary_entries.len(),
                        "top_taxa": report.top_taxa.iter().map(|entry| entry.label.clone()).collect::<Vec<_>>(),
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.deplete_rrna" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "rrna_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_deplete_rrna_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "rrna_db": report.rrna_db,
                        "database_artifact_id": report.database_artifact_id,
                        "database_build_id": report.database_build_id,
                        "screening_engine": report.screening_engine,
                        "report_format": report.report_format,
                        "min_identity": report.min_identity,
                        "reads_removed": report.reads_removed,
                        "bases_removed": report.bases_removed,
                        "rrna_fraction_removed": report.rrna_fraction_removed,
                        "rrna_report_tsv": report.rrna_report_tsv,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.deplete_reference_contaminants" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "contaminant_screen_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_deplete_reference_contaminants_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "reference_catalog_id": report.reference_catalog_id,
                        "contaminant_reference": report.contaminant_reference,
                        "index_artifact": report.index_artifact,
                        "reference_index_backend": report.reference_index_backend,
                        "reference_build_id": report.reference_build_id,
                        "reference_digest": report.reference_digest,
                        "retain_unmapped_pairs": report.retain_unmapped_pairs,
                        "reads_removed": report.reads_removed,
                        "bases_removed": report.bases_removed,
                        "contaminant_fraction_removed": report.contaminant_fraction_removed,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.deplete_host" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "host_depletion_report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_deplete_host_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "reference_scope": report.reference_scope,
                        "reference_catalog_id": report.reference_catalog_id,
                        "reference_index_artifact_id": report.reference_index_artifact_id,
                        "reference_index_backend": report.reference_index_backend,
                        "reference_build_id": report.reference_build_id,
                        "reference_digest": report.reference_digest,
                        "identity_threshold": report.identity_threshold,
                        "retained_read_policy": report.retained_read_policy,
                        "report_format": report.report_format,
                        "retain_unmapped_pairs": report.retain_unmapped_pairs,
                        "reads_removed": report.reads_removed,
                        "bases_removed": report.bases_removed,
                        "host_fraction_removed": report.host_fraction_removed,
                        "removed_host_r1": report.removed_host_r1,
                        "removed_host_r2": report.removed_host_r2,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    None
}
