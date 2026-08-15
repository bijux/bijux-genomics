//! Owner: bijux-dna-bench
//! Benchmark workspace layout reporting and governed status publication.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkConfig, WorkspaceAuthoritativeRoots, WorkspaceLayoutIssue, WorkspaceLayoutReport,
    WorkspaceLocalStageLayout, WorkspaceRootPairSummary,
};

/// Derive the complete benchmark workspace layout report from the normalized configuration.
///
/// # Errors
/// Returns an error when required local workspace roots or the corpus directory identity are
/// absent or invalid.
pub fn benchmark_workspace_layout_report(
    config: &BenchmarkConfig,
) -> Result<WorkspaceLayoutReport> {
    let workspace = &config.workspace;
    let local_results_root = workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))?;
    let local_cache_mirror_root = workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))?;
    let remote_workspace_root = local_cache_mirror_root
        .parent()
        .ok_or_else(|| anyhow!("invalid local.cache_mirror_root"))?
        .to_path_buf();
    let corpus_dir_name = benchmark_workspace_layout_corpus_dir_name(config)?;

    let root_pairs = vec![
        summarize_benchmark_workspace_root_pair(
            "remote-results",
            &local_cache_mirror_root.join("results"),
            &local_cache_mirror_root.join("bijux-dna-results"),
        ),
        summarize_benchmark_workspace_root_pair(
            "remote-reference",
            &local_cache_mirror_root.join("reference"),
            &local_cache_mirror_root.join("bijux-reference"),
        ),
    ];

    let mut issues = Vec::new();
    for pair in &root_pairs {
        if pair.status == "duplicate" {
            issues.push(WorkspaceLayoutIssue {
                issue_id: format!("duplicate-{}-root", pair.scope),
                detail: format!("both {} and {} exist", pair.canonical_root, pair.legacy_root),
            });
        }
    }

    let mut unexpected_remote_siblings = Vec::new();
    for sibling in ["results", corpus_dir_name.as_str(), "extra-data"] {
        let sibling_root = remote_workspace_root.join(sibling);
        if sibling_root.exists() {
            let detail = sibling_root.display().to_string();
            unexpected_remote_siblings.push(detail.clone());
            issues.push(WorkspaceLayoutIssue {
                issue_id: "unexpected-remote-workspace-sibling".to_string(),
                detail: format!("unexpected sibling beside .cache: {detail}"),
            });
        }
    }

    let local_stage_layout = summarize_local_stage_layout(
        &local_results_root,
        &local_cache_mirror_root,
        &corpus_dir_name,
    );
    for stage_id in &local_stage_layout.shared_stage_ids {
        issues.push(WorkspaceLayoutIssue {
            issue_id: "duplicate-local-stage-root".to_string(),
            detail: format!(
                "both {}/{} and {}/{} exist",
                local_stage_layout.cache_corpus_root,
                stage_id,
                local_stage_layout.archive_corpus_root,
                stage_id,
            ),
        });
    }
    for stage_id in &local_stage_layout.archive_only_stage_ids {
        issues.push(WorkspaceLayoutIssue {
            issue_id: "archive-only-local-stage-root".to_string(),
            detail: format!(
                "{}/{} exists outside the governed cache mirror root {}",
                local_stage_layout.archive_corpus_root,
                stage_id,
                local_stage_layout.cache_corpus_root,
            ),
        });
    }

    Ok(WorkspaceLayoutReport {
        local_results_root: local_results_root.display().to_string(),
        local_cache_mirror_root: local_cache_mirror_root.display().to_string(),
        remote_workspace_root: remote_workspace_root.display().to_string(),
        authoritative_roots: WorkspaceAuthoritativeRoots {
            remote_results_root: local_cache_mirror_root.join("results").display().to_string(),
            remote_reference_root: local_cache_mirror_root.join("reference").display().to_string(),
            local_stage_root: local_stage_layout.authoritative_stage_root.clone(),
        },
        root_pairs,
        unexpected_remote_siblings,
        local_stage_layout,
        status: if issues.is_empty() { "clear".to_string() } else { "incomplete".to_string() },
        issue_count: issues.len(),
        issues,
    })
}

/// Publish the JSON and Markdown workspace layout reports beneath the governed documentation root.
///
/// # Errors
/// Returns an error when report derivation, serialization, directory creation, or atomic output
/// publication fails.
pub fn publish_benchmark_workspace_layout_status(
    config: &BenchmarkConfig,
    docs_root: &Path,
) -> Result<()> {
    let report = benchmark_workspace_layout_report(config)?;
    bijux_dna_infra::ensure_dir(docs_root)
        .with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join("workspace-layout-status.json");
    let json = format!("{}\n", serde_json::to_string_pretty(&report)?);
    bijux_dna_infra::atomic_write_bytes(&json_path, json.as_bytes())
        .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join("workspace-layout-status.md");
    let markdown = render_benchmark_workspace_layout_markdown(&report);
    bijux_dna_infra::atomic_write_bytes(&markdown_path, markdown.as_bytes())
        .with_context(|| format!("write {}", markdown_path.display()))
}

