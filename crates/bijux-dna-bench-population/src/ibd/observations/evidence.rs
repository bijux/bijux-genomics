//! Owner: bijux-dna-bench-population
//! Ibd observations evidence.
use crate::ibd::model::ParsedIbdSegment;
use crate::ibd::observations::records::ordered_pair;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalVcfIbdSmokeRow as LocalVcfIbdSmokeRow;
use std::collections::BTreeMap;

pub(crate) fn summarize_ibd_rows(
    segments: &[ParsedIbdSegment],
    overlap_counts: &BTreeMap<(String, String), u64>,
) -> Result<Vec<LocalVcfIbdSmokeRow>> {
    let mut totals = BTreeMap::<(String, String), (u64, f64)>::new();
    for segment in segments {
        let key = ordered_pair(&segment.sample_a, &segment.sample_b);
        let entry = totals.entry(key).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += segment.length_cm;
    }
    let mut rows = Vec::<LocalVcfIbdSmokeRow>::new();
    for ((sample_a, sample_b), (segment_count, total_length)) in totals {
        let overlap_marker_count =
            overlap_counts.get(&(sample_a.clone(), sample_b.clone())).copied().ok_or_else(
                || anyhow!("IBD overlap-count derivation missed pair `{sample_a}` / `{sample_b}`"),
            )?;
        rows.push(LocalVcfIbdSmokeRow {
            sample_a,
            sample_b,
            segment_count,
            total_length,
            overlap_marker_count,
            status: "complete".to_string(),
        });
    }
    Ok(rows)
}
