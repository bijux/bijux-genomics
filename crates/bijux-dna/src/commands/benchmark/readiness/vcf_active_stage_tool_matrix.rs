use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows;
use super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows;
use super::vcf_normalized_metrics_schema::collect_vcf_normalized_metrics_schema_report_rows;
use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::{
    build_vcf_stage_matrix_rows, VcfStageMatrixRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH: &str =
    "benchmarks/readiness/vcf/vcf-active-stage-tool-matrix.tsv";
const VCF_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_active_stage_tool_matrix.v1";
const ACTIVE_SCOPE_STATE: &str = "active";
const COMPLETE_SCOPE_STATE: &str = "complete";
const REMOVED_FROM_SCOPE_STATE: &str = "removed_from_scope";
const ACTIVE_SCOPE_DETAIL: &str = "active";
const COMPLETE_SCOPE_DETAIL: &str = "complete";
const LIFECYCLE_NOT_ACTIVE_SCOPE_DETAIL: &str = "lifecycle_not_active";
const BENCHMARK_NOT_READY_SCOPE_DETAIL: &str = "benchmark_not_ready";
const ACTIVE_SCOPE_PROOF_PATH: &str =
    "benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv";
const COMPLETE_SCOPE_PROOF_PATH: &str = "benchmarks/readiness/vcf-expected-benchmark-results.tsv";
const LIFECYCLE_NOT_ACTIVE_SCOPE_PROOF_PATH: &str =
    "benchmarks/readiness/all-domains/no-planned-rows.json";
const BENCHMARK_NOT_READY_SCOPE_PROOF_PATH: &str =
    "benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfActiveStageToolMatrixRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) stage_support_status: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) scope_state: String,
    pub(crate) scope_detail: String,
    pub(crate) scope_proof_path: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfActiveStageToolMatrixReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) removed_row_count: usize,
    pub(crate) scope_state_counts: BTreeMap<String, usize>,
    pub(crate) scope_detail_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfActiveStageToolMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RetainedVcfBinding {
    stage_id: String,
    tool_id: String,
    tool_statuses: BTreeSet<String>,
}

pub(crate) fn run_render_vcf_active_stage_tool_matrix(
    args: &parse::BenchReadinessRenderVcfActiveStageToolMatrixArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_active_stage_tool_matrix(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_active_stage_tool_matrix(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfActiveStageToolMatrixReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_active_stage_tool_matrix_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_active_stage_tool_matrix_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut scope_state_counts = BTreeMap::<String, usize>::new();
    let mut scope_detail_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *scope_state_counts.entry(row.scope_state.clone()).or_default() += 1;
        *scope_detail_counts.entry(row.scope_detail.clone()).or_default() += 1;
    }

    Ok(VcfActiveStageToolMatrixReport {
        schema_version: VCF_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        tool_count: rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len(),
        active_row_count: count_scope_state(&rows, ACTIVE_SCOPE_STATE),
        complete_row_count: count_scope_state(&rows, COMPLETE_SCOPE_STATE),
        removed_row_count: count_scope_state(&rows, REMOVED_FROM_SCOPE_STATE),
        scope_state_counts,
        scope_detail_counts,
        rows,
    })
}

pub(crate) fn collect_vcf_active_stage_tool_matrix_rows(
    repo_root: &Path,
) -> Result<Vec<VcfActiveStageToolMatrixRow>> {
    let retained_bindings = load_retained_vcf_bindings(repo_root)?;
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let matrix_by_stage = build_vcf_stage_matrix_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let schema_id_by_stage = collect_vcf_normalized_metrics_schema_report_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row.schema_id))
        .collect::<BTreeMap<_, _>>();
    let active_bindings = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "vcf")
        .map(|row| BindingKey { stage_id: row.stage_id, tool_id: row.tool_id })
        .collect::<BTreeSet<_>>();
    let complete_bindings = collect_vcf_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| BindingKey { stage_id: row.stage_id, tool_id: row.tool_id })
        .collect::<BTreeSet<_>>();

    let mut rows = Vec::with_capacity(retained_bindings.len());
    for binding in retained_bindings {
        let catalog_row = catalog_by_stage.get(binding.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF active-stage-tool matrix is missing stage catalog coverage for `{}`",
                binding.stage_id
            )
        })?;
        let stage_matrix_row = matrix_by_stage.get(binding.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF active-stage-tool matrix is missing stage matrix coverage for `{}`",
                binding.stage_id
            )
        })?;
        let schema_id = schema_id_by_stage.get(binding.stage_id.as_str()).cloned().ok_or_else(|| {
            anyhow!(
                "VCF active-stage-tool matrix is missing normalized metrics schema coverage for `{}`",
                binding.stage_id
            )
        })?;

        rows.push(build_vcf_active_stage_tool_matrix_row(
            binding,
            catalog_row,
            stage_matrix_row,
            &schema_id,
            &active_bindings,
            &complete_bindings,
        ));
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_vcf_active_stage_tool_matrix_contract(&rows)?;
    Ok(rows)
}

