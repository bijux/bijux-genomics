use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FastqNormalizedMetricsStageContract {
    pub(crate) stage_id: &'static str,
    pub(crate) extension_id: &'static str,
    pub(crate) required_keys: &'static [&'static str],
}

pub(crate) const FASTQ_NORMALIZED_METRICS_SCHEMA_VERSION: &str = "bijux.fastq_stage_metrics.v1";
pub(crate) const FASTQ_NORMALIZED_METRICS_SCHEMA_ID: &str =
    "bijux.schemas.bench.fastq-normalized-metrics.v1";

const BASE_REQUIRED_KEYS: &[&str] = &["schema_version", "stage", "report_json"];

const FASTQ_NORMALIZED_METRICS_STAGE_CONTRACTS: &[FastqNormalizedMetricsStageContract] = &[
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.validate_reads",
        extension_id: "fastq_validate_reads_v1",
        required_keys: &["validator", "failure_class", "strict_pass", "exit_code"],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.index_reference",
        extension_id: "fastq_index_reference_v1",
        required_keys: &[
            "tool",
            "index_directory",
            "index_files",
            "elapsed_time_s",
            "index_size_bytes",
            "reference_bytes",
            "index_bytes",
            "index_file_count",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.detect_duplicates_premerge",
        extension_id: "fastq_detect_duplicates_premerge_v1",
        required_keys: &["tool", "duplicate_count", "duplicate_fraction", "inspected_pair_count"],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.estimate_library_complexity_prealign",
        extension_id: "fastq_estimate_library_complexity_prealign_v1",
        required_keys: &[
            "tool",
            "reads_in",
            "estimated_complexity",
            "estimated_duplicate_fraction",
            "insufficient_data_reason",
            "complexity_status",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.detect_adapters",
        extension_id: "fastq_detect_adapters_v1",
        required_keys: &["tool", "candidate_adapter_count", "adapter_inference"],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.trim_reads",
        extension_id: "fastq_trim_reads_v2",
        required_keys: &[
            "tool",
            "threads",
            "adapter_policy",
            "adapter_overrides",
            "reads_in",
            "reads_out",
            "reads_retained",
            "reads_dropped",
            "bases_removed",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.trim_terminal_damage",
        extension_id: "fastq_trim_terminal_damage_v2",
        required_keys: &[
            "tool",
            "threads",
            "reads_in",
            "reads_out",
            "reads_retained",
            "bases_removed",
            "execution_policy",
            "trim_5p_bases",
            "trim_3p_bases",
            "udg_classification",
            "ct_ga_asymmetry_pre",
            "ct_ga_asymmetry_post",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.merge_pairs",
        extension_id: "fastq_merge_pairs_v1",
        required_keys: &[
            "tool",
            "paired_mode",
            "merge_engine",
            "threads",
            "merge_overlap",
            "min_length",
            "unmerged_read_policy",
            "reads_r1",
            "reads_r2",
            "input_pair_count",
            "reads_merged",
            "reads_unmerged",
            "merged_pair_count",
            "unmerged_pair_count",
            "discarded_pair_count",
            "merge_rate",
            "raw_backend_report_format",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.cluster_otus",
        extension_id: "fastq_cluster_otus_v1",
        required_keys: &[
            "tool",
            "clustering_threshold",
            "otu_table_tsv",
            "representative_sequences_fasta",
            "otu_count",
            "sample_count",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.remove_chimeras",
        extension_id: "fastq_remove_chimeras_v1",
        required_keys: &[
            "tool",
            "method",
            "detection_scope",
            "filtered_representative_sequences",
            "chimera_count",
            "non_chimera_count",
            "chimera_fraction",
            "raw_backend_report_format",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.remove_duplicates",
        extension_id: "fastq_deduplicate_v1",
        required_keys: &[
            "tool",
            "threads",
            "dedup_mode",
            "keep_order",
            "input_reads",
            "duplicate_reads",
            "unique_reads",
            "output_reads",
            "reads_in",
            "reads_out",
            "duplicates_removed",
            "dedup_rate",
            "duplicate_class_count",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.correct_errors",
        extension_id: "fastq_correct_errors_v1",
        required_keys: &[
            "tool",
            "correction_engine",
            "corrected_reads_r1",
            "corrected_reads",
            "changed_reads",
            "unchanged_reads",
            "kmer_fix_rate",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.filter_reads",
        extension_id: "fastq_filter_reads_v2",
        required_keys: &[
            "tool",
            "filtered_reads_r1",
            "reads_in",
            "reads_out",
            "reads_dropped",
            "reads_retained",
            "reads_removed",
            "reads_removed_by_n",
            "reads_removed_by_entropy",
            "reads_removed_low_complexity",
            "reads_removed_by_kmer",
            "reads_removed_contaminant_kmer",
            "reads_removed_by_length",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.filter_low_complexity",
        extension_id: "fastq_low_complexity_v1",
        required_keys: &[
            "tool",
            "filtered_fastq_r1",
            "reads_in",
            "reads_out",
            "reads_removed_low_complexity",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.extract_umis",
        extension_id: "fastq_extract_umis_v1",
        required_keys: &[
            "tool",
            "umi_pattern",
            "tag_header_format",
            "downstream_propagation",
            "reads_in",
            "reads_out",
            "reads_with_umi",
            "extracted_umi_count",
            "invalid_umi_count",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.profile_reads",
        extension_id: "fastq_profile_reads_v1",
        required_keys: &["tool", "reads_total", "bases_total", "length_histogram_bins"],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.profile_read_lengths",
        extension_id: "fastq_profile_read_lengths_v1",
        required_keys: &[
            "tool",
            "read_count",
            "min_read_length",
            "mean_read_length",
            "median_read_length",
            "max_read_length",
            "histogram_entry_count",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.profile_overrepresented_sequences",
        extension_id: "fastq_profile_overrepresented_sequences_v1",
        required_keys: &["tool", "sequence_count", "flagged_sequences", "top_fraction"],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.normalize_primers",
        extension_id: "fastq_normalize_primers_v1",
        required_keys: &[
            "tool",
            "primer_set_id",
            "normalized_reads_r1",
            "matched_primers",
            "unmatched_reads",
            "trimmed_primer_bases",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.trim_polyg_tails",
        extension_id: "fastq_trim_polyg_tails_v1",
        required_keys: &[
            "tool",
            "trim_polyg",
            "reads_in",
            "reads_out",
            "reads_retained",
            "reads_dropped",
            "bases_removed",
            "trimmed_tail_count",
            "bases_trimmed_polyg",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.screen_taxonomy",
        extension_id: "fastq_screen_taxonomy_v1",
        required_keys: &[
            "tool",
            "classifier",
            "taxonomy_database_id",
            "classified_reads",
            "unclassified_reads",
            "contamination_rate",
            "top_taxa",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.deplete_rrna",
        extension_id: "fastq_deplete_rrna_v2",
        required_keys: &[
            "tool",
            "rrna_db",
            "database_artifact_id",
            "retained_reads",
            "removed_reads",
            "depletion_rate",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.deplete_reference_contaminants",
        extension_id: "fastq_deplete_reference_contaminants_v1",
        required_keys: &[
            "tool",
            "contaminant_reference",
            "contaminant_index_artifact_id",
            "contaminant_screened_reads_r1",
            "contaminant_reads",
            "contaminant_hit_rate",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.deplete_host",
        extension_id: "fastq_deplete_host_v1",
        required_keys: &[
            "tool",
            "host_index_artifact_id",
            "host_depleted_reads_r1",
            "depleted_reads",
            "host_hit_rate",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.report_qc",
        extension_id: "fastq_report_qc_v1",
        required_keys: &[
            "tool",
            "aggregation_engine",
            "aggregation_scope",
            "governed_qc_input_count",
            "multiqc_report",
            "multiqc_data",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.normalize_abundance",
        extension_id: "fastq_normalize_abundance_v1",
        required_keys: &[
            "tool",
            "normalization_method",
            "normalized_abundance_tsv",
            "table_rows",
            "sample_count",
            "sample_totals",
            "numeric_output_valid",
            "zero_fraction",
        ],
    },
    FastqNormalizedMetricsStageContract {
        stage_id: "fastq.infer_asvs",
        extension_id: "fastq_infer_asvs_v1",
        required_keys: &[
            "tool",
            "denoising_method",
            "asv_table_tsv",
            "representative_sequences_fasta",
            "asv_count",
            "sample_count",
        ],
    },
];

#[cfg(test)]
pub(crate) fn fastq_normalized_metrics_stage_contracts(
) -> &'static [FastqNormalizedMetricsStageContract] {
    FASTQ_NORMALIZED_METRICS_STAGE_CONTRACTS
}

pub(crate) fn fastq_normalized_metrics_contract_for_stage(
    stage_id: &str,
) -> Option<&'static FastqNormalizedMetricsStageContract> {
    FASTQ_NORMALIZED_METRICS_STAGE_CONTRACTS.iter().find(|contract| contract.stage_id == stage_id)
}

#[cfg(test)]
pub(crate) fn required_metrics_keys(stage_id: &str) -> &'static [&'static str] {
    fastq_normalized_metrics_contract_for_stage(stage_id)
        .map(|contract| contract.required_keys)
        .unwrap_or(&["schema_version", "stage", "report_json"])
}

pub(crate) fn validate_fastq_normalized_metrics(
    metrics: &serde_json::Value,
) -> Result<&'static FastqNormalizedMetricsStageContract> {
    let object = metrics
        .as_object()
        .ok_or_else(|| anyhow!("FASTQ normalized metrics payload must be a JSON object"))?;
    let schema_version =
        object.get("schema_version").and_then(serde_json::Value::as_str).ok_or_else(|| {
            anyhow!("FASTQ normalized metrics payload is missing string `schema_version`")
        })?;
    if schema_version != FASTQ_NORMALIZED_METRICS_SCHEMA_VERSION {
        return Err(anyhow!(
            "FASTQ normalized metrics payload uses unsupported schema_version `{schema_version}`"
        ));
    }
    let stage_id = object
        .get("stage")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("FASTQ normalized metrics payload is missing string `stage`"))?;
    let contract = fastq_normalized_metrics_contract_for_stage(stage_id).ok_or_else(|| {
        anyhow!("FASTQ normalized metrics payload declares unknown stage `{stage_id}`")
    })?;

    for key in BASE_REQUIRED_KEYS {
        if !object.contains_key(*key) {
            return Err(anyhow!(
                "FASTQ normalized metrics payload for `{stage_id}` is missing base key `{key}`"
            ));
        }
    }
    for key in contract.required_keys {
        if !object.contains_key(*key) {
            return Err(anyhow!(
                "FASTQ normalized metrics payload for `{stage_id}` is missing stage key `{key}`"
            ));
        }
    }

    Ok(contract)
}

pub(crate) fn render_fastq_normalized_metrics_schema() -> serde_json::Value {
    let stage_ids = FASTQ_NORMALIZED_METRICS_STAGE_CONTRACTS
        .iter()
        .map(|contract| contract.stage_id)
        .collect::<Vec<_>>();
    let mut stage_defs = serde_json::Map::new();
    let mut one_of = Vec::new();

    for contract in FASTQ_NORMALIZED_METRICS_STAGE_CONTRACTS {
        let mut required = BTreeSet::new();
        for key in BASE_REQUIRED_KEYS {
            required.insert(*key);
        }
        for key in contract.required_keys {
            required.insert(*key);
        }
        let required = required.into_iter().collect::<Vec<_>>();
        stage_defs.insert(
            contract.stage_id.to_string(),
            serde_json::json!({
                "allOf": [
                    { "$ref": "#/$defs/base" },
                    {
                        "type": "object",
                        "properties": {
                            "stage": { "const": contract.stage_id }
                        },
                        "required": required,
                        "additionalProperties": true,
                        "x-bijux-extension-id": contract.extension_id
                    }
                ]
            }),
        );
        one_of.push(serde_json::json!({
            "$ref": format!("#/$defs/stages/{}", contract.stage_id)
        }));
    }

    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": FASTQ_NORMALIZED_METRICS_SCHEMA_ID,
        "title": "FASTQ normalized benchmark metrics",
        "description": "Governed normalized parser outputs for FASTQ benchmark stages. Each payload must satisfy the shared envelope and exactly one stage-specific extension.",
        "type": "object",
        "oneOf": one_of,
        "$defs": {
            "base": {
                "type": "object",
                "required": BASE_REQUIRED_KEYS,
                "properties": {
                    "schema_version": { "const": FASTQ_NORMALIZED_METRICS_SCHEMA_VERSION },
                    "stage": {
                        "type": "string",
                        "enum": stage_ids
                    },
                    "report_json": { "type": "string" }
                },
                "additionalProperties": true
            },
            "stages": stage_defs
        }
    })
}
