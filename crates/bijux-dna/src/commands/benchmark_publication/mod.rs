use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;

mod entrypoint;
mod corpus_dossier;
mod dossier_index;
mod docs_audit;
mod docs_status;
mod models;
mod results_status;

use crate::commands::benchmark_workspace::{
    benchmark_corpus_spec_path, benchmark_publication_contracts, benchmark_publication_exclusions,
    benchmark_runtime_corpus_dir_name, benchmark_stage_run_relative_root, load_benchmark_config,
    BenchmarkWorkspaceConfig,
    CorpusBenchmarkContract, CorpusBenchmarkExclusion,
};

pub(crate) use self::entrypoint::{
    print_benchmark_publication_targets, run_corpus_fastq_publication_status,
    run_corpus_fastq_published_dossiers, run_corpus_fastq_report,
};
use self::corpus_dossier::render_corpus_fastq_dossier;
use self::dossier_index::write_corpus_fastq_dossier_index;
use self::docs_audit::{audit_publication_docs, render_publication_docs_markdown};
use self::docs_status::{expected_counts_for_scope, write_corpus_fastq_docs_status};
use self::models::{
    RemediationIssue, RemediationIssueGroup, RemediationQueue, RemediationStageEntry,
    StageAuditIssue, StageRunRootCandidate, StageRunRootSelection,
};
#[cfg(test)]
use self::docs_audit::audit_publication_stage;
#[cfg(test)]
use self::docs_status::{load_publication_corpus_spec, load_supplemental_findings};
#[cfg(test)]
use self::models::{
    BenchmarkPublicationStatusReport, ExcludedStageEntry, PublicationCorpusSpec,
    PublicationStageReport,
};
use self::results_status::write_corpus_fastq_results_status;
#[cfg(test)]
use self::models::{PublishedResultsStageReport, PublishedResultsStatusReport, StageResultIssue};
#[cfg(test)]
use self::results_status::{
    audit_published_results, audit_published_results_stage, render_published_results_markdown,
};

fn publication_stage_docs_root(docs_root: &Path, stage_id: &str, corpus_id: &str) -> PathBuf {
    docs_root.join(stage_id).join(corpus_id)
}

fn publication_artifact_file_name(corpus_id: &str, suffix: &str) -> String {
    format!("{corpus_id}-{suffix}")
}

fn publication_method_file_name(corpus_id: &str) -> String {
    format!("{corpus_id}-method.md")
}

fn write_corpus_fastq_remediation_queue(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let publication_status =
        load_json_value(&docs_root.join(publication_artifact_file_name(corpus_id, "status.json")))?;
    let results_status = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "results-status.json",
    )))?;
    let findings_payload = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "publication-findings.json",
    )))?;
    let dossier_index = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "dossier-index.json",
    )))?;
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;
    let queue = build_remediation_queue(
        corpus_id,
        &contracts,
        &publication_status,
        &results_status,
        &findings_payload,
        &dossier_index,
    )?;
    let json_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "remediation-queue.json",
    ));
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&queue)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "remediation-queue.md",
    ));
    fs::write(&markdown_path, render_remediation_queue_markdown(&queue))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn load_json_value(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}



