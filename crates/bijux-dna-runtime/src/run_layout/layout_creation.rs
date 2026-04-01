use std::path::Path;

use anyhow::{Context, Result};
use uuid::Uuid;

use super::RunLayout;

/// Create the canonical run layout under the base directory.
///
/// # Errors
/// Returns an error if directories cannot be created.
pub fn create_run_layout(base_dir: &Path) -> Result<(String, RunLayout)> {
    let run_id = Uuid::new_v4().to_string();
    let run_dir = bijux_dna_infra::run_layout_paths(base_dir, &run_id).run_dir;
    let stages_dir = run_dir.join("stages");
    let summary_dir = run_dir.join("summary");
    bijux_dna_infra::ensure_dir(&stages_dir).context("create run stages dir")?;
    bijux_dna_infra::ensure_dir(&summary_dir).context("create run summary dir")?;
    let layout = RunLayout {
        assessment_path: run_dir.join("input_assessment.json"),
        manifest_path: run_dir.join("execution_manifest.json"),
        environment_path: run_dir.join("environment.json"),
        metadata_path: run_dir.join("run_metadata.json"),
        events_path: run_dir.join("events.jsonl"),
        stages_dir,
        summary_dir,
        run_dir,
    };
    Ok((run_id, layout))
}
