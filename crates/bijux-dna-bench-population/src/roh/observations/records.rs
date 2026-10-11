//! Owner: bijux-dna-bench-population
//! Roh observations records.
use crate::roh::model::{LocalVcfRohSmokePerSampleSummary, LocalVcfRohSmokeSegment};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn parse_roh_segments(
    path: &Path,
    expected_samples: &BTreeSet<String>,
) -> Result<Vec<LocalVcfRohSmokeSegment>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("ROH segments table is empty: {}", path.display()))?;
    if header.trim() != "sample\tcontig\tstart\tend\tlength_bp\tn_sites" {
        bail!("ROH segments header drifted in {}: `{header}`", path.display());
    }
    let mut segments = Vec::<LocalVcfRohSmokeSegment>::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 6 {
            bail!("ROH segments row {} in {} must have 6 columns", index + 2, path.display());
        }
        let sample_id = columns[0].trim().to_string();
        if !expected_samples.contains(&sample_id) {
            bail!("ROH segments table reported unexpected sample `{sample_id}`");
        }
        let contig = columns[1].trim().to_string();
        if contig.is_empty() {
            bail!(
                "ROH segments row {} in {} must keep a non-empty contig",
                index + 2,
                path.display()
            );
        }
        let start = columns[2].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment start from row {} in {}", index + 2, path.display())
        })?;
        let end = columns[3].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment end from row {} in {}", index + 2, path.display())
        })?;
        let length = columns[4].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment length from row {} in {}", index + 2, path.display())
        })?;
        let variant_count = columns[5].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment variant count from row {} in {}", index + 2, path.display())
        })?;
        if end < start {
            bail!("ROH segments row {} in {} has end before start", index + 2, path.display());
        }
        if length == 0 {
            bail!("ROH segments row {} in {} must keep positive length", index + 2, path.display());
        }
        if variant_count == 0 {
            bail!(
                "ROH segments row {} in {} must keep positive variant count",
                index + 2,
                path.display()
            );
        }
        segments.push(LocalVcfRohSmokeSegment {
            sample_id,
            contig,
            start,
            end,
            length,
            variant_count,
        });
    }
    Ok(segments)
}

pub(crate) fn parse_roh_per_sample_summary(
    path: &Path,
    expected_samples: &[String],
) -> Result<Vec<LocalVcfRohSmokePerSampleSummary>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("ROH per-sample summary is empty: {}", path.display()))?;
    if header.trim() != "sample\tsegment_count\ttotal_length_bp\tmean_length_bp" {
        bail!("ROH per-sample header drifted in {}: `{header}`", path.display());
    }
    let mut rows = Vec::<LocalVcfRohSmokePerSampleSummary>::new();
    let mut seen = BTreeSet::<String>::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 4 {
            bail!("ROH per-sample row {} in {} must have 4 columns", index + 2, path.display());
        }
        let sample_id = columns[0].trim().to_string();
        if !expected_samples.iter().any(|expected| expected == &sample_id) {
            bail!("ROH per-sample summary reported unexpected sample `{sample_id}`");
        }
        if !seen.insert(sample_id.clone()) {
            bail!("ROH per-sample summary duplicated sample `{sample_id}`");
        }
        let segment_count = columns[1].trim().parse::<u64>().with_context(|| {
            format!("parse ROH segment count from row {} in {}", index + 2, path.display())
        })?;
        let total_length = columns[2].trim().parse::<u64>().with_context(|| {
            format!("parse ROH total length from row {} in {}", index + 2, path.display())
        })?;
        let mean_length = columns[3].trim().parse::<f64>().with_context(|| {
            format!("parse ROH mean length from row {} in {}", index + 2, path.display())
        })?;
        rows.push(LocalVcfRohSmokePerSampleSummary {
            sample_id,
            segment_count,
            total_length,
            mean_length,
        });
    }
    if rows.len() != expected_samples.len() {
        bail!(
            "ROH per-sample summary expected {} rows, found {}",
            expected_samples.len(),
            rows.len()
        );
    }
    let observed = rows.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    let expected = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    if observed != expected {
        bail!("ROH per-sample summary drifted from governed sample contract");
    }
    Ok(rows)
}
