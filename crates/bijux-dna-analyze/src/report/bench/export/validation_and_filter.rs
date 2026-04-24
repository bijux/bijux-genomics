use super::super::{
    atomic_write_bytes, classify_raw_failure, derived_filter_metrics, gate_payload,
    rank_filter_tools, rank_validate_tools, sanity_flags_filter, sanity_flags_validate,
    semantic_filter, semantic_validate, BTreeMap, BenchmarkFailure, BenchmarkRecord,
    FastqDetectAdaptersMetrics, FastqFilterMetrics, FastqLowComplexityMetrics,
    FastqValidateMetrics, Path, RawFailure, Result,
};
use anyhow::Context;

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
    report.insert("sanity_flags", serde_json::to_value(sanity_flags_validate(records))?);
    let semantic: Vec<_> =
        records.iter().map(|record| semantic_validate(&record.metrics.metrics)).collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    if let Some(class) = qc_class {
        report.insert("qc_class", serde_json::to_value(class)?);
    }
    let rankings = rank_validate_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.validate_reads", &rankings);
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
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.detect_adapters", &BTreeMap::new());
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
    report.insert("sanity_flags", serde_json::to_value(sanity_flags_filter(records))?);
    let derived: Vec<_> = records.iter().map(derived_filter_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let semantic: Vec<_> =
        records.iter().map(|record| semantic_filter(&record.metrics.metrics)).collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let rankings = rank_filter_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.filter_reads", &rankings);
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
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::print_rank_explain("fastq.filter_low_complexity", &BTreeMap::new());
    }
    Ok(())
}