fn build_vcf_active_stage_tool_matrix_row(
    binding: RetainedVcfBinding,
    catalog_row: &VcfStageCatalogRow,
    stage_matrix_row: &VcfStageMatrixRow,
    schema_id: &str,
    active_bindings: &BTreeSet<BindingKey>,
    complete_bindings: &BTreeSet<BindingKey>,
) -> VcfActiveStageToolMatrixRow {
    let binding_key =
        BindingKey { stage_id: binding.stage_id.clone(), tool_id: binding.tool_id.clone() };
    let (scope_state, scope_detail, scope_proof_path, reason) = if active_bindings
        .contains(&binding_key)
    {
        (
            ACTIVE_SCOPE_STATE.to_string(),
            ACTIVE_SCOPE_DETAIL.to_string(),
            ACTIVE_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is part of the governed all-domain active benchmark matrix",
                binding.stage_id, binding.tool_id
            ),
        )
    } else if complete_bindings.contains(&binding_key) {
        (
            COMPLETE_SCOPE_STATE.to_string(),
            COMPLETE_SCOPE_DETAIL.to_string(),
            COMPLETE_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is already represented in the governed VCF expected-result surface even though it is not part of the final job-bearing active matrix",
                binding.stage_id, binding.tool_id
            ),
        )
    } else if catalog_row.support_status == "supported" {
        (
            REMOVED_FROM_SCOPE_STATE.to_string(),
            BENCHMARK_NOT_READY_SCOPE_DETAIL.to_string(),
            BENCHMARK_NOT_READY_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is retained for a supported stage but remains outside active scope because it is not benchmark ready",
                binding.stage_id, binding.tool_id
            ),
        )
    } else {
        (
            REMOVED_FROM_SCOPE_STATE.to_string(),
            LIFECYCLE_NOT_ACTIVE_SCOPE_DETAIL.to_string(),
            LIFECYCLE_NOT_ACTIVE_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is retained but remains outside active scope because the stage lifecycle is not active",
                binding.stage_id, binding.tool_id
            ),
        )
    };

    VcfActiveStageToolMatrixRow {
        stage_id: binding.stage_id,
        tool_id: binding.tool_id,
        tool_status: sorted_status_label(&binding.tool_statuses),
        stage_support_status: catalog_row.support_status.clone(),
        corpus_id: stage_matrix_row.corpus_id.clone(),
        asset_profile_id: stage_matrix_row.asset_profile_id.clone(),
        adapter_id: stage_matrix_row.adapter_id.clone(),
        parser_id: stage_matrix_row.parser_id.clone(),
        schema_id: schema_id.to_string(),
        scope_state,
        scope_detail,
        scope_proof_path,
        reason,
    }
}

