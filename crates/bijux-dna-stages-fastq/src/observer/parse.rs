//! Owner: bijux-dna-stages-fastq
//! Observer parsers for tool stdout and report artifacts.
//! Supported formats must be deterministic and stable across versions.

use anyhow::{anyhow, Context, Result};
use tracing::warn;

use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_domain_fastq::metrics::{
    AdapterRemovalToolMetricsV1, FastpToolMetricsV1, FastqcToolMetricsV1, MultiqcToolMetricsV1,
    SamtoolsFlagstatMetricsV1, SeqkitToolMetricsV1,
};

/// # Errors
/// Returns an error if stdout cannot be parsed.
pub fn parse_seqkit_stats(output: &str) -> Result<SeqkitMetrics> {
    let mut lines = output.lines();
    let header = lines.next().ok_or_else(|| anyhow!("empty seqkit output"))?;
    let data = lines.next().ok_or_else(|| anyhow!("missing seqkit data"))?;
    let header_fields: Vec<&str> = header.split('\t').collect();
    let data_fields: Vec<&str> = data.split('\t').collect();
    if header_fields.len() != data_fields.len() {
        return Err(anyhow!("seqkit header/data column mismatch"));
    }
    let col = |name: &str| -> Result<&str> {
        let idx = header_fields
            .iter()
            .position(|field| field == &name)
            .ok_or_else(|| anyhow!("seqkit column missing: {name}"))?;
        data_fields
            .get(idx)
            .copied()
            .ok_or_else(|| anyhow!("seqkit data missing for {name}"))
    };
    let reads: u64 = col("num_seqs")?.parse().context("parse reads")?;
    let bases: u64 = col("sum_len")?.parse().context("parse bases")?;
    let mean_q = if header_fields.iter().any(|field| field == &"avg_qual") {
        col("avg_qual")?.parse().context("parse mean_q")?
    } else if header_fields.iter().any(|field| field == &"mean_qual") {
        col("mean_qual")?.parse().context("parse mean_q")?
    } else {
        warn!("seqkit avg_qual/mean_qual missing; defaulting mean_q to 0.0");
        0.0
    };
    let gc_percent = if let Some((idx, _)) = header_fields
        .iter()
        .enumerate()
        .find(|(_, field)| field.to_lowercase().starts_with("gc"))
    {
        data_fields
            .get(idx)
            .ok_or_else(|| anyhow!("seqkit data missing for gc"))?
            .parse()
            .context("parse gc_percent")?
    } else {
        warn!("seqkit gc column missing; defaulting gc_percent to 0.0");
        0.0
    };
    Ok(SeqkitMetrics {
        reads,
        bases,
        mean_q,
        gc_percent,
    })
}

/// # Errors
/// Returns an error if stdout cannot be parsed.
pub fn parse_length_histogram(output: &str) -> Result<Vec<(u64, u64)>> {
    let mut counts: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();
    for line in output.lines() {
        let mut parts = line.split('\t');
        let _id = parts.next();
        let len = parts
            .next()
            .ok_or_else(|| anyhow!("seqkit fx2tab length missing"))?;
        let cleaned: String = len.chars().filter(char::is_ascii_digit).collect();
        if cleaned.is_empty() {
            continue;
        }
        let length: u64 = cleaned.parse().context("parse length")?;
        *counts.entry(length).or_insert(0) += 1;
    }
    Ok(counts.into_iter().collect())
}

/// # Errors
/// Returns an error if stdout cannot be parsed.
pub fn parse_fastqvalidator_count(stdout: &str) -> Result<u64> {
    let line = stdout
        .lines()
        .find(|line| line.to_lowercase().contains("total reads"))
        .ok_or_else(|| anyhow!("fastqvalidator total reads line missing"))?;
    let count = line
        .split_once(':')
        .ok_or_else(|| anyhow!("fastqvalidator total reads format missing ':'"))?
        .1
        .trim();
    Ok(count.parse::<u64>()?)
}

/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_deduplicate_report(report_json: &str) -> Result<(u64, u64)> {
    let reads_in = parse_report_u64_field(report_json, "reads_in")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_in"))?;
    let reads_out = parse_report_u64_field(report_json, "reads_out")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_out"))?;
    Ok((reads_in, reads_out))
}

/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_low_complexity_report(report_json: &str) -> Result<u64> {
    parse_report_u64_field(report_json, "reads_removed_low_complexity")
        .ok_or_else(|| anyhow!("low-complexity report missing reads_removed_low_complexity"))
}

