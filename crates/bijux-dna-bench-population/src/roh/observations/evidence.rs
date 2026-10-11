//! Owner: bijux-dna-bench-population
//! Roh observations evidence.
use crate::roh::model::{LocalVcfRohSmokePerSampleSummary, LocalVcfRohSmokeSegment};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::collections::BTreeMap;

pub(crate) fn validate_roh_summary_contract(
    source_roh_report: &serde_json::Value,
    segments: &[LocalVcfRohSmokeSegment],
    per_sample_summary: &[LocalVcfRohSmokePerSampleSummary],
) -> Result<()> {
    let segment_count =
        u64::try_from(segments.len()).map_err(|_| anyhow!("segment count overflow"))?;
    let total_length = segments.iter().map(|segment| segment.length).sum::<u64>();
    let reported_segment_count = source_roh_report
        .get("segment_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("ROH source report missing `segment_count`"))?;
    let reported_total_length = source_roh_report
        .get("total_length_bp")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("ROH source report missing `total_length_bp`"))?;
    if reported_segment_count != segment_count {
        bail!(
            "ROH source report segment count drifted: expected {segment_count}, found {reported_segment_count}"
        );
    }
    if reported_total_length != total_length {
        bail!(
            "ROH source report total length drifted: expected {total_length}, found {reported_total_length}"
        );
    }

    let mut derived = BTreeMap::<String, (u64, u64)>::new();
    for segment in segments {
        let entry = derived.entry(segment.sample_id.clone()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += segment.length;
    }
    for summary in per_sample_summary {
        let (segment_count, total_length) =
            derived.get(&summary.sample_id).copied().unwrap_or((0, 0));
        if summary.segment_count != segment_count {
            bail!(
                "ROH per-sample summary segment count drifted for `{}`: expected {}, found {}",
                summary.sample_id,
                segment_count,
                summary.segment_count
            );
        }
        if summary.total_length != total_length {
            bail!(
                "ROH per-sample summary total length drifted for `{}`: expected {}, found {}",
                summary.sample_id,
                total_length,
                summary.total_length
            );
        }
    }
    Ok(())
}
