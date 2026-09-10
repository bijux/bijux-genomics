//! Owner: bijux-dna-bench-population
//! Render normalized tabular population observations.
use std::fmt::Write as _;
// Roh publication.
use crate::roh::model::LocalVcfRohSmokeSegment;

pub(crate) fn build_roh_tsv(segments: &[LocalVcfRohSmokeSegment]) -> String {
    let mut rendered = String::from("sample_id\tcontig\tstart\tend\tlength\tvariant_count\n");
    for segment in segments {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}",
            segment.sample_id,
            segment.contig,
            segment.start,
            segment.end,
            segment.length,
            segment.variant_count
        );
    }
    rendered
}