fn audit_publication_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
    expected_total: usize,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !summary_path.is_file() || fs::metadata(summary_path)?.len() == 0 {
        return Ok(());
    }
    let summary = load_json_value(summary_path)?;
    if value_string(&summary, "stage_id") != Some(contract.stage_id.as_str()) {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-stage-id-drift",
            format!(
                "{} stage_id={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("stage_id").and_then(|value| value.as_str())
            ),
            "error",
        );
    }
    if value_string(&summary, "scenario_id") != Some(contract.scenario_id.as_str()) {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-scenario-id-drift",
            format!(
                "{} scenario_id={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("scenario_id").and_then(|value| value.as_str())
            ),
            "error",
        );
    }
    if sorted_json_string_array(summary.get("tools")) != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-tool-roster-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                json_string_array(summary.get("tools")),
                expected_tools
            ),
            "error",
        );
    }
    if summary
        .get("samples_total")
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as usize
        != expected_total
    {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-sample-count-drift",
            format!(
                "{} samples_total={:?} expected {}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("samples_total"),
                expected_total
            ),
            "error",
        );
    }
    if summary
        .get("samples_failed")
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
        != 0
    {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-sample-failures",
            format!(
                "{} samples_failed={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("samples_failed")
            ),
            "error",
        );
    }
    if sort_count_map(summary.get("cohort_counts"))? != *expected_cohort_counts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-cohort-count-drift",
            format!(
                "{} cohort_counts={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("cohort_counts"),
                expected_cohort_counts
            ),
            "error",
        );
    }
    let summary_tool_ids = summary
        .get("tool_summary")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("tool").and_then(|value| value.as_str()))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if summary_tool_ids != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-tool-summary-drift",
            format!(
                "{} tool_summary tools={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary_tool_ids,
                expected_tools
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_sample_results(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    sample_results_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
    expected_total: usize,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !sample_results_path.is_file() || fs::metadata(sample_results_path)?.len() == 0 {
        return Ok(());
    }
    let sample_rows = load_csv_rows(sample_results_path)?;
    if sample_rows.is_empty() {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "empty-sample-results-rows",
            format!(
                "no CSV rows in {}",
                relative_to_docs_root(sample_results_path, docs_root)
            ),
            "error",
        );
        return Ok(());
    }

    let mut per_sample_tools = BTreeMap::<String, Vec<String>>::new();
    let mut sample_metadata = BTreeMap::<String, (String, String, String, String, String)>::new();
    let mut cohort_counts_by_rows = BTreeMap::<String, usize>::new();
    let mut observed_tools = BTreeSet::new();

    for row in &sample_rows {
        let (Some(sample_id), Some(tool)) = (
            csv_required_value(row, "sample_id"),
            csv_required_value(row, "tool"),
        ) else {
            append_stage_audit_issue(
                issues,
                &contract.stage_id,
                "sample-results-missing-sample-or-tool",
                format!(
                    "invalid row in {}",
                    relative_to_docs_root(sample_results_path, docs_root)
                ),
                "error",
            );
            continue;
        };
        observed_tools.insert(tool.clone());
        per_sample_tools
            .entry(sample_id.clone())
            .or_default()
            .push(tool);
        let metadata_tuple = (
            csv_report_value(row, "accession"),
            csv_report_value(row, "era"),
            csv_report_value(row, "layout"),
            csv_report_value(row, "study_accession"),
            csv_report_value(row, "size_band"),
        );
        if let Some(existing) = sample_metadata.get(&sample_id) {
            if existing != &metadata_tuple {
                append_stage_audit_issue(
                    issues,
                    &contract.stage_id,
                    "sample-results-metadata-drift",
                    format!(
                        "{} sample {} metadata differs across rows",
                        relative_to_docs_root(sample_results_path, docs_root),
                        sample_id
                    ),
                    "error",
                );
            }
        } else {
            *cohort_counts_by_rows
                .entry(format!("{}_{}", metadata_tuple.1, metadata_tuple.2))
                .or_default() += 1;
            sample_metadata.insert(sample_id, metadata_tuple);
        }
    }

    let observed_tools = observed_tools.into_iter().collect::<Vec<_>>();
    if observed_tools != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-tool-roster-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(sample_results_path, docs_root),
                observed_tools,
                expected_tools
            ),
            "error",
        );
    }
    if sample_metadata.len() != expected_total {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-sample-count-drift",
            format!(
                "{} unique_samples={:?} expected {}",
                relative_to_docs_root(sample_results_path, docs_root),
                sample_metadata.len(),
                expected_total
            ),
            "error",
        );
    }
    if cohort_counts_by_rows != *expected_cohort_counts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-cohort-count-drift",
            format!(
                "{} cohort_counts={:?} expected {:?}",
                relative_to_docs_root(sample_results_path, docs_root),
                cohort_counts_by_rows,
                expected_cohort_counts
            ),
            "error",
        );
    }
    for (sample_id, tools) in &per_sample_tools {
        let mut sample_tools = tools.clone();
        sample_tools.sort();
        if sample_tools != expected_tools {
            append_stage_audit_issue(
                issues,
                &contract.stage_id,
                "sample-results-tool-coverage-drift",
                format!(
                    "{} sample {} tools={:?} expected {:?}",
                    relative_to_docs_root(sample_results_path, docs_root),
                    sample_id,
                    sample_tools,
                    expected_tools
                ),
                "error",
            );
        }
    }
    let expected_row_count = expected_total * expected_tools.len();
    if sample_rows.len() != expected_row_count {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-row-count-drift",
            format!(
                "{} row_count={:?} expected {}",
                relative_to_docs_root(sample_results_path, docs_root),
                sample_rows.len(),
                expected_row_count
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_tool_runtime_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    tool_runtime_summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
) -> Result<()> {
    if !tool_runtime_summary_path.is_file() || fs::metadata(tool_runtime_summary_path)?.len() == 0 {
        return Ok(());
    }
    let mut observed_tools = load_csv_rows(tool_runtime_summary_path)?
        .into_iter()
        .filter_map(|row| csv_required_value(&row, "tool"))
        .collect::<Vec<_>>();
    observed_tools.sort();
    if observed_tools != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "tool-runtime-summary-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(tool_runtime_summary_path, docs_root),
                observed_tools,
                expected_tools
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_cohort_runtime_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    cohort_runtime_summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !cohort_runtime_summary_path.is_file()
        || fs::metadata(cohort_runtime_summary_path)?.len() == 0
    {
        return Ok(());
    }
    let observed_cohorts = load_csv_rows(cohort_runtime_summary_path)?
        .into_iter()
        .filter(|row| {
            let dimension = csv_report_value(row, "dimension");
            dimension == "missing" || dimension == "era_layout"
        })
        .map(|row| csv_report_value(&row, "cohort"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let expected_cohorts = expected_cohort_counts.keys().cloned().collect::<Vec<_>>();
    if observed_cohorts != expected_cohorts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "cohort-runtime-summary-drift",
            format!(
                "{} cohorts={:?} expected {:?}",
                relative_to_docs_root(cohort_runtime_summary_path, docs_root),
                observed_cohorts,
                expected_cohorts
            ),
            "error",
        );
    }
    Ok(())
}

fn audit_sample_runtime_outliers(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    sample_runtime_outliers_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_total: usize,
) -> Result<()> {
    if !sample_runtime_outliers_path.is_file()
        || fs::metadata(sample_runtime_outliers_path)?.len() == 0
    {
        return Ok(());
    }
    let unique_sample_ids = load_csv_rows(sample_runtime_outliers_path)?
        .into_iter()
        .map(|row| csv_value(&row, "sample_id"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if unique_sample_ids.len() != expected_total {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-runtime-outlier-coverage-drift",
            format!(
                "{} unique_samples={:?} expected {}",
                relative_to_docs_root(sample_runtime_outliers_path, docs_root),
                unique_sample_ids.len(),
                expected_total
            ),
            "error",
        );
    }
    Ok(())
}

fn append_stage_audit_issue(
    issues: &mut Vec<StageAuditIssue>,
    stage_id: &str,
    issue_id: &str,
    detail: String,
    severity: &str,
) {
    issues.push(StageAuditIssue {
        stage_id: stage_id.to_string(),
        issue_id: issue_id.to_string(),
        severity: severity.to_string(),
        detail,
    });
}

fn relative_to_docs_root(path: &Path, docs_root: &Path) -> String {
    let repo_root = docs_root
        .parent()
        .and_then(Path::parent)
        .unwrap_or(docs_root.parent().unwrap_or(docs_root));
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn relative_to_repo_root(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn load_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let Some(header_line) = lines.next() else {
        return Ok(Vec::new());
    };
    let headers = header_line
        .split(',')
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values = line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        let mut row = BTreeMap::new();
        for (header, value) in headers.iter().zip(values.iter()) {
            row.insert(header.clone(), (*value).to_string());
        }
        rows.push(row);
    }
    Ok(rows)
}

fn csv_value(row: &BTreeMap<String, String>, key: &str) -> String {
    row.get(key)
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "missing".to_string())
}

fn csv_required_value(row: &BTreeMap<String, String>, key: &str) -> Option<String> {
    let value = csv_value(row, key);
    (value != "missing" && !value.is_empty()).then_some(value)
}

fn csv_report_value(row: &BTreeMap<String, String>, key: &str) -> String {
    csv_required_value(row, key).unwrap_or_else(|| "missing".to_string())
}

fn sort_count_map(value: Option<&serde_json::Value>) -> Result<BTreeMap<String, usize>> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("count map must be a JSON object"))?;
    object
        .iter()
        .map(|(key, value)| {
            let count = value
                .as_u64()
                .ok_or_else(|| anyhow!("count map entry `{key}` must be an unsigned integer"))?;
            Ok((key.clone(), count as usize))
        })
        .collect()
}

fn summary_corpus_id(summary_corpus_root: &Path) -> Result<String> {
    summary_corpus_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("summary corpus_root must end with a corpus directory name"))
}

fn configured_stage_run_roots(
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

fn unique_existing_run_roots(
    reported_run_root: &Path,
    configured_roots: &[StageRunRootCandidate],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for root in std::iter::once(reported_run_root).chain(
        configured_roots
            .iter()
            .map(|candidate| candidate.path.as_path()),
    ) {
        if !root.is_dir() || roots.iter().any(|existing| existing == root) {
            continue;
        }
        roots.push(root.to_path_buf());
    }
    roots
}

fn select_stage_run_root(candidates: &[StageRunRootCandidate]) -> StageRunRootSelection {
    let existing_candidates = candidates
        .iter()
        .filter(|candidate| candidate.path.is_dir())
        .cloned()
        .collect::<Vec<_>>();
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
            freshest_path = candidate.path.clone();
            freshest_timestamp = candidate_timestamp;
        }
    }
    StageRunRootSelection {
        selected_path: freshest_path.clone(),
        newest_available_path: Some(freshest_path),
    }
}

