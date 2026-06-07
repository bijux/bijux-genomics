use anyhow::{anyhow, Context, Result};

use super::{
    parse_seqkit_stats, AdapterRemovalToolMetricsV1, FastpToolMetricsV1, FastqcToolMetricsV1,
    MultiqcToolMetricsV1, SamtoolsFlagstatMetricsV1, SeqkitToolMetricsV1,
};

/// # Errors
/// Returns an error if fastp JSON cannot be parsed.
pub fn parse_fastp_metrics(report_json: &str) -> Result<FastpToolMetricsV1> {
    let parsed: serde_json::Value =
        serde_json::from_str(report_json).context("parse fastp json")?;
    let filtering =
        parsed.get("filtering_result").ok_or_else(|| anyhow!("fastp filtering_result missing"))?;
    Ok(FastpToolMetricsV1 {
        schema_version: "bijux.fastp.metrics.v1".to_string(),
        passed_filter_reads: filtering
            .get("passed_filter_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        low_quality_reads: filtering
            .get("low_quality_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        too_many_n_reads: filtering
            .get("too_many_N_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        too_short_reads: filtering
            .get("too_short_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
    })
}

/// Parse `AdapterRemoval` output into canonical merge metrics.
///
/// # Errors
/// Returns an error if required summary lines are missing or malformed.
pub fn parse_adapterremoval_metrics(stdout: &str) -> Result<AdapterRemovalToolMetricsV1> {
    let pairs_processed =
        parse_required_prefix_u64(stdout, "Total number of read pairs", "adapterremoval")?;
    let pairs_merged =
        parse_required_prefix_u64(stdout, "Number of fully overlapping pairs", "adapterremoval")?;
    let merge_rate = if pairs_processed > 0 {
        u64_to_f64(pairs_merged) / u64_to_f64(pairs_processed)
    } else {
        0.0
    };
    Ok(AdapterRemovalToolMetricsV1 {
        schema_version: "bijux.adapterremoval.metrics.v1".to_string(),
        pairs_processed,
        pairs_merged,
        merge_rate,
    })
}

/// # Errors
/// Returns an error if seqkit output cannot be parsed.
pub fn parse_seqkit_tool_metrics(output: &str) -> Result<SeqkitToolMetricsV1> {
    let parsed = parse_seqkit_stats(output)?;
    Ok(SeqkitToolMetricsV1 {
        schema_version: "bijux.seqkit.metrics.v1".to_string(),
        reads: parsed.reads,
        bases: parsed.bases,
        mean_q: Some(parsed.mean_q),
        gc_percent: Some(parsed.gc_percent),
    })
}

/// Parse `samtools flagstat` output into canonical alignment metrics.
///
/// # Errors
/// Returns an error if required summary lines are missing or malformed.
pub fn parse_samtools_flagstat_metrics(stdout: &str) -> Result<SamtoolsFlagstatMetricsV1> {
    let total_reads = parse_required_prefix_u64(stdout, "in total", "samtools flagstat")?;
    let mapped_reads = parse_required_prefix_u64(stdout, "mapped (", "samtools flagstat")?;
    let mapped_rate =
        if total_reads > 0 { u64_to_f64(mapped_reads) / u64_to_f64(total_reads) } else { 0.0 };
    Ok(SamtoolsFlagstatMetricsV1 {
        schema_version: "bijux.samtools.flagstat.v1".to_string(),
        total_reads,
        mapped_reads,
        mapped_rate,
    })
}

/// Parse `FastQC` summary text into canonical summary metrics.
///
/// # Errors
/// Returns an error if required summary lines are missing or malformed.
pub fn parse_fastqc_summary_metrics(summary_txt: &str) -> Result<FastqcToolMetricsV1> {
    let mut total_sequences = None;
    let mut gc_percent = None;
    for line in summary_txt.lines() {
        if let Some((key, value)) = line.split_once('\t') {
            if key.trim() == "Total Sequences" {
                total_sequences =
                    Some(value.trim().parse::<u64>().context("parse fastqc total sequences")?);
            } else if key.trim() == "%GC" {
                gc_percent = Some(value.trim().parse::<f64>().context("parse fastqc %GC")?);
            }
        }
    }
    Ok(FastqcToolMetricsV1 {
        schema_version: "bijux.fastqc.metrics.v1".to_string(),
        total_sequences: total_sequences
            .ok_or_else(|| anyhow!("fastqc total sequences missing"))?,
        gc_percent: gc_percent.ok_or_else(|| anyhow!("fastqc %GC missing"))?,
    })
}

/// # Errors
/// Returns an error if multiqc general stats JSON cannot be parsed.
pub fn parse_multiqc_general_stats_metrics(raw_json: &str) -> Result<MultiqcToolMetricsV1> {
    let parsed: serde_json::Value =
        serde_json::from_str(raw_json).context("parse multiqc general stats json")?;
    let sample_count = parsed.as_object().map_or(0, serde_json::Map::len) as u64;
    let module_count = parsed
        .as_object()
        .and_then(|obj| obj.values().next())
        .and_then(serde_json::Value::as_object)
        .map_or(0, serde_json::Map::len) as u64;
    Ok(MultiqcToolMetricsV1 {
        schema_version: "bijux.multiqc.metrics.v1".to_string(),
        sample_count,
        module_count,
    })
}

pub(super) fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(f64::MAX)
}

pub(super) fn parse_report_u64_field(raw: &str, field: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| {
            value.get(field).and_then(serde_json::Value::as_u64).or_else(|| {
                value
                    .as_object()
                    .and_then(|obj| obj.get(field))
                    .and_then(serde_json::Value::as_str)
                    .and_then(|s| s.parse::<u64>().ok())
            })
        })
        .or_else(|| parse_kv_u64_field(raw, field))
}

pub(super) fn parse_kv_u64_field(raw: &str, field: &str) -> Option<u64> {
    raw.lines().filter_map(|line| line.split_once('=')).find_map(|(k, v)| {
        if k.trim() == field {
            v.trim().parse::<u64>().ok()
        } else {
            None
        }
    })
}

pub(super) fn parse_prefix_u64(raw: &str, marker: &str) -> u64 {
    raw.lines()
        .find_map(|line| {
            if line.contains(marker) || line.starts_with(marker) {
                if let Some(value) =
                    line.split_whitespace().next().and_then(|value| value.parse::<u64>().ok())
                {
                    return Some(value);
                }
                return line
                    .split_once(':')
                    .and_then(|(_, value)| value.split_whitespace().next())
                    .and_then(|value| value.parse::<u64>().ok());
            }
            None
        })
        .unwrap_or(0)
}

fn parse_required_prefix_u64(raw: &str, marker: &str, parser_name: &str) -> Result<u64> {
    raw.lines()
        .find_map(|line| {
            if line.contains(marker) || line.starts_with(marker) {
                if let Some(value) =
                    line.split_whitespace().next().and_then(|value| value.parse::<u64>().ok())
                {
                    return Some(Ok(value));
                }
                return Some(
                    line.split_once(':')
                        .and_then(|(_, value)| value.split_whitespace().next())
                        .ok_or_else(|| anyhow!("{parser_name} line for `{marker}` is malformed"))
                        .and_then(|value| {
                            value.parse::<u64>().with_context(|| {
                                format!("{parser_name} line for `{marker}` does not start with an integer")
                            })
                        }),
                );
            }
            None
        })
        .unwrap_or_else(|| Err(anyhow!("{parser_name} line for `{marker}` is missing")))
}
