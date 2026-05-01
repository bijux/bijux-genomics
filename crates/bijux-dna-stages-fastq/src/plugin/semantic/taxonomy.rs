use std::fs;

use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::observer::{
    parse_deplete_host_report, parse_deplete_reference_contaminants_report,
    parse_deplete_rrna_report, parse_screen_taxonomy_report,
};

pub(super) fn observed_taxonomy_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    match plan.stage_id.as_str() {
        "fastq.screen_taxonomy" => screen_taxonomy_metrics(artifacts),
        "fastq.deplete_rrna" => deplete_rrna_metrics(artifacts),
        "fastq.deplete_reference_contaminants" => deplete_reference_contaminants_metrics(artifacts),
        "fastq.deplete_host" => deplete_host_metrics(artifacts),
        _ => None,
    }
}

fn report_raw(artifacts: &[ArtifactRef], name: &str) -> Option<String> {
    artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
}

fn screen_taxonomy_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report =
        parse_screen_taxonomy_report(&report_raw(artifacts, "classification_report_json")?).ok()?;
    Some(serde_json::json!({
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
    }))
}

fn deplete_rrna_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_deplete_rrna_report(&report_raw(artifacts, "rrna_report_json")?).ok()?;
    Some(serde_json::json!({
        "paired_mode": report.paired_mode,
        "threads": report.threads,
        "rrna_db": report.rrna_db,
        "database_artifact_id": report.database_artifact_id,
        "database_build_id": report.database_build_id,
        "database_digest": report.database_digest,
        "screening_engine": report.screening_engine,
        "report_format": report.report_format,
        "min_identity": report.min_identity,
        "retained_read_role": report.retained_read_role,
        "rejected_read_role": report.rejected_read_role,
        "reads_removed": report.reads_removed,
        "bases_removed": report.bases_removed,
        "rrna_fraction_removed": report.rrna_fraction_removed,
        "rrna_report_tsv": report.rrna_report_tsv,
        "raw_backend_report": report.raw_backend_report,
        "raw_backend_report_format": report.raw_backend_report_format,
    }))
}

fn deplete_reference_contaminants_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_deplete_reference_contaminants_report(&report_raw(
        artifacts,
        "contaminant_screen_report_json",
    )?)
    .ok()?;
    Some(serde_json::json!({
        "paired_mode": report.paired_mode,
        "threads": report.threads,
        "reference_catalog_id": report.reference_catalog_id,
        "contaminant_reference": report.contaminant_reference,
        "reference_index_artifact_id": report.reference_index_artifact_id,
        "reference_index_backend": report.reference_index_backend,
        "reference_build_id": report.reference_build_id,
        "reference_digest": report.reference_digest,
        "match_threshold": report.match_threshold,
        "retained_read_role": report.retained_read_role,
        "rejected_read_role": report.rejected_read_role,
        "retain_unmapped_pairs": report.retain_unmapped_pairs,
        "reads_removed": report.reads_removed,
        "bases_removed": report.bases_removed,
        "contaminant_fraction_removed": report.contaminant_fraction_removed,
        "raw_backend_report": report.raw_backend_report,
        "raw_backend_report_format": report.raw_backend_report_format,
    }))
}

fn deplete_host_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report =
        parse_deplete_host_report(&report_raw(artifacts, "host_depletion_report_json")?).ok()?;
    Some(serde_json::json!({
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
    }))
}