fn run_root_freshness_timestamp(run_root: &Path) -> Option<DateTime<Utc>> {
    let manifest_path = run_root.join("run_manifest.json");
    if manifest_path.is_file() {
        let manifest = load_json_value(&manifest_path).ok()?;
        for key in [
            "completed_at_utc",
            "generated_at_utc",
            "finished_at_utc",
            "started_at_utc",
        ] {
            if let Some(parsed) =
                parse_utc_timestamp(manifest.get(key).and_then(|value| value.as_str()))
            {
                return Some(parsed);
            }
        }
    }
    None
}

fn run_root_observed_timestamp(run_root: &Path) -> Option<DateTime<Utc>> {
    run_root_freshness_timestamp(run_root).or_else(|| {
        fs::metadata(run_root)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(DateTime::<Utc>::from)
    })
}

fn parse_utc_timestamp(raw: Option<&str>) -> Option<DateTime<Utc>> {
    let normalized = raw?.trim().replace('Z', "+00:00");
    if normalized.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(&normalized)
        .map(|value| value.with_timezone(&Utc))
        .ok()
}

fn find_polluting_ds_store_files(root: &Path) -> Vec<PathBuf> {
    let mut polluting_files = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return polluting_files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            polluting_files.extend(find_polluting_ds_store_files(&path));
        } else if path.file_name().and_then(|value| value.to_str()) == Some(".DS_Store") {
            polluting_files.push(path);
        }
    }
    polluting_files.sort();
    polluting_files
}

fn observed_tools_from_report(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let pattern = Regex::new(r#""tool"\s*:\s*"([^"]+)""#).expect("tool regex");
    let tools = pattern
        .captures_iter(&text)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    Ok(tools.into_iter().collect())
}

fn localize_results_path(
    path_str: &str,
    local_results_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
) -> PathBuf {
    let path = PathBuf::from(path_str);
    if path.exists() {
        return path;
    }

    let mut root_mappings = vec![("/results/", vec![local_results_root.to_path_buf()])];
    if let Some(extra_data_root) = workspace
        .local
        .as_ref()
        .and_then(|row| row.extra_data_root.as_deref())
        .map(PathBuf::from)
    {
        root_mappings.push(("/extra-data/", vec![extra_data_root]));
    }
    if let Some(reference_root) = workspace
        .local
        .as_ref()
        .and_then(|row| row.reference_root.as_deref())
        .map(PathBuf::from)
    {
        root_mappings.push(("/reference/", vec![reference_root]));
    }

    let mut fallback_path = None;
    for (marker, mapped_roots) in root_mappings {
        if !path_str.contains(marker) {
            continue;
        }
        let suffix = path_str
            .split_once(marker)
            .map(|(_, tail)| tail)
            .unwrap_or_default();
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

fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

fn sorted_json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    let mut values = json_string_array(value);
    values.sort();
    values
}

fn json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect()
}

fn value_string<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(|entry| entry.as_str())
}

fn build_remediation_queue(
    corpus_id: &str,
    contracts: &[CorpusBenchmarkContract],
    publication_status: &serde_json::Value,
    results_status: &serde_json::Value,
    findings_payload: &serde_json::Value,
    dossier_index: &serde_json::Value,
) -> Result<RemediationQueue> {
    let publication_by_stage = stage_value_lookup(publication_status);
    let results_by_stage = stage_value_lookup(results_status);
    let dossier_by_stage = stage_value_lookup(dossier_index);
    let findings_by_stage = findings_lookup(findings_payload);

    let stages = contracts
        .iter()
        .map(|contract| -> Result<RemediationStageEntry> {
            let publication_stage = publication_by_stage.get(&contract.stage_id);
            let results_stage = results_by_stage.get(&contract.stage_id);
            let dossier_stage = dossier_by_stage.get(&contract.stage_id);

            let mut issues = collect_stage_issues(publication_stage, "publication");
            issues.extend(collect_stage_issues(results_stage, "results"));
            issues.extend(findings_by_stage.get(&contract.stage_id).cloned().ok_or_else(
                || anyhow!("remediation queue missing findings for stage `{}`", contract.stage_id),
            )?);
            let issue_groups = summarize_issue_groups(&issues);
            let issue_ids = issues
                .iter()
                .map(|issue| issue.issue_id.clone())
                .collect::<Vec<_>>();

            Ok(RemediationStageEntry {
                stage_id: contract.stage_id.clone(),
                owner: "benchmark-governance".to_string(),
                status: if issues.is_empty() {
                    "clear".to_string()
                } else {
                    "open".to_string()
                },
                issue_count: issues.len(),
                issue_group_count: issue_groups.len(),
                recommended_action: if issues.is_empty() {
                    "none".to_string()
                } else {
                    classify_recommended_action(&issue_ids)
                },
                publication_status: stage_value_string(publication_stage, "status", "missing"),
                results_status: stage_value_string(results_stage, "status", "missing"),
                sample_scope: contract.sample_scope.clone(),
                published_generated_at_utc: stage_value_optional_string(
                    dossier_stage,
                    "generated_at_utc",
                ),
                run_root_source: stage_value_optional_string(dossier_stage, "run_root_source"),
                issue_groups,
                issues,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(RemediationQueue {
        corpus_id: corpus_id.to_string(),
        stage_count: stages.len(),
        open_stage_count: stages.iter().filter(|stage| stage.status == "open").count(),
        clear_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "clear")
            .count(),
        stages,
    })
}

fn stage_value_lookup<'a>(
    payload: &'a serde_json::Value,
) -> BTreeMap<String, &'a serde_json::Value> {
    payload
        .get("stages")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|stage| {
            stage
                .get("stage_id")
                .and_then(|value| value.as_str())
                .map(|stage_id| (stage_id.to_string(), stage))
        })
        .collect()
}

fn declared_issue_field<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value.get(field)
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
}

fn findings_lookup(payload: &serde_json::Value) -> BTreeMap<String, Vec<RemediationIssue>> {
    let mut findings_by_stage = BTreeMap::new();
    for finding in payload
        .get("findings")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
    {
        let Some(stage_id) = declared_issue_field(finding, "stage_id") else {
            continue;
        };
        let Some(issue_id) = declared_issue_field(finding, "issue_id") else {
            continue;
        };
        let Some(detail) = declared_issue_field(finding, "detail") else {
            continue;
        };
        let Some(severity) = declared_issue_field(finding, "severity") else {
            continue;
        };
        findings_by_stage
            .entry(stage_id.to_string())
            .or_insert_with(Vec::new)
            .push(RemediationIssue {
                issue_id: issue_id.to_string(),
                detail: detail.to_string(),
                severity: severity.to_string(),
                source: "findings".to_string(),
            });
    }
    findings_by_stage
}

