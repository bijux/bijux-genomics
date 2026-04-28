use std::fs;

use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::metrics::f64_from_u64;
use crate::observer::{
    parse_correct_errors_report, parse_deduplicate_report, parse_remove_chimeras_report,
    parse_remove_duplicates_provenance, parse_remove_duplicates_report,
};

pub(super) fn observed_cleanup_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    match plan.stage_id.as_str() {
        "fastq.remove_duplicates" => remove_duplicates_metrics(artifacts),
        "fastq.remove_chimeras" => remove_chimeras_metrics(artifacts),
        "fastq.correct_errors" => correct_errors_metrics(artifacts),
        _ => None,
    }
}

fn report_raw(artifacts: &[ArtifactRef], name: &str) -> Option<String> {
    artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
}

fn remove_duplicates_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let raw_report = report_raw(artifacts, "report_json")?;
    if let Ok(report) = parse_remove_duplicates_report(&raw_report) {
        let provenance = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "duplicate_provenance_json")
            .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
            .and_then(|raw| parse_remove_duplicates_provenance(&raw).ok());
        return Some(serde_json::json!({
            "paired_mode": report.paired_mode,
            "dedup_mode": report.dedup_mode,
            "keep_order": report.keep_order,
            "pair_count_match": report.pair_count_match,
            "duplicates_removed": report.duplicates_removed,
            "dedup_rate": report.dedup_rate,
            "duplicate_class_count": report.duplicate_classes.len(),
            "duplicate_classes": report.duplicate_classes,
            "duplicate_provenance_json": report.duplicate_provenance_json,
            "raw_backend_report": report.raw_backend_report,
            "raw_backend_report_format": report.raw_backend_report_format,
            "backend_log": provenance.as_ref().and_then(|value| value.backend_log.clone()),
        }));
    }
    legacy_deduplicate_metrics(&raw_report)
}

fn legacy_deduplicate_metrics(raw_report: &str) -> Option<serde_json::Value> {
    let (reads_in, reads_out) = parse_deduplicate_report(raw_report).ok()?;
    let duplicates_removed = reads_in.saturating_sub(reads_out);
    let dedup_rate =
        if reads_in > 0 { f64_from_u64(duplicates_removed) / f64_from_u64(reads_in) } else { 0.0 };
    Some(serde_json::json!({
        "reads_in": reads_in,
        "reads_out": reads_out,
        "duplicates_removed": duplicates_removed,
        "dedup_rate": dedup_rate,
    }))
}

fn remove_chimeras_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_remove_chimeras_report(&report_raw(artifacts, "report_json")?).ok()?;
    Some(serde_json::json!({
        "paired_mode": report.paired_mode,
        "threads": report.threads,
        "method": report.method,
        "detection_scope": report.detection_scope,
        "reads_in": report.reads_in,
        "reads_out": report.reads_out,
        "chimeras_removed": report.chimeras_removed,
        "chimera_fraction": report.chimera_fraction,
        "used_fallback": report.used_fallback,
        "chimeras_fasta": report.chimeras_fasta,
        "uchime_report_tsv": report.uchime_report_tsv,
        "raw_backend_report": report.raw_backend_report,
        "raw_backend_report_format": report.raw_backend_report_format,
        "backend_metrics": report.backend_metrics,
    }))
}

fn correct_errors_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = parse_correct_errors_report(&report_raw(artifacts, "report_json")?).ok()?;
    Some(serde_json::json!({
        "paired_mode": report.paired_mode,
        "threads": report.threads,
        "correction_engine": report.correction_engine,
        "quality_encoding": report.quality_encoding,
        "kmer_size": report.kmer_size,
        "genome_size": report.genome_size,
        "max_memory_gb": report.max_memory_gb,
        "trusted_kmer_artifact": report.trusted_kmer_artifact,
        "conservative_mode": report.conservative_mode,
        "corrected_reads": report.corrected_reads,
        "kmer_fix_rate": report.kmer_fix_rate,
        "correction_effect": report.correction_effect,
        "raw_backend_report": report.raw_backend_report,
        "raw_backend_report_format": report.raw_backend_report_format,
        "input_r1": report.input_r1,
        "input_r2": report.input_r2,
        "output_r1": report.output_r1,
        "output_r2": report.output_r2,
    }))
}
