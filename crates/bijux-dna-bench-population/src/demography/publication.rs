//! Owner: bijux-dna-bench-population
//! Demography publication.
use crate::execution::stage_result::path_relative_to_repo;
use crate::execution::stage_result::BenchStageResultOutputV1;
use std::path::Path;

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