fn collect_stage_issues(stage: Option<&&serde_json::Value>, source: &str) -> Vec<RemediationIssue> {
    stage
        .and_then(|value| value.get("issues"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|issue| {
            let issue_id = declared_issue_field(issue, "issue_id")?;
            let detail = declared_issue_field(issue, "detail")?;
            let severity = declared_issue_field(issue, "severity")?;
            Some(RemediationIssue {
                issue_id: issue_id.to_string(),
                detail: detail.to_string(),
                severity: severity.to_string(),
                source: source.to_string(),
            })
        })
        .collect()
}

fn summarize_issue_groups(issues: &[RemediationIssue]) -> Vec<RemediationIssueGroup> {
    let mut grouped = BTreeMap::<String, (usize, BTreeMap<String, ()>, Vec<String>, String)>::new();
    for issue in issues {
        let group = grouped
            .entry(issue.issue_id.clone())
            .or_insert_with(|| (0, BTreeMap::new(), Vec::new(), issue.severity.clone()));
        group.0 += 1;
        group.1.insert(issue.source.clone(), ());
        let detail = issue.detail.trim();
        if !detail.is_empty() && !group.2.iter().any(|existing| existing == detail) {
            group.2.push(detail.to_string());
        }
    }
    grouped
        .into_iter()
        .map(
            |(issue_id, (count, sources, details, severity))| RemediationIssueGroup {
                issue_id,
                count,
                sources: sources.into_keys().collect(),
                severity,
                example_details: details.iter().take(3).cloned().collect(),
                additional_detail_count: details.len().saturating_sub(3),
            },
        )
        .collect()
}

fn classify_recommended_action(issue_ids: &[String]) -> String {
    let sync_issue_ids = [
        "missing-local-run-root",
        "missing-stage-run-manifest",
        "missing-localized-report-json",
        "duplicate-result-root-ambiguity",
    ];
    let publish_issue_ids = [
        "missing-published-summary",
        "missing-corpus-dir",
        "missing-summary-json",
        "missing-benchmark-md",
        "missing-sample-results-csv",
        "missing-tool-runtime-summary-csv",
        "missing-cohort-runtime-summary-csv",
        "missing-sample-runtime-outliers-csv",
    ];
    let rerun_issue_fragments = ["sample-failures", "dry-run", "sample-limit"];
    if issue_ids
        .iter()
        .any(|issue_id| sync_issue_ids.contains(&issue_id.as_str()))
    {
        return "sync-or-normalize-results".to_string();
    }
    if issue_ids
        .iter()
        .any(|issue_id| publish_issue_ids.contains(&issue_id.as_str()))
    {
        return "render-or-publish-dossier".to_string();
    }
    if issue_ids.iter().any(|issue_id| {
        rerun_issue_fragments
            .iter()
            .any(|fragment| issue_id.contains(fragment))
    }) {
        return "rerun-benchmark-stage".to_string();
    }
    "repair-benchmark-contract".to_string()
}

fn render_remediation_queue_markdown(queue: &RemediationQueue) -> String {
    let mut lines = vec![
        format!("# `{}` FASTQ remediation queue", queue.corpus_id),
        "".to_string(),
        format!("- Governed publication stages: `{}`", queue.stage_count),
        format!("- Open stages: `{}`", queue.open_stage_count),
        format!("- Clear stages: `{}`", queue.clear_stage_count),
        "".to_string(),
        "## Stage queue".to_string(),
        "".to_string(),
    ];
    for stage in &queue.stages {
        lines.push(format!(
            "- `{}`: `{}` via `{}`",
            stage.stage_id, stage.status, stage.recommended_action
        ));
        lines.push(format!(
            "  - publication `{}`, results `{}`, owner `{}`",
            stage.publication_status, stage.results_status, stage.owner
        ));
        if let Some(generated_at) = stage.published_generated_at_utc.as_deref() {
            lines.push(format!(
                "  - dossier `{}` from `{}`",
                generated_at,
                stage.run_root_source.as_deref().unwrap_or("missing")
            ));
        }
        for group in &stage.issue_groups {
            lines.push(format!(
                "  - issue group `{}` x{} from {}",
                group.issue_id,
                group.count,
                group.sources.join(", ")
            ));
            for detail in &group.example_details {
                lines.push(format!("    - {detail}"));
            }
            if group.additional_detail_count > 0 {
                lines.push(format!(
                    "    - (+{} more detail rows)",
                    group.additional_detail_count
                ));
            }
        }
    }
    lines.join("\n") + "\n"
}

fn stage_value_string(stage: Option<&&serde_json::Value>, key: &str, default: &str) -> String {
    stage
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or(default)
        .to_string()
}

fn stage_value_optional_string(stage: Option<&&serde_json::Value>, key: &str) -> Option<String> {
    stage
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn classify_run_root_source(
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
    if remote_corpus_root
        .parent()
        .is_some_and(|root| run_root.starts_with(root))
    {
        return "remote-custom".to_string();
    }
    "custom".to_string()
}

fn workspace_remote_corpus_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.corpus_root"))
}

fn workspace_remote_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.results_root"))
}

fn workspace_local_cache_mirror_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))
}

