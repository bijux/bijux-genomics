//! Owner: scalar shape and explicit enum admission for local benchmark parameter observations.
use anyhow::{anyhow, ensure, Result};
use std::path::PathBuf;

pub(super) fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(serde_json::Value::as_str).map(ToString::to_string)
}

pub(super) fn json_u32(value: &serde_json::Value, key: &str) -> Option<u32> {
    value.get(key).and_then(serde_json::Value::as_u64).and_then(|raw| u32::try_from(raw).ok())
}

pub(super) fn json_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value.get(key).and_then(serde_json::Value::as_u64)
}

pub(super) fn json_f64(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key).and_then(serde_json::Value::as_f64)
}

pub(super) fn json_bool(value: &serde_json::Value, key: &str) -> Option<bool> {
    value.get(key).and_then(serde_json::Value::as_bool)
}

pub(super) fn json_path(value: &serde_json::Value, key: &str) -> Option<PathBuf> {
    json_string(value, key).map(PathBuf::from)
}

/// Use the domain enum's wire decoder while preserving the benchmark field error context.
pub(super) fn decode_enum_value<T: serde::de::DeserializeOwned>(
    raw: &str,
    field: &str,
) -> Result<T> {
    T::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(raw))
        .map_err(|_| anyhow!("unsupported {field} `{raw}`"))
}

#[derive(Clone, Copy)]
enum ScalarKind {
    Text,
    U32,
    U64,
    Number,
    Boolean,
}

pub(super) fn validate_raw_fields(stage: &str, raw: &serde_json::Value) -> Result<()> {
    ensure!(raw.is_null() || raw.is_object(), "raw benchmark parameters must be an object or null");
    let fields = raw_fields(stage);
    for (field, kind) in fields {
        let Some(value) = raw.get(field).filter(|value| !value.is_null()) else {
            continue;
        };
        let valid = match kind {
            ScalarKind::Text => value.is_string(),
            ScalarKind::U32 => value.as_u64().is_some_and(|value| u32::try_from(value).is_ok()),
            ScalarKind::U64 => value.as_u64().is_some(),
            ScalarKind::Number => value.as_f64().is_some_and(f64::is_finite),
            ScalarKind::Boolean => value.is_boolean(),
        };
        ensure!(
            valid,
            "invalid scalar type or range for benchmark parameter `{field}` in stage `{stage}`"
        );
    }
    Ok(())
}

fn raw_fields(stage: &str) -> &'static [(&'static str, ScalarKind)] {
    match stage {
        "fastq.cluster_otus" => {
            &[("otu_identity", ScalarKind::Number), ("threads", ScalarKind::U32)]
        }
        "fastq.correct_errors" => &[
            ("threads", ScalarKind::U32),
            ("quality_encoding", ScalarKind::Text),
            ("kmer_size", ScalarKind::U32),
            ("musket_kmer_budget", ScalarKind::U64),
            ("genome_size", ScalarKind::U64),
            ("max_memory_gb", ScalarKind::U32),
            ("trusted_kmer_artifact", ScalarKind::Text),
            ("conservative_mode", ScalarKind::Boolean),
        ],
        "fastq.deplete_host" => &[
            ("host_identity_threshold", ScalarKind::Number),
            ("retain_unmapped_only", ScalarKind::Boolean),
            ("threads", ScalarKind::U32),
        ],
        "fastq.deplete_reference_contaminants" => {
            &[("decoy_mode", ScalarKind::Text), ("threads", ScalarKind::U32)]
        }
        "fastq.deplete_rrna" => &[
            ("rrna_db", ScalarKind::Text),
            ("min_identity", ScalarKind::Number),
            ("threads", ScalarKind::U32),
        ],
        "fastq.detect_adapters" => &[("threads", ScalarKind::U32)],
        "fastq.extract_umis" => &[
            ("threads", ScalarKind::U32),
            ("umi_pattern", ScalarKind::Text),
            ("extraction_location", ScalarKind::Text),
            ("read_name_transform", ScalarKind::Text),
            ("failed_extraction_policy", ScalarKind::Text),
            ("grouping_policy", ScalarKind::Text),
            ("downstream_dedup_policy", ScalarKind::Text),
            ("downstream_propagation", ScalarKind::Text),
        ],
        "fastq.filter_low_complexity" => {
            &[("entropy_threshold", ScalarKind::Number), ("polyx_threshold", ScalarKind::U32)]
        }
        "fastq.filter_reads" => &[
            ("threads", ScalarKind::U32),
            ("max_n", ScalarKind::U32),
            ("max_n_fraction", ScalarKind::Number),
            ("max_n_count", ScalarKind::U32),
            ("low_complexity_threshold", ScalarKind::Number),
            ("entropy_threshold", ScalarKind::Number),
            ("kmer_ref", ScalarKind::Text),
            ("polyx_policy", ScalarKind::Text),
        ],
        "fastq.infer_asvs" => &[
            ("denoising_method", ScalarKind::Text),
            ("pooling_mode", ScalarKind::Text),
            ("chimera_policy", ScalarKind::Text),
            ("threads", ScalarKind::U32),
        ],
        "fastq.merge_pairs" => &[
            ("threads", ScalarKind::U32),
            ("merge_overlap", ScalarKind::U32),
            ("min_len", ScalarKind::U32),
            ("unmerged_read_policy", ScalarKind::Text),
        ],
        "fastq.normalize_abundance" => &[("method", ScalarKind::Text)],
        "fastq.normalize_primers" => &[
            ("primer_set_id", ScalarKind::Text),
            ("marker_id", ScalarKind::Text),
            ("primer_fasta", ScalarKind::Text),
            ("orientation_policy", ScalarKind::Text),
            ("max_mismatch_rate", ScalarKind::Number),
            ("min_overlap_bp", ScalarKind::U32),
            ("strict_5p_anchor", ScalarKind::Boolean),
            ("allow_iupac_codes", ScalarKind::Boolean),
        ],
        "fastq.trim_terminal_damage" => &[
            ("threads", ScalarKind::U32),
            ("damage_mode", ScalarKind::Text),
            ("execution_policy", ScalarKind::Text),
            ("trim_5p_bases", ScalarKind::U32),
            ("trim_3p_bases", ScalarKind::U32),
        ],
        _ => &[],
    }
}
