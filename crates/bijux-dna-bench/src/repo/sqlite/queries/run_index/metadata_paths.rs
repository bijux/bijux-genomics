//! Owner: bijux-dna-bench
//! Metadata path policy for run-index query results.

use bijux_dna_core::contract::RunIndexEntry;

use crate::repo::RunMetadata;

pub(super) fn resolve_run_metadata(
    artifacts_root: &std::path::Path,
    run: &RunIndexEntry,
) -> RunMetadata {
    let manifest_path = artifacts_root.join(run.run_id.as_str()).join("manifest.json");
    let metrics_path = artifacts_root.join(run.run_id.as_str()).join("metrics.json");
    RunMetadata { manifest_path, metrics_path }
}
