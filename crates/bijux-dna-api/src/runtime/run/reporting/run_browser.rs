use super::{Path, Result};
use crate::request_args::{RunBrowserRequestV1, RunBrowserResponseV1, RunBrowserRowV1};
use std::path::PathBuf;

/// Build the run browser data model from a directory that contains run subdirectories.
///
/// # Errors
/// Returns an error if directory discovery fails.
pub fn browse_runs(request: &RunBrowserRequestV1) -> Result<RunBrowserResponseV1> {
    let mut rows = collect_rows(&request.runs_root)?;
    rows.retain(|row| row_matches_filter(row, &request.filter));
    rows.sort_by(|left, right| {
        left
            .run_id
            .cmp(&right.run_id)
            .then_with(|| left.run_dir.cmp(&right.run_dir))
    });
    let default_page_size = 50;
    let page_size = if request.page_size == 0 {
        default_page_size
    } else {
        request.page_size
    };
    let offset = request
        .page_token
        .as_deref()
        .and_then(|token| token.parse::<usize>().ok())
        .unwrap_or(0)
        .min(rows.len());
    let end = offset.saturating_add(page_size).min(rows.len());
    let next_page_token = (end < rows.len()).then(|| end.to_string());
    let page_rows = rows[offset..end].to_vec();

    Ok(RunBrowserResponseV1 {
        schema_version: "bijux.run_browser.v1".to_string(),
        runs_root: request.runs_root.clone(),
        total_rows: rows.len(),
        page_size,
        next_page_token,
        rows: page_rows,
    })
}

fn collect_rows(runs_root: &Path) -> Result<Vec<RunBrowserRowV1>> {
    let mut run_dirs = Vec::<PathBuf>::new();
    if runs_root.join("run_manifest.json").exists() {
        run_dirs.push(runs_root.to_path_buf());
    }
    for entry in std::fs::read_dir(runs_root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.join("run_manifest.json").exists() {
            run_dirs.push(path);
        }
    }

    let mut rows = Vec::with_capacity(run_dirs.len());
    for run_dir in run_dirs {
        rows.push(row_for_run_dir(&run_dir));
    }
    Ok(rows)
}

fn row_for_run_dir(run_dir: &Path) -> RunBrowserRowV1 {
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(run_dir.to_path_buf());
    let manifest = read_json(&layout.manifest_path);
    let run_state = layout
        .run_state_path
        .exists()
        .then(|| std::fs::read_to_string(&layout.run_state_path).ok())
        .flatten()
        .and_then(|raw| serde_json::from_str::<bijux_dna_runtime::run_layout::RunStateV1>(&raw).ok());
    let artifact_count = layout
        .artifact_inventory_path
        .exists()
        .then(|| std::fs::read(&layout.artifact_inventory_path).ok())
        .flatten()
        .and_then(|raw| serde_json::from_slice::<bijux_dna_runtime::run_layout::ArtifactInventoryV1>(&raw).ok())
        .map_or_else(
            || {
                manifest
                    .as_ref()
                    .and_then(|value| value.get("output_artifacts"))
                    .and_then(serde_json::Value::as_array)
                    .map_or(0, Vec::len)
            },
            |inventory| inventory.artifacts.len(),
        );

    let run_id = manifest
        .as_ref()
        .and_then(|value| value.get("run_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            run_state
                .as_ref()
                .map(|state| state.run_id.clone())
        })
        .unwrap_or_else(|| {
            run_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("run")
                .to_string()
        });

    let has_failures = manifest
        .as_ref()
        .and_then(|value| value.get("failures"))
        .and_then(serde_json::Value::as_array)
        .is_some_and(|failures| !failures.is_empty())
        || layout.failure_path.exists();

    RunBrowserRowV1 {
        run_id,
        run_dir: run_dir.to_path_buf(),
        profile_id: manifest
            .as_ref()
            .and_then(|value| value.get("profile_id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        pipeline_id: manifest
            .as_ref()
            .and_then(|value| value.get("pipeline_id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        correlation_id: manifest
            .as_ref()
            .and_then(|value| value.get("correlation_id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        mode: run_state.as_ref().map(|state| state.mode),
        state: run_state.as_ref().map(|state| state.state),
        has_failures,
        has_evidence_bundle: layout.evidence_bundle_path.exists(),
        artifact_count,
    }
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
    path.exists()
        .then(|| std::fs::read(path).ok())
        .flatten()
        .and_then(|raw| serde_json::from_slice::<serde_json::Value>(&raw).ok())
}

fn row_matches_filter(
    row: &RunBrowserRowV1,
    filter: &crate::request_args::RunBrowserFilterV1,
) -> bool {
    if filter
        .run_id_prefix
        .as_deref()
        .is_some_and(|prefix| !row.run_id.starts_with(prefix))
    {
        return false;
    }
    if filter
        .profile_id
        .as_deref()
        .is_some_and(|profile| row.profile_id.as_deref() != Some(profile))
    {
        return false;
    }
    if filter
        .pipeline_id
        .as_deref()
        .is_some_and(|pipeline| row.pipeline_id.as_deref() != Some(pipeline))
    {
        return false;
    }
    if filter
        .correlation_id
        .as_deref()
        .is_some_and(|correlation| row.correlation_id.as_deref() != Some(correlation))
    {
        return false;
    }
    if filter.state.is_some_and(|state| row.state != Some(state)) {
        return false;
    }
    if filter.mode.is_some_and(|mode| row.mode != Some(mode)) {
        return false;
    }
    if filter
        .has_failures
        .is_some_and(|has_failures| row.has_failures != has_failures)
    {
        return false;
    }
    true
}
