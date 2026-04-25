use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{anyhow, Context, Result};
use regex::Regex;

use super::models::{StageRunRootCandidate, StageRunRootSelection};
use crate::commands::benchmark_workspace::{
    benchmark_stage_run_relative_root, BenchmarkWorkspaceConfig,
};

pub(super) fn publication_stage_docs_root(
    docs_root: &Path,
    stage_id: &str,
    corpus_id: &str,
) -> PathBuf {
    docs_root.join(stage_id).join(corpus_id)
}

pub(super) fn publication_artifact_file_name(corpus_id: &str, suffix: &str) -> String {
    format!("{corpus_id}-{suffix}")
}

pub(super) fn publication_method_file_name(corpus_id: &str) -> String {
    format!("{corpus_id}-method.md")
}

pub(super) fn load_json_value(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

pub(super) fn relative_to_docs_root(path: &Path, docs_root: &Path) -> String {
    let repo_root = docs_root
        .parent()
        .and_then(Path::parent)
        .unwrap_or(docs_root.parent().unwrap_or(docs_root));
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

pub(super) fn relative_to_repo_root(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

pub(super) fn load_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let csv_text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = csv_text.lines();
    let Some(header_line) = lines.next() else {
        return Ok(Vec::new());
    };
    let headers = header_line.split(',').map(|value| value.trim().to_string()).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values = line.split(',').map(str::trim).collect::<Vec<_>>();
        let mut csv_row = BTreeMap::new();
        for (header, value) in headers.iter().zip(values.iter()) {
            csv_row.insert(header.clone(), (*value).to_string());
        }
        rows.push(csv_row);
    }
    Ok(rows)
}

pub(super) fn csv_value(row: &BTreeMap<String, String>, key: &str) -> String {
    row.get(key).map_or_else(|| "missing".to_string(), |value| value.trim().to_string())
}

pub(super) fn csv_required_value(row: &BTreeMap<String, String>, key: &str) -> Option<String> {
    let value = csv_value(row, key);
    (value != "missing" && !value.is_empty()).then_some(value)
}

pub(super) fn csv_report_value(row: &BTreeMap<String, String>, key: &str) -> String {
    csv_required_value(row, key).unwrap_or_else(|| "missing".to_string())
}

pub(super) fn sort_count_map(value: Option<&serde_json::Value>) -> Result<BTreeMap<String, usize>> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let object = value.as_object().ok_or_else(|| anyhow!("count map must be a JSON object"))?;
    object
        .iter()
        .map(|(key, value)| {
            let count = value
                .as_u64()
                .ok_or_else(|| anyhow!("count map entry `{key}` must be an unsigned integer"))?;
            let count = usize::try_from(count)
                .map_err(|_| anyhow!("count map entry `{key}` exceeds usize"))?;
            Ok((key.clone(), count))
        })
        .collect()
}

pub(super) fn summary_corpus_id(summary_corpus_root: &Path) -> Result<String> {
    summary_corpus_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("summary corpus_root must end with a corpus directory name"))
}

pub(super) fn configured_stage_run_roots(
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    stage_id: &str,
) -> Result<Vec<StageRunRootCandidate>> {
    Ok(vec![
        StageRunRootCandidate {
            path: workspace_local_cache_mirror_root(workspace)?.join(
                benchmark_stage_run_relative_root(workspace, "local-cache", corpus_id, stage_id)?,
            ),
        },
        StageRunRootCandidate {
            path: workspace_local_results_root(workspace)?.join(benchmark_stage_run_relative_root(
                workspace,
                "local-archive",
                corpus_id,
                stage_id,
            )?),
        },
    ])
}

pub(super) fn unique_existing_run_roots(
    reported_run_root: &Path,
    configured_roots: &[StageRunRootCandidate],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for root in std::iter::once(reported_run_root)
        .chain(configured_roots.iter().map(|candidate| candidate.path.as_path()))
    {
        if !root.is_dir() || roots.iter().any(|existing| existing == root) {
            continue;
        }
        roots.push(root.to_path_buf());
    }
    roots
}