/// Summarize a canonical and legacy workspace root pair deterministically.
#[must_use]
pub fn summarize_benchmark_workspace_root_pair(
    scope: &str,
    canonical_root: &Path,
    legacy_root: &Path,
) -> WorkspaceRootPairSummary {
    let canonical_entries = entry_names(canonical_root);
    let legacy_entries = entry_names(legacy_root);
    let canonical_set = canonical_entries.iter().cloned().collect::<BTreeSet<_>>();
    let legacy_set = legacy_entries.iter().cloned().collect::<BTreeSet<_>>();
    WorkspaceRootPairSummary {
        scope: scope.to_string(),
        canonical_root: canonical_root.display().to_string(),
        legacy_root: legacy_root.display().to_string(),
        canonical_exists: canonical_root.exists(),
        legacy_exists: legacy_root.exists(),
        canonical_entries: canonical_entries.clone(),
        legacy_entries: legacy_entries.clone(),
        shared_entries: canonical_set.intersection(&legacy_set).cloned().collect(),
        canonical_only_entries: canonical_set.difference(&legacy_set).cloned().collect(),
        legacy_only_entries: legacy_set.difference(&canonical_set).cloned().collect(),
        status: if canonical_root.exists() && legacy_root.exists() {
            "duplicate".to_string()
        } else {
            "clear".to_string()
        },
    }
}

/// Resolve the stable corpus directory identity used by workspace layout reports.
///
/// # Errors
/// Returns an error when neither a remote corpus root nor a declared corpus is available.
pub fn benchmark_workspace_layout_corpus_dir_name(config: &BenchmarkConfig) -> Result<String> {
    if let Some(dir_name) = config
        .workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(dir_name.to_string());
    }
    if let Some(corpus_id) = config.corpora.keys().next() {
        return Ok(corpus_id.replace('-', "_"));
    }
    Err(anyhow!(
        "benchmark config must declare workspace.remote.corpus_root or at least one [corpora] entry"
    ))
}

/// Render a deterministic Markdown presentation of a workspace layout report.
#[must_use]
pub fn render_benchmark_workspace_layout_markdown(report: &WorkspaceLayoutReport) -> String {
    let mut lines = vec![
        "# Benchmark Workspace Layout Status".to_string(),
        String::new(),
        format!("- Local results root: `{}`", report.local_results_root),
        format!("- Local cache mirror root: `{}`", report.local_cache_mirror_root),
        format!("- Mirrored remote workspace root: `{}`", report.remote_workspace_root),
        format!(
            "- Authoritative remote results root: `{}`",
            report.authoritative_roots.remote_results_root
        ),
        format!(
            "- Authoritative remote reference root: `{}`",
            report.authoritative_roots.remote_reference_root
        ),
        format!(
            "- Authoritative local publication root: `{}`",
            report.authoritative_roots.local_stage_root
        ),
        format!("- Status: `{}`", report.status),
        format!("- Issues: `{}`", report.issue_count),
        String::new(),
        "## Root Pairs".to_string(),
        String::new(),
    ];
    for pair in &report.root_pairs {
        lines.push(format!(
            "- `{}`: `{}` (canonical `{}`, legacy `{}`)",
            pair.scope, pair.status, pair.canonical_root, pair.legacy_root
        ));
        if !pair.canonical_entries.is_empty() {
            lines.push(format!(
                "  - canonical entries: {}",
                quoted_entries(&pair.canonical_entries)
            ));
        }
        if !pair.legacy_entries.is_empty() {
            lines.push(format!("  - legacy entries: {}", quoted_entries(&pair.legacy_entries)));
        }
        if !pair.shared_entries.is_empty() {
            lines.push(format!("  - shared entries: {}", quoted_entries(&pair.shared_entries)));
        }
    }
    lines.extend([
        String::new(),
        "## Local Stage Layout".to_string(),
        String::new(),
        format!("- Archive corpus root: `{}`", report.local_stage_layout.archive_corpus_root),
        format!("- Cache corpus root: `{}`", report.local_stage_layout.cache_corpus_root),
    ]);
    if !report.local_stage_layout.shared_stage_ids.is_empty() {
        lines.push(format!(
            "- Shared stage ids: {}",
            quoted_entries(&report.local_stage_layout.shared_stage_ids)
        ));
    }
    if !report.local_stage_layout.archive_only_stage_ids.is_empty() {
        lines.push(format!(
            "- Archive-only stage ids: {}",
            quoted_entries(&report.local_stage_layout.archive_only_stage_ids)
        ));
    }
    if !report.local_stage_layout.cache_only_stage_ids.is_empty() {
        lines.push(format!(
            "- Cache-only stage ids: {}",
            quoted_entries(&report.local_stage_layout.cache_only_stage_ids)
        ));
    }
    lines.join("\n") + "\n"
}

