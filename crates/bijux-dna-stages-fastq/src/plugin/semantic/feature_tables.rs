use super::*;

pub(super) fn observed_feature_table_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if plan.stage_id.as_str() == "fastq.index_reference" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_index_reference_report(&raw_report) {
                    return Some(serde_json::json!({
                        "threads": report.threads,
                        "index_format": report.index_format,
                        "reference_bytes": report.reference_bytes,
                        "index_bytes": report.index_bytes,
                        "index_file_count": report.index_file_count,
                        "index_prefix": report.index_prefix,
                        "emitted_file_count": report.emitted_files.len(),
                        "emitted_files": report.emitted_files,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.normalize_abundance" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_normalize_abundance_report(&raw_report) {
                    return Some(serde_json::json!({
                        "method": report.method,
                        "input_value_column": report.input_value_column,
                        "normalized_value_column": report.normalized_value_column,
                        "compositional_rule": report.compositional_rule,
                        "scale_factor": report.scale_factor,
                        "table_rows": report.table_rows,
                        "sample_count": report.sample_count,
                        "feature_count": report.feature_count,
                        "zero_fraction": report.zero_fraction,
                        "per_sample_sums": report.per_sample_sums,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                        "used_fallback": report.used_fallback,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.infer_asvs" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_infer_asvs_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "denoising_method": report.denoising_method,
                        "pooling_mode": report.pooling_mode,
                        "chimera_policy": report.chimera_policy,
                        "requires_r_runtime": report.requires_r_runtime,
                        "output_table_kind": report.output_table_kind,
                        "asv_count": report.asv_count,
                        "sample_count": report.sample_count,
                        "representative_sequence_count": report.representative_sequence_count,
                        "asv_table_tsv": report.asv_table_tsv,
                        "asv_sequences_fasta": report.asv_sequences_fasta,
                        "taxonomy_ready_fasta": report.taxonomy_ready_fasta,
                        "taxonomy_ready_fastq": report.taxonomy_ready_fastq,
                        "used_fallback": report.used_fallback,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.cluster_otus" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_cluster_otus_report(&raw_report) {
                    return Some(serde_json::json!({
                        "otu_identity": report.otu_identity,
                        "threads": report.threads,
                        "otu_count": report.otu_count,
                        "sample_count": report.sample_count,
                        "representative_sequence_count": report.representative_sequence_count,
                        "output_table_kind": report.output_table_kind,
                        "otu_table": report.otu_table,
                        "otu_representatives": report.otu_representatives,
                        "taxonomy_ready_fasta": report.taxonomy_ready_fasta,
                        "taxonomy_ready_fastq": report.taxonomy_ready_fastq,
                        "used_fallback": report.used_fallback,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    None
}
