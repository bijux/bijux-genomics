use super::*;

pub(super) fn observed_cleanup_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if plan.stage_id.as_str() == "fastq.remove_duplicates" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
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
                let parsed: Result<(u64, u64), _> = parse_deduplicate_report(&raw_report);
                if let Ok((reads_in, reads_out)) = parsed {
                    let duplicates_removed = reads_in.saturating_sub(reads_out);
                    let dedup_rate = if reads_in > 0 {
                        duplicates_removed as f64 / reads_in as f64
                    } else {
                        0.0
                    };
                    return Some(serde_json::json!({
                        "reads_in": reads_in,
                        "reads_out": reads_out,
                        "duplicates_removed": duplicates_removed,
                        "dedup_rate": dedup_rate,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.remove_chimeras" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_remove_chimeras_report(&raw_report) {
                    return Some(serde_json::json!({
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
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.correct_errors" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_correct_errors_report(&raw_report) {
                    return Some(serde_json::json!({
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
                    }));
                }
            }
        }
    }
    None
}
