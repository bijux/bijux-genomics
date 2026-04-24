use super::super::{
    atomic_write_bytes, classify_raw_failure, derived_correct_metrics, derived_merge_metrics,
    derived_umi_metrics, gate_payload, rank_correct_tools, rank_merge_tools, rank_umi_tools,
    sanity_flags_correct, sanity_flags_merge, sanity_flags_qc_post, sanity_flags_stats,
    sanity_flags_umi, semantic_stats, BTreeMap, BenchmarkFailure, BenchmarkRecord, Context,
    FastqClusterOtusMetrics, FastqCorrectMetrics, FastqDepleteHostMetrics,
    FastqDepleteReferenceContaminantsMetrics, FastqDepleteRrnaMetrics, FastqIndexReferenceMetrics,
    FastqInferAsvsMetrics, FastqMergeMetrics, FastqNormalizeAbundanceMetrics,
    FastqNormalizePrimersMetrics, FastqOverrepresentedMetrics, FastqQcPostMetrics,
    FastqReadLengthMetrics, FastqScreenMetrics, FastqStatsMetrics, FastqUmiMetrics, Path,
    RawFailure, Result,
};
use crate::aggregate::{FastqChimeraMetrics, FastqDuplicateMetrics};

/// Write the merge benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_merge_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqMergeMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert("sanity_flags", serde_json::to_value(sanity_flags_merge(records))?);
    let derived: Vec<_> = records.iter().map(derived_merge_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_merge_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.merge_pairs", &rankings);
    }
    Ok(())
}

/// Write the correct benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_correct_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert("sanity_flags", serde_json::to_value(sanity_flags_correct(records))?);
    let derived: Vec<_> = records.iter().map(derived_correct_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_correct_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.correct_errors", &rankings);
    }
    Ok(())
}

/// Write the qc-post benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_qc_post_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqQcPostMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert("sanity_flags", serde_json::to_value(sanity_flags_qc_post(records))?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.report_qc", &BTreeMap::new());
    }
    Ok(())
}

/// Write the screen-taxonomy benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_screen_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqScreenMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let semantic: Vec<_> = records
        .iter()
        .map(|record| {
            serde_json::json!({
                "contamination_rate": record.metrics.metrics.contamination_rate,
                "has_summary_entries": record
                    .metrics
                    .metrics
                    .contamination_summary
                    .as_value()
                    .get("entries")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|entries| !entries.is_empty()),
            })
        })
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.screen_taxonomy", &BTreeMap::new());
    }
    Ok(())
}

/// Write the host depletion benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_deplete_host_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqDepleteHostMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let semantic: Vec<_> = records
        .iter()
        .map(|record| {
            serde_json::json!({
                "host_fraction_removed": record.metrics.metrics.host_fraction_removed,
                "reads_removed": record.metrics.metrics.reads_in.saturating_sub(record.metrics.metrics.reads_out),
            })
        })
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.deplete_host", &BTreeMap::new());
    }
    Ok(())
}

/// Write the reference contaminant depletion benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_deplete_reference_contaminants_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqDepleteReferenceContaminantsMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let semantic: Vec<_> = records
        .iter()
        .map(|record| {
            serde_json::json!({
                "contaminant_fraction_removed": record.metrics.metrics.contaminant_fraction_removed,
                "reads_removed": record.metrics.metrics.reads_in.saturating_sub(record.metrics.metrics.reads_out),
            })
        })
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.deplete_reference_contaminants", &BTreeMap::new());
    }
    Ok(())
}

/// Write the rRNA depletion benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_deplete_rrna_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqDepleteRrnaMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let semantic: Vec<_> = records
        .iter()
        .map(|record| {
            serde_json::json!({
                "rrna_fraction_removed": record.metrics.metrics.rrna_fraction_removed,
                "reads_removed": record.metrics.metrics.reads_in.saturating_sub(record.metrics.metrics.reads_out),
            })
        })
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.deplete_rrna", &BTreeMap::new());
    }
    Ok(())
}

/// Write the OTU clustering benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_cluster_otus_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqClusterOtusMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let semantic: Vec<_> = records
        .iter()
        .map(|record| {
            serde_json::json!({
                "otu_count": record.metrics.metrics.otu_count,
                "representative_count": record.metrics.metrics.representative_count,
            })
        })
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.cluster_otus", &BTreeMap::new());
    }
    Ok(())
}

/// Write the umi benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_umi_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqUmiMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert("sanity_flags", serde_json::to_value(sanity_flags_umi(records))?);
    let derived: Vec<_> = records.iter().map(derived_umi_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_umi_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.extract_umis", &rankings);
    }
    Ok(())
}

/// Write the index-reference benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_index_reference_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqIndexReferenceMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.index_reference", &BTreeMap::new());
    }
    Ok(())
}

/// Write the stats benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_stats_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqStatsMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert("sanity_flags", serde_json::to_value(sanity_flags_stats(records))?);
    let semantic: Vec<_> =
        records.iter().map(|record| semantic_stats(&record.metrics.metrics)).collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.profile_reads", &BTreeMap::new());
    }
    Ok(())
}

/// Write the overrepresented-sequence benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_overrepresented_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqOverrepresentedMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.profile_overrepresented_sequences", &BTreeMap::new());
    }
    Ok(())
}

/// Write the read-length benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writing fails.
pub fn write_read_lengths_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqReadLengthMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.profile_read_lengths", &BTreeMap::new());
    }
    Ok(())
}

/// Write the duplicate-removal benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writing fails.
pub fn write_duplicates_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqDuplicateMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.remove_duplicates", &BTreeMap::new());
    }
    Ok(())
}

/// Write the chimera-removal benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writing fails.
pub fn write_chimeras_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqChimeraMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.remove_chimeras", &BTreeMap::new());
    }
    Ok(())
}

/// Write the normalize-primers benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writing fails.
pub fn write_normalize_primers_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqNormalizePrimersMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.normalize_primers", &BTreeMap::new());
    }
    Ok(())
}

/// Write the infer-asvs benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writing fails.
pub fn write_infer_asvs_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqInferAsvsMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.infer_asvs", &BTreeMap::new());
    }
    Ok(())
}

/// Write the normalize-abundance benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writing fails.
pub fn write_normalize_abundance_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqNormalizeAbundanceMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.normalize_abundance", &BTreeMap::new());
    }
    Ok(())
}