/// # Errors
/// Returns an error if fastp JSON cannot be parsed.
#[allow(dead_code)]
pub fn parse_fastp_metrics(report_json: &str) -> Result<FastpToolMetricsV1> {
    let parsed: serde_json::Value =
        serde_json::from_str(report_json).context("parse fastp json")?;
    let filtering = parsed
        .get("filtering_result")
        .ok_or_else(|| anyhow!("fastp filtering_result missing"))?;
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

/// # Errors
/// Returns an error if AdapterRemoval output cannot be parsed.
#[allow(dead_code)]
pub fn parse_adapterremoval_metrics(stdout: &str) -> Result<AdapterRemovalToolMetricsV1> {
    let pairs_processed = parse_prefix_u64(stdout, "Total number of read pairs");
    let pairs_merged = parse_prefix_u64(stdout, "Number of fully overlapping pairs");
    let merge_rate = if pairs_processed > 0 {
        pairs_merged as f64 / pairs_processed as f64
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
#[allow(dead_code)]
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

/// # Errors
/// Returns an error if samtools flagstat output cannot be parsed.
#[allow(dead_code)]
pub fn parse_samtools_flagstat_metrics(stdout: &str) -> Result<SamtoolsFlagstatMetricsV1> {
    let total_reads = parse_prefix_u64(stdout, "in total");
    let mapped_reads = parse_prefix_u64(stdout, "mapped (");
    let mapped_rate = if total_reads > 0 {
        mapped_reads as f64 / total_reads as f64
    } else {
        0.0
    };
    Ok(SamtoolsFlagstatMetricsV1 {
        schema_version: "bijux.samtools.flagstat.v1".to_string(),
        total_reads,
        mapped_reads,
        mapped_rate,
    })
}

/// # Errors
/// Returns an error if fastqc summary cannot be parsed.
#[allow(dead_code)]
pub fn parse_fastqc_summary_metrics(summary_txt: &str) -> Result<FastqcToolMetricsV1> {
    let mut total_sequences = 0;
    let mut gc_percent = 0.0;
    for line in summary_txt.lines() {
        if let Some((key, value)) = line.split_once('\t') {
            if key.trim() == "Total Sequences" {
                total_sequences = value.trim().parse::<u64>().unwrap_or(0);
            } else if key.trim() == "%GC" {
                gc_percent = value.trim().parse::<f64>().unwrap_or(0.0);
            }
        }
    }
    Ok(FastqcToolMetricsV1 {
        schema_version: "bijux.fastqc.metrics.v1".to_string(),
        total_sequences,
        gc_percent,
    })
}

/// # Errors
/// Returns an error if multiqc general stats JSON cannot be parsed.
#[allow(dead_code)]
pub fn parse_multiqc_general_stats_metrics(raw_json: &str) -> Result<MultiqcToolMetricsV1> {
    let parsed: serde_json::Value =
        serde_json::from_str(raw_json).context("parse multiqc general stats json")?;
    let sample_count = parsed.as_object().map_or(0, |obj| obj.len()) as u64;
    let module_count = parsed
        .as_object()
        .and_then(|obj| obj.values().next())
        .and_then(serde_json::Value::as_object)
        .map_or(0, |obj| obj.len()) as u64;
    Ok(MultiqcToolMetricsV1 {
        schema_version: "bijux.multiqc.metrics.v1".to_string(),
        sample_count,
        module_count,
    })
}

fn parse_report_u64_field(raw: &str, field: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| {
            value
                .get(field)
                .and_then(serde_json::Value::as_u64)
                .or_else(|| {
                    value
                        .as_object()
                        .and_then(|obj| obj.get(field))
                        .and_then(serde_json::Value::as_str)
                        .and_then(|s| s.parse::<u64>().ok())
                })
        })
        .or_else(|| parse_kv_u64_field(raw, field))
}

fn parse_kv_u64_field(raw: &str, field: &str) -> Option<u64> {
    raw.lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(k, v)| {
            if k.trim() == field {
                v.trim().parse::<u64>().ok()
            } else {
                None
            }
        })
}

#[allow(dead_code)]
fn parse_prefix_u64(raw: &str, marker: &str) -> u64 {
    raw.lines()
        .find_map(|line| {
            if line.contains(marker) || line.starts_with(marker) {
                if let Some(value) = line
                    .split_whitespace()
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
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

#[cfg(test)]
mod tests {
    use super::{
        parse_adapterremoval_metrics, parse_deduplicate_report, parse_fastp_metrics,
        parse_fastqc_summary_metrics, parse_fastqvalidator_count, parse_length_histogram,
        parse_low_complexity_report, parse_multiqc_general_stats_metrics,
        parse_samtools_flagstat_metrics, parse_seqkit_stats, parse_seqkit_tool_metrics,
    };
    use anyhow::Result;

    #[test]
    fn parse_fastqvalidator_count_parses_fixture() -> Result<()> {
        let stdout =
            include_str!("../../tests/fixtures/fastqvalidator/default/fastqvalidator_v1.txt");
        let count = parse_fastqvalidator_count(stdout)?;
        assert_eq!(count, 12345);
        Ok(())
    }

    #[test]
    fn parse_fastqvalidator_count_rejects_missing_marker() {
        let stdout = "fastqvalidator output without total reads";
        assert!(parse_fastqvalidator_count(stdout).is_err());
    }

    #[test]
    fn parse_seqkit_stats_parses_fixture() -> Result<()> {
        let stdout = include_str!("../../tests/fixtures/seqkit/default/seqkit_stats_v1.txt");
        let metrics = parse_seqkit_stats(stdout)?;
        assert_eq!(metrics.reads, 1000);
        assert_eq!(metrics.bases, 100_000);
        Ok(())
    }

    #[test]
    fn parse_length_histogram_parses_fixture() -> Result<()> {
        let stdout = "readA\t100\nreadB\t100\nreadC\t50\n";
        let metrics = parse_length_histogram(stdout)?;
        assert_eq!(metrics.len(), 2);
        Ok(())
    }

    #[test]
    fn parse_deduplicate_report_parses_fixture() -> Result<()> {
        let raw =
            include_str!("../../tests/fixtures/deduplicate/default/deduplicate_report_v1.json");
        let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
        assert_eq!(reads_in, 1000);
        assert_eq!(reads_out, 820);
        Ok(())
    }

    #[test]
    fn parse_low_complexity_report_parses_fixture() -> Result<()> {
        let raw = include_str!(
            "../../tests/fixtures/low_complexity/default/low_complexity_report_v1.json"
        );
        let removed = parse_low_complexity_report(raw)?;
        assert_eq!(removed, 137);
        Ok(())
    }

    #[test]
    fn parse_deduplicate_report_parses_key_value_fixture() -> Result<()> {
        let raw = include_str!(
            "../../tests/fixtures/stage_output_bank/default/fastq.deduplicate.fastuniq.txt"
        );
        let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
        assert_eq!(reads_in, 1000);
        assert_eq!(reads_out, 820);
        Ok(())
    }

    #[test]
    fn parse_low_complexity_report_parses_key_value_fixture() -> Result<()> {
        let raw = include_str!(
            "../../tests/fixtures/stage_output_bank/default/fastq.low_complexity.bbduk.txt"
        );
        let removed = parse_low_complexity_report(raw)?;
        assert_eq!(removed, 137);
        Ok(())
    }

    #[test]
    fn parse_fastp_metrics_fixture() -> Result<()> {
        let raw = include_str!("../../tests/fixtures/tool_metrics/default/fastp.json");
        let parsed = parse_fastp_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.fastp.metrics.v1");
        assert_eq!(parsed.passed_filter_reads, 960);
        assert_eq!(parsed.too_short_reads, 12);
        Ok(())
    }

    #[test]
    fn parse_adapterremoval_metrics_fixture() -> Result<()> {
        let raw = include_str!("../../tests/fixtures/tool_metrics/default/adapterremoval.txt");
        let parsed = parse_adapterremoval_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.adapterremoval.metrics.v1");
        assert_eq!(parsed.pairs_processed, 1000);
        assert_eq!(parsed.pairs_merged, 640);
        Ok(())
    }

    #[test]
    fn parse_seqkit_tool_metrics_fixture() -> Result<()> {
        let raw = include_str!("../../tests/fixtures/seqkit/default/seqkit_stats_v1.txt");
        let parsed = parse_seqkit_tool_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.seqkit.metrics.v1");
        assert_eq!(parsed.reads, 1000);
        Ok(())
    }

    #[test]
    fn parse_samtools_flagstat_fixture() -> Result<()> {
        let raw = include_str!("../../tests/fixtures/tool_metrics/default/samtools_flagstat.txt");
        let parsed = parse_samtools_flagstat_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.samtools.flagstat.v1");
        assert_eq!(parsed.total_reads, 1000);
        assert_eq!(parsed.mapped_reads, 900);
        Ok(())
    }

    #[test]
    fn parse_fastqc_summary_fixture() -> Result<()> {
        let raw = include_str!("../../tests/fixtures/tool_metrics/default/fastqc_summary.txt");
        let parsed = parse_fastqc_summary_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.fastqc.metrics.v1");
        assert_eq!(parsed.total_sequences, 1000);
        assert_eq!(parsed.gc_percent, 42.0);
        Ok(())
    }

    #[test]
    fn parse_multiqc_general_stats_fixture() -> Result<()> {
        let raw =
            include_str!("../../tests/fixtures/tool_metrics/default/multiqc_general_stats.json");
        let parsed = parse_multiqc_general_stats_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.multiqc.metrics.v1");
        assert_eq!(parsed.sample_count, 2);
        assert_eq!(parsed.module_count, 2);
        Ok(())
    }
}
