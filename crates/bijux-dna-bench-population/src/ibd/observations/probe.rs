//! Owner: bijux-dna-bench-population
//! Require the ROH control to remain productive when IBD fails on sparse overlap.
use crate::ibd::execution::artifacts::read_json;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::path::Path;

pub(crate) fn validate_productive_roh_control(source_roh_summary_path: &Path) -> Result<u64> {
    let roh_summary = read_json(source_roh_summary_path)?;
    let unrelated_stage_segment_count = roh_summary
        .get("segment_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("sparse-overlap ROH summary missing `segment_count`"))?;
    if unrelated_stage_segment_count == 0 {
        bail!("sparse-overlap probe expected ROH to remain productive");
    }

    Ok(unrelated_stage_segment_count)
}
