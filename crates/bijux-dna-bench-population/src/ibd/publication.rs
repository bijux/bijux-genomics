//! Owner: bijux-dna-bench-population
//! Render normalized tabular population observations.
use std::fmt::Write as _;
// Ibd publication.
use crate::execution::stage_result::path_relative_to_repo;
use crate::execution::stage_result::BenchStageResultOutputV1;
use bijux_dna_bench_model::BenchmarkLocalVcfIbdSmokeRow as LocalVcfIbdSmokeRow;
use std::path::Path;

pub(crate) fn build_ibd_tsv(rows: &[LocalVcfIbdSmokeRow]) -> String {
    let mut rendered = String::from(
        "sample_a\tsample_b\tsegment_count\ttotal_length\toverlap_marker_count\tstatus\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{:.3}\t{}\t{}",
            row.sample_a,
            row.sample_b,
            row.segment_count,
            row.total_length,
            row.overlap_marker_count,
            row.status
        );
    }
    rendered
}

pub(crate) fn stage_result_output(
    repo_root: &Path,
    artifact_id: &str,
    declared_path: &str,
    realized_path: &Path,
    role: &str,
) -> BenchStageResultOutputV1 {
    BenchStageResultOutputV1 {
        artifact_id: artifact_id.to_string(),
        declared_path: declared_path.to_string(),
        realized_path: path_relative_to_repo(repo_root, realized_path),
        role: role.to_string(),
        optional: false,
        exists: true,
    }
}