pub(super) fn select_stage_run_root(candidates: &[StageRunRootCandidate]) -> StageRunRootSelection {
    let existing_candidates =
        candidates.iter().filter(|candidate| candidate.path.is_dir()).cloned().collect::<Vec<_>>();
    if existing_candidates.is_empty() {
        return StageRunRootSelection {
            selected_path: PathBuf::new(),
            newest_available_path: None,
        };
    }
    let mut freshest_path = existing_candidates[0].path.clone();
    let mut freshest_timestamp = run_root_freshness_timestamp(&freshest_path);
    for candidate in existing_candidates.iter().skip(1) {
        let candidate_timestamp = run_root_freshness_timestamp(&candidate.path);
        if candidate_timestamp.is_some()
            && (freshest_timestamp.is_none() || candidate_timestamp > freshest_timestamp)
        {
            freshest_path.clone_from(&candidate.path);
            freshest_timestamp = candidate_timestamp;
        }
    }
    StageRunRootSelection {
        selected_path: freshest_path.clone(),
        newest_available_path: Some(freshest_path),
    }
}

pub(super) fn run_root_freshness_timestamp(run_root: &Path) -> Option<i64> {
    let manifest_path = run_root.join("run_manifest.json");
    if manifest_path.is_file() {
        let manifest = load_json_value(&manifest_path).ok()?;
        for key in ["completed_at_utc", "generated_at_utc", "finished_at_utc", "started_at_utc"] {
            if let Some(parsed) = manifest
                .get(key)
                .and_then(|value| value.as_str())
                .and_then(bijux_dna_api::v1::api::shared::parse_rfc3339_timestamp_to_unix_seconds)
            {
                return Some(parsed);
            }
        }
    }
    None
}

pub(super) fn run_root_observed_timestamp(run_root: &Path) -> Option<i64> {
    run_root_freshness_timestamp(run_root).or_else(|| {
        fs::metadata(run_root)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .and_then(|value| i64::try_from(value.as_secs()).ok())
    })
}

pub(super) fn observed_tools_from_report(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let pattern = Regex::new(r#""tool"\s*:\s*"([^"]+)""#)
        .map_err(|err| anyhow!("compile tool regex: {err}"))?;
    let tools = pattern
        .captures_iter(&text)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    Ok(tools.into_iter().collect())
}

pub(super) fn localize_results_path(
    path_str: &str,
    local_results_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
) -> PathBuf {
    let path = PathBuf::from(path_str);
    if path.exists() {
        return path;
    }

    let mut root_mappings = vec![("/results/", vec![local_results_root.to_path_buf()])];
    if let Some(extra_data_root) =
        workspace.local.as_ref().and_then(|row| row.extra_data_root.as_deref()).map(PathBuf::from)
    {
        root_mappings.push(("/extra-data/", vec![extra_data_root]));
    }
    if let Some(reference_root) =
        workspace.local.as_ref().and_then(|row| row.reference_root.as_deref()).map(PathBuf::from)
    {
        root_mappings.push(("/reference/", vec![reference_root]));
    }

    let mut fallback_path = None;
    for (marker, mapped_roots) in root_mappings {
        if !path_str.contains(marker) {
            continue;
        }
        let suffix = path_str.split_once(marker).map(|(_, tail)| tail).unwrap_or_default();
        for mapped_root in mapped_roots {
            let localized = mapped_root.join(suffix);
            if localized.exists() {
                return localized;
            }
            if fallback_path.is_none() {
                fallback_path = Some(localized);
            }
        }
    }
    fallback_path.unwrap_or(path)
}

pub(super) fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

pub(super) fn sorted_json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    let mut values = json_string_array(value);
    values.sort();
    values
}

pub(super) fn json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect()
}

pub(super) fn value_string<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(|entry| entry.as_str())
}

pub(super) fn classify_run_root_source(
    run_root: &Path,
    expected_remote_run_root: &Path,
    expected_local_cache_mirror_run_root: &Path,
    expected_local_results_run_root: &Path,
    remote_corpus_root: &Path,
) -> String {
    if run_root == expected_local_cache_mirror_run_root {
        return "local-cache-mirror".to_string();
    }
    if run_root == expected_local_results_run_root {
        return "local-results-root".to_string();
    }
    if run_root == expected_remote_run_root {
        return "remote-results-root".to_string();
    }
    if remote_corpus_root.parent().is_some_and(|root| run_root.starts_with(root)) {
        return "remote-custom".to_string();
    }
    "custom".to_string()
}

pub(super) fn workspace_remote_corpus_root(
    workspace: &BenchmarkWorkspaceConfig,
) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.corpus_root"))
}

pub(super) fn workspace_remote_results_root(
    workspace: &BenchmarkWorkspaceConfig,
) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.results_root"))
}

pub(super) fn workspace_local_cache_mirror_root(
    workspace: &BenchmarkWorkspaceConfig,
) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))
}

pub(super) fn workspace_local_results_root(
    workspace: &BenchmarkWorkspaceConfig,
) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))
}

pub(super) fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