fn summarize_local_stage_layout(
    local_results_root: &Path,
    local_cache_mirror_root: &Path,
    corpus_dir_name: &str,
) -> WorkspaceLocalStageLayout {
    let archive_corpus_root = local_results_root.join(corpus_dir_name);
    let cache_corpus_root = local_cache_mirror_root.join("results").join(corpus_dir_name);
    let archive_stage_ids = entry_names(&archive_corpus_root);
    let cache_stage_ids = entry_names(&cache_corpus_root);
    let archive_set = archive_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    let cache_set = cache_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    WorkspaceLocalStageLayout {
        archive_corpus_root: archive_corpus_root.display().to_string(),
        cache_corpus_root: cache_corpus_root.display().to_string(),
        archive_stage_ids: archive_stage_ids.clone(),
        cache_stage_ids: cache_stage_ids.clone(),
        shared_stage_ids: archive_set.intersection(&cache_set).cloned().collect(),
        archive_only_stage_ids: archive_set.difference(&cache_set).cloned().collect(),
        cache_only_stage_ids: cache_set.difference(&archive_set).cloned().collect(),
        authoritative_stage_root: cache_corpus_root.display().to_string(),
    }
}

fn entry_names(root: &Path) -> Vec<String> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut entries = root
        .read_dir()
        .ok()
        .into_iter()
        .flat_map(|iter| iter.filter_map(Result::ok))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn quoted_entries(entries: &[String]) -> String {
    entries.iter().map(|entry| format!("`{entry}`")).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        benchmark_workspace_layout_corpus_dir_name, benchmark_workspace_layout_report,
        publish_benchmark_workspace_layout_status, summarize_benchmark_workspace_root_pair,
    };
    use bijux_dna_bench_model::{
        BenchmarkConfig, BenchmarkCorpusConfig, BenchmarkWorkspaceConfig, BenchmarkWorkspaceLocal,
        BenchmarkWorkspaceRemote,
    };

    fn config(results_root: &Path, cache_root: &Path) -> BenchmarkConfig {
        BenchmarkConfig {
            workspace: BenchmarkWorkspaceConfig {
                local: Some(BenchmarkWorkspaceLocal {
                    results_root: Some(results_root.display().to_string()),
                    cache_mirror_root: Some(cache_root.display().to_string()),
                    ..BenchmarkWorkspaceLocal::default()
                }),
                remote: Some(BenchmarkWorkspaceRemote {
                    corpus_root: Some("/remote/cache/benchmark_corpus".to_string()),
                    ..BenchmarkWorkspaceRemote::default()
                }),
                ..BenchmarkWorkspaceConfig::default()
            },
            ..BenchmarkConfig::default()
        }
    }

    use std::path::Path;

    #[test]
    fn root_pair_summary_marks_shared_roots_as_duplicate() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical");
        let legacy = temp.path().join("legacy");
        std::fs::create_dir_all(canonical.join("results"))?;
        std::fs::create_dir_all(legacy.join("results"))?;

        let summary =
            summarize_benchmark_workspace_root_pair("remote-results", &canonical, &legacy);
        assert_eq!(summary.status, "duplicate");
        assert_eq!(summary.shared_entries, vec!["results"]);
        Ok(())
    }

    #[test]
    fn layout_report_and_publication_share_the_benchmark_authority() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let results_root = temp.path().join("archive");
        let cache_root = temp.path().join("mirror/.cache");
        std::fs::create_dir_all(results_root.join("benchmark_corpus/fastq.trim_reads"))?;
        std::fs::create_dir_all(cache_root.join("results/benchmark_corpus/fastq.trim_reads"))?;
        let config = config(&results_root, &cache_root);

        let report = benchmark_workspace_layout_report(&config)?;
        assert_eq!(report.status, "incomplete");
        assert!(report.issues.iter().any(|issue| issue.issue_id == "duplicate-local-stage-root"));

        let docs_root = temp.path().join("docs");
        publish_benchmark_workspace_layout_status(&config, &docs_root)?;
        let published: bijux_dna_bench_model::WorkspaceLayoutReport = serde_json::from_slice(
            &std::fs::read(docs_root.join("workspace-layout-status.json"))?,
        )?;
        assert_eq!(published, report);
        assert!(docs_root.join("workspace-layout-status.md").is_file());
        Ok(())
    }

    #[test]
    fn corpus_directory_falls_back_to_declared_corpus_identity() -> anyhow::Result<()> {
        let mut config = BenchmarkConfig::default();
        config
            .corpora
            .insert("human-short-read-diversity".to_string(), BenchmarkCorpusConfig::default());
        assert_eq!(
            benchmark_workspace_layout_corpus_dir_name(&config)?,
            "human_short_read_diversity"
        );
        Ok(())
    }

    #[test]
    fn corpus_directory_selection_is_sorted_and_deterministic() -> anyhow::Result<()> {
        let config = BenchmarkConfig {
            corpora: BTreeMap::from([
                ("zeta-corpus".to_string(), BenchmarkCorpusConfig::default()),
                ("alpha-corpus".to_string(), BenchmarkCorpusConfig::default()),
            ]),
            ..BenchmarkConfig::default()
        };
        assert_eq!(benchmark_workspace_layout_corpus_dir_name(&config)?, "alpha_corpus");
        Ok(())
    }
}