fn load_retained_vcf_bindings(repo_root: &Path) -> Result<Vec<RetainedVcfBinding>> {
    let mut bindings = BTreeMap::<BindingKey, BTreeSet<String>>::new();
    for relative_path in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let raw = fs::read_to_string(repo_root.join(relative_path))
            .with_context(|| format!("read {}", repo_root.join(relative_path).display()))?;
        let parsed: toml::Value = toml::from_str(&raw)
            .with_context(|| format!("parse {}", repo_root.join(relative_path).display()))?;
        let entries = parsed
            .get("tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing tools in {relative_path}"))?;
        for entry in entries {
            let tool_id = entry
                .get("id")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| anyhow!("tool entry in {relative_path} is missing id"))?;
            let tool_status = entry
                .get("status")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| anyhow!("tool `{tool_id}` in {relative_path} is missing status"))?;
            let stage_ids =
                entry.get("stage_ids").and_then(toml::Value::as_array).ok_or_else(|| {
                    anyhow!("tool `{tool_id}` in {relative_path} is missing stage_ids")
                })?;
            for stage_id in stage_ids.iter().filter_map(toml::Value::as_str) {
                bindings
                    .entry(BindingKey {
                        stage_id: stage_id.to_string(),
                        tool_id: tool_id.to_string(),
                    })
                    .or_default()
                    .insert(tool_status.to_string());
            }
        }
    }

    let mut rows = bindings
        .into_iter()
        .map(|(key, tool_statuses)| RetainedVcfBinding {
            stage_id: key.stage_id,
            tool_id: key.tool_id,
            tool_statuses,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    Ok(rows)
}

fn render_vcf_active_stage_tool_matrix_tsv(rows: &[VcfActiveStageToolMatrixRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\ttool_status\tstage_support_status\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tscope_state\tscope_detail\tscope_proof_path\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.tool_status),
            sanitize_tsv(&row.stage_support_status),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.scope_state),
            sanitize_tsv(&row.scope_detail),
            sanitize_tsv(&row.scope_proof_path),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn ensure_vcf_active_stage_tool_matrix_contract(
    rows: &[VcfActiveStageToolMatrixRow],
) -> Result<()> {
    let unique_rows = rows
        .iter()
        .map(|row| (row.stage_id.as_str(), row.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    if unique_rows.len() != rows.len() {
        return Err(anyhow!(
            "VCF active-stage-tool matrix must keep one row per retained VCF stage/tool binding"
        ));
    }

    if rows.iter().any(|row| {
        row.scope_state != ACTIVE_SCOPE_STATE
            && row.scope_state != COMPLETE_SCOPE_STATE
            && row.scope_state != REMOVED_FROM_SCOPE_STATE
    }) {
        return Err(anyhow!(
            "VCF active-stage-tool matrix encountered a row outside active/complete/removed scope resolution"
        ));
    }

    let active_row_count = count_scope_state(rows, ACTIVE_SCOPE_STATE);
    let complete_row_count = count_scope_state(rows, COMPLETE_SCOPE_STATE);
    let removed_row_count = count_scope_state(rows, REMOVED_FROM_SCOPE_STATE);
    if active_row_count + complete_row_count + removed_row_count != rows.len() {
        return Err(anyhow!(
            "VCF active-stage-tool matrix must classify every retained binding into exactly one scope state"
        ));
    }

    let expected_rows = [
        (
            "vcf.call",
            "bcftools",
            "production",
            "supported",
            ACTIVE_SCOPE_STATE,
            ACTIVE_SCOPE_DETAIL,
            ACTIVE_SCOPE_PROOF_PATH,
        ),
        (
            "vcf.call_gl",
            "angsd",
            "planned",
            "supported",
            REMOVED_FROM_SCOPE_STATE,
            BENCHMARK_NOT_READY_SCOPE_DETAIL,
            BENCHMARK_NOT_READY_SCOPE_PROOF_PATH,
        ),
        (
            "vcf.impute",
            "beagle-imputation",
            "experimental",
            "planned",
            REMOVED_FROM_SCOPE_STATE,
            LIFECYCLE_NOT_ACTIVE_SCOPE_DETAIL,
            LIFECYCLE_NOT_ACTIVE_SCOPE_PROOF_PATH,
        ),
        (
            "vcf.prepare_reference_panel",
            "bcftools",
            "production",
            "supported",
            ACTIVE_SCOPE_STATE,
            ACTIVE_SCOPE_DETAIL,
            ACTIVE_SCOPE_PROOF_PATH,
        ),
        (
            "vcf.phasing",
            "eagle",
            "experimental,planned",
            "planned",
            REMOVED_FROM_SCOPE_STATE,
            LIFECYCLE_NOT_ACTIVE_SCOPE_DETAIL,
            LIFECYCLE_NOT_ACTIVE_SCOPE_PROOF_PATH,
        ),
    ];

    for (
        stage_id,
        tool_id,
        tool_status,
        stage_support_status,
        scope_state,
        scope_detail,
        scope_proof_path,
    ) in expected_rows
    {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
            .ok_or_else(|| {
                anyhow!(
                    "VCF active-stage-tool matrix is missing retained binding `{stage_id}` / `{tool_id}`"
                )
            })?;
        if row.tool_status != tool_status
            || row.stage_support_status != stage_support_status
            || row.scope_state != scope_state
            || row.scope_detail != scope_detail
            || row.scope_proof_path != scope_proof_path
        {
            return Err(anyhow!(
                "VCF active-stage-tool matrix drifted for `{stage_id}` / `{tool_id}`"
            ));
        }
    }

    Ok(())
}

fn sorted_status_label(statuses: &BTreeSet<String>) -> String {
    statuses.iter().cloned().collect::<Vec<_>>().join(",")
}

fn count_scope_state(rows: &[VcfActiveStageToolMatrixRow], scope_state: &str) -> usize {
    rows.iter().filter(|row| row.scope_state == scope_state).count()
}

fn sanitize_tsv(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ")
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