fn workspace_local_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    fn validate_reads_contract() -> crate::commands::benchmark_workspace::CorpusBenchmarkContract {
        crate::commands::benchmark_workspace::CorpusBenchmarkContract {
            stage_id: "fastq.validate_reads".to_string(),
            scenario_id: "validation_fairness".to_string(),
            sample_scope: "full".to_string(),
            tools: vec![
                "fastqvalidator".to_string(),
                "fastqc".to_string(),
                "fastq_scan".to_string(),
                "fqtools".to_string(),
                "seqtk".to_string(),
            ],
        }
    }

    fn sample_workspace(
        cache_root: &Path,
        archive_root: &Path,
        remote_root: &Path,
        remote_corpus_root: &Path,
    ) -> crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
        crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(
                crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                    results_root: Some(archive_root.display().to_string()),
                    cache_mirror_root: Some(cache_root.display().to_string()),
                    extra_data_root: Some(cache_root.join("extra-data").display().to_string()),
                    reference_root: Some(cache_root.join("reference").display().to_string()),
                },
            ),
            remote: Some(
                crate::commands::benchmark_workspace::BenchmarkWorkspaceRemote {
                    corpus_root: Some(remote_corpus_root.display().to_string()),
                    results_root: Some(remote_root.join("results").display().to_string()),
                    ..Default::default()
                },
            ),
            layout: None,
            artifacts: BTreeMap::new(),
            sync: None,
        }
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(
            path,
            format!("{}\n", serde_json::to_string_pretty(&value).expect("json")),
        )
        .expect("write json");
    }

    #[test]
    fn publication_command_maps_profile_overrepresented_stage_report() {
        assert_eq!(
            super::corpus_fastq_publication_command(
                "fastq.profile_overrepresented_sequences",
                "corpus-01",
                "report",
                None,
            )
            .expect("report command"),
            "bijux-dna bench corpus-fastq-report --stage fastq.profile_overrepresented_sequences --corpus-id corpus-01"
        );
    }

    #[test]
    fn publication_command_maps_merge_pairs_stage_run() {
        assert_eq!(
            super::corpus_fastq_publication_command("fastq.merge_pairs", "corpus-01", "run", None)
                .expect("run command"),
            "bijux-dna bench corpus-fastq --corpus-id corpus-01 --stage fastq.merge_pairs"
        );
    }

    #[test]
    fn publication_command_includes_config_override() {
        assert_eq!(
            super::corpus_fastq_publication_command(
                "fastq.filter_reads",
                "corpus-01",
                "report",
                Some(Path::new("configs/bench/alt.toml")),
            )
            .expect("report command"),
            "bijux-dna bench corpus-fastq-report --stage fastq.filter_reads --corpus-id corpus-01 --config configs/bench/alt.toml"
        );
    }

    #[test]
    fn corpus_fastq_report_docs_root_tracks_stage_contract() {
        let docs_root = super::absolutize(Path::new("/repo"), Path::new("docs/benchmark"))
            .join("fastq.validate_reads")
            .join("corpus-01");
        assert_eq!(
            docs_root,
            Path::new("/repo/docs/benchmark/fastq.validate_reads/corpus-01")
        );
    }

    #[test]
    fn resolve_existing_dossier_path_uses_benchmark_markdown_contract() {
        let temp = tempdir().expect("tempdir");
        let stage_docs_root = temp.path().join("docs/benchmark/fastq.validate_reads/corpus-01");
        fs::create_dir_all(&stage_docs_root).expect("stage docs root");
        fs::write(stage_docs_root.join("legacy-site.md"), "# legacy\n").expect("legacy dossier");

        assert_eq!(
            super::resolve_existing_dossier_path(&stage_docs_root),
            stage_docs_root.join("benchmark.md")
        );
    }

    #[test]
    fn run_corpus_fastq_report_writes_governed_dossier_without_python_scripts() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let cache_root = repo_root.join("cache-mirror");
        let archive_root = repo_root.join("archive");
        let remote_root = repo_root.join("remote");
        let remote_corpus_root = repo_root.join("benchmark_corpus");
        let config_path = repo_root.join("configs/bench/benchmark.toml");
        let corpus_spec_path = repo_root.join("configs/runtime/corpora/corpus-01.toml");
        fs::create_dir_all(config_path.parent().expect("config dir")).expect("config dir");
        fs::create_dir_all(corpus_spec_path.parent().expect("corpus spec dir"))
            .expect("corpus spec dir");
        fs::create_dir_all(remote_corpus_root.join("raw/DRR000001")).expect("raw dir");
        fs::create_dir_all(remote_corpus_root.join("normalized")).expect("normalized dir");
        fs::create_dir_all(cache_root.join("results")).expect("cache results dir");
        fs::create_dir_all(&archive_root).expect("archive dir");
        fs::create_dir_all(remote_root.join("results")).expect("remote results dir");

        fs::write(
            &config_path,
            format!(
                r#"[workspace.local]
results_root = "{}"
cache_mirror_root = "{}"
extra_data_root = "{}"
reference_root = "{}"

[workspace.remote]
corpus_root = "{}"
results_root = "{}"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"]
"#,
                archive_root.display(),
                cache_root.display(),
                cache_root.join("extra-data").display(),
                cache_root.join("reference").display(),
                remote_corpus_root.display(),
                remote_root.join("results").display(),
            ),
        )
        .expect("write benchmark config");
        fs::write(
            &corpus_spec_path,
            r#"schema_version = "bijux.corpus_spec.v1"
corpus_id = "corpus-01"
target_ancient_se = 0
target_ancient_pe = 0
target_modern_se = 1
target_modern_pe = 0

[[samples]]
accession = "DRR000001"
study_accession = "PRJ000001"
era = "modern"
layout = "se"
size_band = "under_100mb"
reason = "Compact validation fixture."
"#,
        )
        .expect("write corpus spec");

        let raw_fastq = remote_corpus_root.join("raw/DRR000001/reads.fastq.gz");
        let normalized_fastq = remote_corpus_root.join("normalized/sample_0001_R1.fastq.gz");
        fs::write(&raw_fastq, b"raw-fastq\n").expect("raw fastq");
        fs::write(&normalized_fastq, b"raw-fastq\n").expect("normalized fastq");
        write_json(
            &remote_corpus_root.join("MANIFEST.json"),
            serde_json::json!({
                "files": {
                    "raw/DRR000001/reads.fastq.gz": "sha256:fixture",
                    "normalized/sample_0001_R1.fastq.gz": "sha256:fixture"
                }
            }),
        );

        let run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}, "execution": {"runtime_s": 1.2, "exit_code": 0}},
                    {"context": {"tool": "fastqc"}, "execution": {"runtime_s": 2.3, "exit_code": 0}},
                    {"context": {"tool": "fastq_scan"}, "execution": {"runtime_s": 0.9, "exit_code": 0}},
                    {"context": {"tool": "fqtools"}, "execution": {"runtime_s": 1.0, "exit_code": 0}},
                    {"context": {"tool": "seqtk"}, "execution": {"runtime_s": 1.1, "exit_code": 0}}
                ]
            }),
        );
        write_json(
            &run_root.join("run_manifest.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "platform": "cluster-apptainer",
                "samples_failed": 0,
                "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                "runs": [{
                    "sample_id": "sample_0001",
                    "report_json": sample_report
                }]
            }),
        );

        super::run_corpus_fastq_report(
            repo_root,
            &crate::commands::cli::BenchCorpusFastqReportArgs {
                stage: "fastq.validate_reads".to_string(),
                corpus_id: "corpus-01".to_string(),
                config: Some(PathBuf::from("configs/bench/benchmark.toml")),
                docs_root: PathBuf::from("docs/benchmark"),
                run_root: Some(run_root.clone()),
            },
        )
        .expect("render dossier");

        let stage_docs_root = repo_root
            .join("docs/benchmark")
            .join("fastq.validate_reads")
            .join("corpus-01");
        let summary = fs::read_to_string(stage_docs_root.join("summary.json")).expect("summary");
        let benchmark_md =
            fs::read_to_string(stage_docs_root.join("benchmark.md")).expect("benchmark md");
        assert!(summary.contains("\"stage_id\": \"fastq.validate_reads\""));
        assert!(summary.contains("\"samples_total\": 1"));
        assert!(benchmark_md.contains("generated directly by `bijux-dna`"));
        assert!(stage_docs_root.join("tool_runtime_summary.csv").is_file());
        assert!(stage_docs_root.join("cohort_runtime_summary.csv").is_file());
        assert!(stage_docs_root
            .join("sample_runtime_outliers.csv")
            .is_file());
    }

    #[test]
    fn classify_run_root_source_prefers_local_cache_mirror() {
        assert_eq!(
            super::classify_run_root_source(
                Path::new(
                    "/archive/bench/cluster/.cache/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/bench/cluster/.cache/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/bench/cluster/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/archive/bench/cluster/.cache/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new("/bench/cluster/.cache/benchmark_corpus"),
            ),
            "local-cache-mirror"
        );
    }

    #[test]
    fn localize_results_path_does_not_translate_legacy_results_aliases() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                results_root: Some("/bench/local/results".to_string()),
                cache_mirror_root: Some("/bench/local/cache-mirror".to_string()),
                extra_data_root: None,
                reference_root: None,
            }),
            ..Default::default()
        };

        let localized = super::localize_results_path(
            "/bench/local/cache-mirror/bijux-dna-results/corpus_01/fastq.validate_reads/cluster-apptainer/run_manifest.json",
            Path::new("/bench/local/results"),
            &workspace,
        );

        assert_eq!(
            localized,
            PathBuf::from(
                "/bench/local/cache-mirror/bijux-dna-results/corpus_01/fastq.validate_reads/cluster-apptainer/run_manifest.json"
            )
        );
    }

    #[test]
    fn stage_run_relative_root_uses_workspace_local_cache_template() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            layout: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLayout {
                stage_runs: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceStageRuns {
                    local_cache_results_template: Some(
                        "results/{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    ..Default::default()
                }),
            }),
            ..Default::default()
        };
        assert_eq!(
            crate::commands::benchmark_workspace::benchmark_stage_run_relative_root(
                &workspace,
                "local-cache",
                "benchmark_corpus",
                "fastq.validate_reads",
            )
            .expect("relative root"),
            Path::new("results/benchmark_corpus/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn configured_stage_run_roots_only_publish_local_mirrors() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                results_root: Some("/bench/local/archive".to_string()),
                cache_mirror_root: Some("/bench/local/cache-mirror".to_string()),
                extra_data_root: Some("/bench/local/extra-data".to_string()),
                reference_root: None,
            }),
            remote: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceRemote {
                results_root: Some("/bench/remote/results".to_string()),
                ..Default::default()
            }),
            layout: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLayout {
                stage_runs: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceStageRuns {
                    local_cache_results_template: Some(
                        "results/{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    local_archive_results_template: Some(
                        "{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    remote_results_template: Some(
                        "{corpus_id}/{stage_id}/remote-cluster".to_string(),
                    ),
                }),
            }),
            ..Default::default()
        };

        let roots =
            super::configured_stage_run_roots(&workspace, "benchmark_corpus", "fastq.validate_reads")
                .expect("stage roots");
        assert_eq!(roots.len(), 2);
        assert_eq!(
            roots[0].path,
            PathBuf::from("/bench/local/cache-mirror/results/benchmark_corpus/fastq.validate_reads/cluster")
        );
        assert_eq!(
            roots[1].path,
            PathBuf::from("/bench/local/archive/benchmark_corpus/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn select_stage_run_root_requires_existing_mirrors() {
        let roots = vec![
            super::StageRunRootCandidate {
                path: PathBuf::from("/bench/local/cache-mirror/results/corpus_01/fastq.validate_reads/cluster-apptainer"),
            },
            super::StageRunRootCandidate {
                path: PathBuf::from("/bench/local/archive/corpus_01/fastq.validate_reads/cluster-apptainer"),
            },
        ];

        let selection = super::select_stage_run_root(&roots);

        assert!(selection.selected_path.as_os_str().is_empty());
        assert!(selection.newest_available_path.is_none());
    }

    #[test]
    fn dossier_stage_entry_uses_requested_corpus_contract() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = remote_root.join("shared-corpus-root");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );

        let entry = super::build_dossier_stage_entry(
            temp.path(),
            &docs_root,
            &workspace,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("dossier entry");

        assert_eq!(
            entry.expected_remote_run_root,
            remote_root
                .join("results")
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
        assert_eq!(
            entry.expected_local_cache_mirror_run_root,
            cache_root
                .join("results")
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
        assert_eq!(
            entry.expected_local_results_run_root,
            archive_root
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
    }

    #[test]
    fn results_audit_tracks_missing_published_stage_summary() {
        let temp = tempdir().expect("tempdir");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let report = super::audit_published_results(
            temp.path(),
            &workspace,
            &temp.path().join("docs").join("benchmark"),
            "corpus-01",
            &[validate_reads_contract()],
        )
        .expect("results audit");
        assert_eq!(report.applicable_stage_count, 1);
        assert!(report
            .stages
            .iter()
            .flat_map(|stage| stage.issues.iter())
            .any(|issue| issue.issue_id == "missing-published-summary"));
    }

    #[test]
    fn results_audit_requires_summary_corpus_root() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "run_root": cache_root.join("results"),
            }),
        );

        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "missing-summary-corpus-root"));
    }

    #[test]
    fn results_audit_missing_local_run_root_reports_expected_mirror() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let reported_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": reported_run_root,
            }),
        );
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        let missing_issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_id == "missing-local-run-root")
            .expect("missing issue");
        assert!(missing_issue
            .detail
            .contains(&reported_run_root.display().to_string()));
        assert!(missing_issue.detail.contains("expected_local_mirror="));
        assert_eq!(
            report.reported_run_root,
            reported_run_root.display().to_string()
        );
        assert!(report.available_run_roots.is_empty());
    }

    #[test]
    fn results_audit_flags_duplicate_local_run_roots() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let canonical_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let legacy_run_root = archive_root
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": canonical_run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        for run_root in [&canonical_run_root, &legacy_run_root] {
            write_json(
                &run_root.join("run_manifest.json"),
                serde_json::json!({
                    "stage_id": "fastq.validate_reads",
                    "scenario_id": "validation_fairness",
                    "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                    "dry_run": false,
                    "sample_limit": serde_json::Value::Null,
                    "samples_failed": 0,
                    "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
                }),
            );
        }
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "duplicate-result-root-ambiguity"));
    }

    #[test]
    fn results_audit_flags_newer_available_duplicate_run_root() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let canonical_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let legacy_run_root = archive_root
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": canonical_run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        for (run_root, generated_at_utc) in [
            (&canonical_run_root, "2026-03-28T00:00:00Z"),
            (&legacy_run_root, "2026-03-29T00:00:00Z"),
        ] {
            write_json(
                &run_root.join("run_manifest.json"),
                serde_json::json!({
                    "stage_id": "fastq.validate_reads",
                    "scenario_id": "validation_fairness",
                    "generated_at_utc": generated_at_utc,
                    "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                    "dry_run": false,
                    "sample_limit": serde_json::Value::Null,
                    "samples_failed": 0,
                    "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
                }),
            );
        }
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert_eq!(
            report.selected_run_root,
            canonical_run_root.display().to_string()
        );
        assert_eq!(
            report.newest_available_run_root,
            legacy_run_root.display().to_string()
        );
        assert!(!report.selected_run_root_is_newest);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "newer-run-root-available"));
    }

    #[test]
    fn results_audit_markdown_lists_selected_and_available_run_roots() {
        let rendered =
            super::render_published_results_markdown(&super::PublishedResultsStatusReport {
                corpus_id: "corpus-01".to_string(),
                applicable_stage_count: 1,
                published_stage_count: 1,
                complete_stage_count: 0,
                incomplete_stage_count: 1,
                issue_count: 1,
                stages: vec![super::PublishedResultsStageReport {
                    stage_id: "fastq.validate_reads".to_string(),
                    status: "incomplete".to_string(),
                    issue_count: 1,
                    reported_run_root:
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                        .to_string(),
                    selected_run_root:
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                        .to_string(),
                    newest_available_run_root:
                        "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                        .to_string(),
                    selected_run_root_is_newest: false,
                    available_run_roots: vec![
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                        "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                    ],
                    issues: vec![super::StageResultIssue {
                        stage_id: "fastq.validate_reads".to_string(),
                        issue_id: "missing-local-run-root".to_string(),
                        detail: "missing local mirror".to_string(),
                    }],
                }],
            });
        assert!(rendered.contains("selected run root"));
        assert!(rendered.contains(
            "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
        ));
        assert!(rendered.contains(
            "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
        ));
    }

    #[test]
    fn observed_tools_from_report_collects_nested_tool_literals() {
        let temp = tempdir().expect("tempdir");
        let report_path = temp.path().join("report.json");
        write_json(
            &report_path,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"parameters": {"tool": "seqtk"}}},
                    {"context": {"tool": "fastqvalidator"}},
                ],
            }),
        );
        let observed_tools = super::observed_tools_from_report(&report_path).expect("tools");
        assert_eq!(observed_tools, vec!["fastqvalidator", "seqtk"]);
    }

    #[test]
    fn results_audit_flags_polluting_mirror_artifacts() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let run_root = temp
            .path()
            .join("mirror")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        fs::create_dir_all(run_root.join("bench")).expect("create bench");
        fs::write(run_root.join("bench").join(".DS_Store"), "").expect("write ds store");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        write_json(
            &run_root.join("run_manifest.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                "dry_run": false,
                "sample_limit": serde_json::Value::Null,
                "samples_failed": 0,
                "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
            }),
        );
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "polluting-mirror-artifact"));
    }

    #[test]
    fn publication_docs_report_missing_stage_artifacts() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 1\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 1\n",
            ),
        )
        .expect("write corpus spec");
        let stage_root = docs_root.join("fastq.validate_reads");
        let corpus_root = stage_root.join("corpus-01");
        fs::create_dir_all(&corpus_root).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        write_json(
            &corpus_root.join("summary.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
            }),
        );
        fs::write(corpus_root.join("sample_results.csv"), "sample_id,tool\n").expect("sample csv");
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[validate_reads_contract()],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert_eq!(validate_report.status, "incomplete");
        assert!(validate_report.issue_count >= 4);
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "missing-benchmark-md"));
    }

    #[test]
    fn publication_docs_markdown_summarizes_completion_and_issue_count() {
        let markdown =
            super::render_publication_docs_markdown(&super::BenchmarkPublicationStatusReport {
                corpus_id: "corpus-01".to_string(),
                docs_root: "/bench/docs/benchmark".to_string(),
                benchmarkable_stage_count: 3,
                applicable_stage_count: 2,
                completed_stage_count: 1,
                incomplete_stage_count: 1,
                excluded_stage_count: 1,
                issue_count: 3,
                audit_warning_count: 0,
                audit_warnings: Vec::new(),
                supplemental_findings_generated_at_utc: None,
                excluded_stages: vec![super::ExcludedStageEntry {
                    stage_id: "fastq.index_reference".to_string(),
                    reason: "reference bundle benchmark".to_string(),
                }],
                stages: vec![
                    super::PublicationStageReport {
                        stage_id: "fastq.validate_reads".to_string(),
                        scenario_id: "validation_fairness".to_string(),
                        sample_scope: "full".to_string(),
                        contract_tool_roster: vec!["fastqvalidator".to_string()],
                        expected_tool_roster: vec!["fastqvalidator".to_string()],
                        method_path: "benchmark/fastq.validate_reads/corpus-01-method.md"
                            .to_string(),
                        corpus_path: "benchmark/fastq.validate_reads/corpus-01".to_string(),
                        status: "complete".to_string(),
                        issue_count: 0,
                        results_status: "complete".to_string(),
                        results_issue_count: 0,
                        results_selected_run_root:
                            "/bench/results/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                        results_newest_available_run_root:
                            "/bench/results/fastq.validate_reads/cluster-apptainer".to_string(),
                        results_selected_run_root_is_newest: true,
                        issues: Vec::new(),
                    },
                    super::PublicationStageReport {
                        stage_id: "fastq.trim_reads".to_string(),
                        scenario_id: "trim_fairness".to_string(),
                        sample_scope: "full".to_string(),
                        contract_tool_roster: vec!["fastp".to_string()],
                        expected_tool_roster: vec!["fastp".to_string()],
                        method_path: "benchmark/fastq.trim_reads/corpus-01-method.md".to_string(),
                        corpus_path: "benchmark/fastq.trim_reads/corpus-01".to_string(),
                        status: "incomplete".to_string(),
                        issue_count: 3,
                        results_status: "incomplete".to_string(),
                        results_issue_count: 2,
                        results_selected_run_root:
                            "/bench/results/fastq.trim_reads/cluster-apptainer"
                            .to_string(),
                        results_newest_available_run_root:
                            "/bench/archive/fastq.trim_reads/cluster-apptainer"
                            .to_string(),
                        results_selected_run_root_is_newest: false,
                        issues: vec![super::StageAuditIssue {
                            stage_id: "fastq.trim_reads".to_string(),
                            issue_id: "missing-corpus-dir".to_string(),
                            severity: "error".to_string(),
                            detail: "missing docs/benchmark/fastq.trim_reads/corpus-01".to_string(),
                        }],
                    },
                ],
            });
        assert!(markdown.contains("Benchmarkable governed stages: `3`"));
        assert!(markdown.contains("Completed stage dossiers: `1`"));
        assert!(markdown.contains("Publication issues: `3`"));
        assert!(markdown.contains(
            "`fastq.trim_reads`: `incomplete` (`3` publication issues, results `incomplete`, scope `full`)"
        ));
        assert!(
            markdown.contains(
                "selected mirrored run root: `/bench/results/fastq.trim_reads/cluster-apptainer`"
            )
        );
        assert!(markdown.contains(
            "newest mirrored run root: `/bench/archive/fastq.trim_reads/cluster-apptainer` (selected newest=`false`)"
        ));
        assert!(markdown.contains("mirrored result issues: `2`"));
        assert!(markdown.contains("`fastq.index_reference`: reference bundle benchmark"));
    }

    #[test]
    fn publication_docs_append_supplemental_findings() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 0\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 0\n",
            ),
        )
        .expect("write corpus spec");
        let stage_root = docs_root.join("fastq.validate_reads");
        fs::create_dir_all(stage_root.join("corpus-01")).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        let mut supplemental = BTreeMap::new();
        supplemental.insert(
            "fastq.validate_reads".to_string(),
            vec![super::StageAuditIssue {
                stage_id: "fastq.validate_reads".to_string(),
                issue_id: "fixture-integrity-gap".to_string(),
                severity: "error".to_string(),
                detail: "synthetic fixture does not represent a publishable benchmark lineage"
                    .to_string(),
            }],
        );
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[
                crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                    sample_scope: "paired".to_string(),
                    ..validate_reads_contract()
                },
            ],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &supplemental,
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "fixture-integrity-gap"));
    }

    #[test]
    fn load_supplemental_findings_warns_when_freshness_missing() {
        let temp = tempdir().expect("tempdir");
        let findings_path = temp.path().join("findings.json");
        write_json(
            &findings_path,
            serde_json::json!({
                "findings": [{
                    "stage_id": "fastq.validate_reads",
                    "issue_id": "fixture-gap",
                    "detail": "fixture gap",
                }],
            }),
        );
        let (findings, warnings, generated_at_utc) =
            super::load_supplemental_findings(&findings_path).expect("findings");
        assert!(findings.contains_key("fastq.validate_reads"));
        assert_eq!(generated_at_utc, None);
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("generated_at_utc")));
    }

    #[test]
    fn publication_docs_reject_missing_tool_coverage_in_sample_results() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 1\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 1\n",
            ),
        )
        .expect("write corpus spec");
        let stage_root = docs_root.join("fastq.validate_reads");
        let corpus_root = stage_root.join("corpus-01");
        fs::create_dir_all(&corpus_root).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        write_json(
            &corpus_root.join("summary.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "tools": ["fastqvalidator", "seqtk"],
                "samples_total": 4,
                "samples_failed": 0,
                "cohort_counts": {
                    "ancient_pe": 1,
                    "ancient_se": 1,
                    "modern_pe": 1,
                    "modern_se": 1,
                },
                "tool_summary": [
                    {"tool": "fastqvalidator"},
                    {"tool": "seqtk"},
                ],
            }),
        );
        fs::write(
            corpus_root.join("sample_results.csv"),
            concat!(
                "sample_id,accession,era,layout,study_accession,size_band,tool\n",
                "sample_0001,ACC1,ancient,se,PRJ1,under_100mb,fastqvalidator\n",
                "sample_0002,ACC2,ancient,pe,PRJ2,under_100mb,fastqvalidator\n",
                "sample_0003,ACC3,modern,se,PRJ3,under_500mb,fastqvalidator\n",
                "sample_0004,ACC4,modern,pe,PRJ4,under_500mb,fastqvalidator\n",
            ),
        )
        .expect("sample csv");
        fs::write(
            corpus_root.join("tool_runtime_summary.csv"),
            "tool\nfastqvalidator\nseqtk\n",
        )
        .expect("tool summary");
        fs::write(
            corpus_root.join("cohort_runtime_summary.csv"),
            "cohort\nancient_pe\nancient_se\nmodern_pe\nmodern_se\n",
        )
        .expect("cohort summary");
        fs::write(
            corpus_root.join("sample_runtime_outliers.csv"),
            "sample_id\nsample_0001\nsample_0002\nsample_0003\nsample_0004\n",
        )
        .expect("outliers");
        fs::write(corpus_root.join("benchmark.md"), "# dossier\n").expect("dossier");
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[
                crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                    stage_id: "fastq.validate_reads".to_string(),
                    scenario_id: "validation_fairness".to_string(),
                    sample_scope: "full".to_string(),
                    tools: vec!["fastqvalidator".to_string(), "seqtk".to_string()],
                },
            ],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "sample-results-tool-coverage-drift"));
    }

    #[test]
    fn remediation_queue_merges_publication_results_and_findings() {
        let queue = super::build_remediation_queue(
            "corpus-01",
            &[
                crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                    stage_id: "fastq.validate_reads".to_string(),
                    scenario_id: "governed-fixture".to_string(),
                    sample_scope: "paired-subset".to_string(),
                    tools: Vec::new(),
                },
            ],
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "status": "incomplete",
                    "issues": [{
                        "issue_id": "missing-benchmark-md",
                        "detail": "missing docs dossier",
                    }],
                }],
            }),
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "status": "incomplete",
                    "issues": [{
                        "issue_id": "missing-local-run-root",
                        "detail": "missing local mirror root",
                    }],
                }],
            }),
            &serde_json::json!({
                "findings": [{
                    "stage_id": "fastq.validate_reads",
                    "issue_id": "publication-gap",
                    "detail": "supplemental finding",
                    "severity": "error",
                }],
            }),
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "generated_at_utc": "2026-03-28T00:00:00Z",
                    "run_root_source": "local-results-root",
                }],
            }),
        )
        .expect("remediation queue");

        let stage = queue.stages.first().expect("stage");
        assert_eq!(stage.stage_id, "fastq.validate_reads");
        assert_eq!(stage.status, "open");
        assert_eq!(stage.issue_count, 3);
        assert_eq!(stage.recommended_action, "sync-or-normalize-results");
        assert_eq!(
            stage.published_generated_at_utc.as_deref(),
            Some("2026-03-28T00:00:00Z")
        );
        assert_eq!(stage.run_root_source.as_deref(), Some("local-results-root"));
    }

    #[test]
    fn remediation_queue_markdown_uses_issue_groups() {
        let rendered = super::render_remediation_queue_markdown(&super::RemediationQueue {
            corpus_id: "corpus-01".to_string(),
            stage_count: 1,
            open_stage_count: 1,
            clear_stage_count: 0,
            stages: vec![super::RemediationStageEntry {
                stage_id: "fastq.validate_reads".to_string(),
                owner: "benchmark-governance".to_string(),
                status: "open".to_string(),
                issue_count: 2,
                issue_group_count: 1,
                recommended_action: "sync-or-normalize-results".to_string(),
                publication_status: "incomplete".to_string(),
                results_status: "incomplete".to_string(),
                sample_scope: "paired-subset".to_string(),
                published_generated_at_utc: Some("2026-03-28T00:00:00Z".to_string()),
                run_root_source: Some("local-cache-mirror".to_string()),
                issue_groups: vec![super::RemediationIssueGroup {
                    issue_id: "missing-localized-report-json".to_string(),
                    count: 2,
                    sources: vec!["results".to_string()],
                    severity: "error".to_string(),
                    example_details: vec![
                        "sample_0001 missing report.json".to_string(),
                        "sample_0002 missing report.json".to_string(),
                    ],
                    additional_detail_count: 0,
                }],
                issues: vec![
                    super::RemediationIssue {
                        issue_id: "missing-localized-report-json".to_string(),
                        detail: "sample_0001 missing report.json".to_string(),
                        severity: "error".to_string(),
                        source: "results".to_string(),
                    },
                    super::RemediationIssue {
                        issue_id: "missing-localized-report-json".to_string(),
                        detail: "sample_0002 missing report.json".to_string(),
                        severity: "error".to_string(),
                        source: "results".to_string(),
                    },
                ],
            }],
        });

        assert!(rendered.contains("issue group `missing-localized-report-json` x2"));
        assert!(rendered.contains("sample_0001 missing report.json"));
    }
}
