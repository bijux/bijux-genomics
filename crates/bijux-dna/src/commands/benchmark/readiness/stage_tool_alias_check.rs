use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_candidate_rows,
    collect_all_domain_active_stage_tool_matrix_rows,
};
use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::all_domain_rendered_commands::collect_all_domain_rendered_command_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;
use bijux_dna_api::v1::api::bench::{
    derive_benchmark_stage_tool_alias_violations, derive_benchmark_tool_alias_clusters,
    legacy_benchmark_stage_aliases, BenchmarkStageToolAliasViolation as StageToolAliasViolationRow,
    BenchmarkToolAliasCluster,
};

pub(crate) const DEFAULT_STAGE_TOOL_ALIAS_CHECK_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/stage-tool-alias-check.json";
const STAGE_TOOL_ALIAS_CHECK_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.stage_tool_alias_check.v1";
const ACCEPTED_SCOPE_MIGRATION_VALIDATION_ONLY: &str = "migration_validation_only";
const ALIAS_KIND_STAGE: &str = "stage";
const ALIAS_KIND_TOOL: &str = "tool";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationValidationStageAliasRow {
    pub(crate) alias_stage_id: String,
    pub(crate) canonical_stage_id: String,
    pub(crate) accepted_scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageToolAliasCheckReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) migration_validation_stage_alias_count: usize,
    pub(crate) tool_alias_cluster_count: usize,
    pub(crate) candidate_row_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) expected_result_row_count: usize,
    pub(crate) rendered_command_row_count: usize,
    pub(crate) surface_violation_counts: BTreeMap<String, usize>,
    pub(crate) alias_kind_violation_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) migration_validation_stage_aliases: Vec<MigrationValidationStageAliasRow>,
    pub(crate) tool_alias_clusters: Vec<BenchmarkToolAliasCluster>,
    pub(crate) violations: Vec<StageToolAliasViolationRow>,
}

pub(crate) fn run_render_stage_tool_alias_check(
    args: &parse::BenchReadinessRenderStageToolAliasCheckArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_tool_alias_check(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_TOOL_ALIAS_CHECK_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_stage_tool_alias_check(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageToolAliasCheckReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_stage_tool_alias_check_report(repo_root, &output_path)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload =
        serde_json::to_vec_pretty(&report).context("serialize stage-tool alias check report")?;
    fs::write(&output_path, payload).with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!(
            "stale benchmark stage/tool aliases leaked into governed active job surfaces"
        ));
    }
    Ok(report)
}

fn build_stage_tool_alias_check_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<StageToolAliasCheckReport> {
    let candidate_rows = collect_all_domain_active_stage_tool_matrix_candidate_rows(repo_root)?;
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let rendered_rows = collect_all_domain_rendered_command_rows(repo_root)?;

    let migration_validation_stage_aliases = legacy_benchmark_stage_aliases()
        .iter()
        .map(|alias| MigrationValidationStageAliasRow {
            alias_stage_id: alias.alias_stage_id.to_string(),
            canonical_stage_id: alias.canonical_stage_id.to_string(),
            accepted_scope: ACCEPTED_SCOPE_MIGRATION_VALIDATION_ONLY.to_string(),
        })
        .collect::<Vec<_>>();
    let tool_alias_clusters =
        derive_benchmark_tool_alias_clusters(candidate_rows.iter().map(|row| row.tool_id.as_str()));
    let violations = derive_benchmark_stage_tool_alias_violations(
        &candidate_rows,
        &active_rows,
        &expected_rows,
        &rendered_rows,
        &tool_alias_clusters,
    );

    let mut surface_violation_counts = BTreeMap::<String, usize>::new();
    let mut alias_kind_violation_counts = BTreeMap::<String, usize>::new();
    for violation in &violations {
        *surface_violation_counts.entry(violation.surface.clone()).or_default() += 1;
        *alias_kind_violation_counts.entry(violation.alias_kind.clone()).or_default() += 1;
    }

    let report = StageToolAliasCheckReport {
        schema_version: STAGE_TOOL_ALIAS_CHECK_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        migration_validation_stage_alias_count: migration_validation_stage_aliases.len(),
        tool_alias_cluster_count: tool_alias_clusters.len(),
        candidate_row_count: candidate_rows.len(),
        active_row_count: active_rows.len(),
        expected_result_row_count: expected_rows.len(),
        rendered_command_row_count: rendered_rows.len(),
        surface_violation_counts,
        alias_kind_violation_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        migration_validation_stage_aliases,
        tool_alias_clusters,
        violations,
    };
    ensure_stage_tool_alias_check_contract(&report)?;
    Ok(report)
}

fn ensure_stage_tool_alias_check_contract(report: &StageToolAliasCheckReport) -> Result<()> {
    if report.migration_validation_stage_alias_count
        != report.migration_validation_stage_aliases.len()
    {
        return Err(anyhow!(
            "stage-tool alias check drifted from its migration-validation stage alias inventory"
        ));
    }
    if report.tool_alias_cluster_count != report.tool_alias_clusters.len() {
        return Err(anyhow!(
            "stage-tool alias check drifted from its tool alias cluster inventory"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!("stage-tool alias check drifted from its violation inventory"));
    }
    if report.migration_validation_stage_aliases.is_empty() {
        return Err(anyhow!(
            "stage-tool alias check must keep an explicit migration-validation stage alias inventory"
        ));
    }
    if report
        .migration_validation_stage_aliases
        .iter()
        .map(|row| row.alias_stage_id.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        != report.migration_validation_stage_aliases.len()
    {
        return Err(anyhow!(
            "stage-tool alias check cannot keep duplicate migration-validation stage aliases"
        ));
    }
    if report.migration_validation_stage_aliases.iter().any(|row| {
        row.accepted_scope != ACCEPTED_SCOPE_MIGRATION_VALIDATION_ONLY
            || row.alias_stage_id == row.canonical_stage_id
    }) {
        return Err(anyhow!(
            "stage-tool alias check kept an invalid migration-validation stage alias row"
        ));
    }
    if report.tool_alias_clusters.iter().any(|cluster| {
        cluster.alias_tool_ids.is_empty()
            || cluster.alias_tool_ids.contains(&cluster.canonical_tool_id)
    }) {
        return Err(anyhow!("stage-tool alias check kept an invalid tool alias cluster"));
    }
    if report.violations.iter().any(|violation| {
        (violation.alias_kind != ALIAS_KIND_STAGE && violation.alias_kind != ALIAS_KIND_TOOL)
            || violation.alias_id.trim().is_empty()
            || violation.canonical_id.trim().is_empty()
    }) {
        return Err(anyhow!("stage-tool alias check kept an invalid alias violation row"));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!("stage-tool alias check cannot be ok while alias violations remain"));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!("stage-tool alias check must keep explicit violations when failing"));
    }
    Ok(())
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
