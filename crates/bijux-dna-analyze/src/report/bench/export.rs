use super::*;

/// Write the validate benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_validate_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqValidateMetrics>],
    failures: &[RawFailure],
    qc_class: Option<&str>,
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_validate(records))?,
    );
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_validate(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    if let Some(class) = qc_class {
        report.insert("qc_class", serde_json::to_value(class)?);
    }
    let rankings = rank_validate_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.validate_reads", &rankings);
    }
    Ok(())
}

/// Write the detect-adapters benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_detect_adapters_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqDetectAdaptersMetrics>],
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.detect_adapters", &BTreeMap::new());
    }
    Ok(())
}

/// Write the filter benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_filter_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqFilterMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_filter(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_filter_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_filter(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let rankings = rank_filter_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.filter_reads", &rankings);
    }
    Ok(())
}

/// Write the low-complexity benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_filter_low_complexity_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqLowComplexityMetrics>],
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.filter_low_complexity", &BTreeMap::new());
    }
    Ok(())
}

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
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_merge(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_merge_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_merge_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.merge_pairs", &rankings);
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
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_correct(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_correct_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_correct_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.correct_errors", &rankings);
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
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_qc_post(records))?,
    );
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.report_qc", &BTreeMap::new());
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
        crate::decision::score::print_rank_explain("fastq.screen_taxonomy", &BTreeMap::new());
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
        crate::decision::score::print_rank_explain("fastq.deplete_host", &BTreeMap::new());
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
        crate::decision::score::print_rank_explain(
            "fastq.deplete_reference_contaminants",
            &BTreeMap::new(),
        );
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
        crate::decision::score::print_rank_explain("fastq.deplete_rrna", &BTreeMap::new());
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
        crate::decision::score::print_rank_explain("fastq.cluster_otus", &BTreeMap::new());
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
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_umi(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_umi_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_umi_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.extract_umis", &rankings);
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.index_reference", &BTreeMap::new());
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
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_stats(records))?,
    );
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_stats(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.profile_reads", &BTreeMap::new());
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain(
            "fastq.profile_overrepresented_sequences",
            &BTreeMap::new(),
        );
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.profile_read_lengths", &BTreeMap::new());
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.remove_duplicates", &BTreeMap::new());
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.remove_chimeras", &BTreeMap::new());
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.normalize_primers", &BTreeMap::new());
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.infer_asvs", &BTreeMap::new());
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
    atomic_write_bytes(&path, json.as_bytes()).map_err(anyhow::Error::from).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.normalize_abundance", &BTreeMap::new());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::{FastqDeltaMetrics, FastqTrimMetrics, FastqValidateMetrics};

    #[test]
    fn semantic_trim_generates_summary() {
        let metrics = FastqTrimMetrics {
            reads_in: 100,
            reads_out: 80,
            bases_in: 1000,
            bases_out: 800,
            pairs_in: None,
            pairs_out: None,
            mean_q_before: 30.0,
            mean_q_after: 31.5,
            delta_metrics: FastqDeltaMetrics {
                read_retention: 0.8,
                base_retention: 0.8,
                mean_q_delta: 1.5,
                gc_delta: 0.1,
            },
            paired_mode: None,
            adapter_policy: None,
            polyx_policy: None,
            n_policy: None,
            contaminant_policy: None,
            raw_backend_report_format: None,
            adapter_preset: Some("default".to_string()),
            adapter_bank_id: Some("bank.v1".to_string()),
            adapter_bank_hash: Some("sha256:abc".to_string()),
            adapter_overrides: None,
        };
        let summary = semantic_trim(&metrics);
        assert!(matches!(
            summary.integrity.reads_in.value,
            MetricValue::U64(100)
        ));
        assert!(matches!(
            summary.integrity.reads_out.value,
            MetricValue::U64(80)
        ));
        assert!(summary.quality_shift.is_some());
    }

    #[test]
    fn semantic_validate_generates_summary() {
        let metrics = FastqValidateMetrics {
            reads_in: 50,
            reads_out: 50,
            bases_in: 500,
            bases_out: 500,
            pairs_in: None,
            pairs_out: None,
            reads_total: 50,
            reads_valid: 45,
            reads_invalid: 5,
            mean_q: 32.0,
            validated_inputs: Some(2),
            validated_pairs: Some(22),
            pair_sync_checked: Some(true),
            pair_sync_pass: Some(false),
            pair_count_match: Some(false),
            strict_pass: Some(false),
            failure_class: Some("header_sync_mismatch".to_string()),
        };
        let summary = semantic_validate(&metrics);
        assert!(matches!(
            summary.integrity.reads_in.value,
            MetricValue::U64(50)
        ));
        assert!(matches!(
            summary.integrity.reads_out.value,
            MetricValue::U64(45)
        ));
        assert!(summary.quality_shift.is_none());
    }
}
