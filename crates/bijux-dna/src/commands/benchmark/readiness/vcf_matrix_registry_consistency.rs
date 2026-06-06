use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_vcf_stage_catalog::build_vcf_stage_catalog_rows;
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH: &str =
    "target/bench-readiness/vcf-matrix-registry-consistency.json";
const VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_matrix_registry_consistency.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfMatrixRegistryConsistencyRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) drift_kind: String,
    pub(crate) stage_support_status: String,
    pub(crate) tool_registry_status: String,
    pub(crate) registry_stage_ids: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfMatrixRegistryConsistencyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) passes_gate: bool,
    pub(crate) stage_count: usize,
    pub(crate) matrix_row_count: usize,
    pub(crate) registry_pair_count: usize,
    pub(crate) benchmark_ready_registry_pair_count: usize,
    pub(crate) unregistered_matrix_pair_count: usize,
    pub(crate) missing_benchmark_ready_registry_pair_count: usize,
    pub(crate) rows: Vec<VcfMatrixRegistryConsistencyRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VcfRegistryToolRecord {
    tool_id: String,
    stage_ids: BTreeSet<String>,
    statuses: BTreeSet<String>,
}

pub(crate) fn run_render_vcf_matrix_registry_consistency(
    args: &parse::BenchReadinessRenderVcfMatrixRegistryConsistencyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_matrix_registry_consistency(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_matrix_registry_consistency(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfMatrixRegistryConsistencyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stage_support_by_id = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id, row.support_status))
        .collect::<BTreeMap<_, _>>();
    let matrix_rows = build_vcf_stage_matrix_rows()?;
    let registry_records = load_vcf_registry_tool_records(repo_root)?;

    let registry_pair_count = registry_records.iter().map(|record| record.stage_ids.len()).sum();
    let registry_by_tool = registry_records
        .iter()
        .cloned()
        .map(|record| (record.tool_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let registry_pairs = registry_records
        .iter()
        .flat_map(|record| {
            record
                .stage_ids
                .iter()
                .cloned()
                .map(|stage_id| (stage_id, record.tool_id.clone()))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();

    let matrix_pairs = matrix_rows
        .iter()
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();

    let benchmark_ready_registry_pairs = registry_records
        .iter()
        .flat_map(|record| {
            record
                .stage_ids
                .iter()
                .filter(|stage_id| {
                    record.statuses.contains("production")
                        && stage_support_by_id
                            .get(stage_id.as_str())
                            .is_some_and(|status| status == "supported")
                })
                .cloned()
                .map(|stage_id| (stage_id, record.tool_id.clone()))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();

    let mut rows = Vec::new();
    for (stage_id, tool_id) in matrix_pairs.difference(&registry_pairs) {
        let registry_record = registry_by_tool.get(tool_id.as_str());
        let stage_support_status = stage_support_by_id
            .get(stage_id.as_str())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let tool_registry_status = registry_record
            .map(|record| sorted_status_label(&record.statuses))
            .unwrap_or_else(|| "tool_missing".to_string());
        let registry_stage_ids = registry_record
            .map(|record| record.stage_ids.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        rows.push(VcfMatrixRegistryConsistencyRow {
            stage_id: stage_id.clone(),
            tool_id: tool_id.clone(),
            drift_kind: "matrix_row_unregistered".to_string(),
            stage_support_status,
            tool_registry_status: tool_registry_status.clone(),
            registry_stage_ids: registry_stage_ids.clone(),
            reason: format!(
                "VCF matrix row `{stage_id}` / `{tool_id}` is benchmarked but the VCF registry does not admit that pair; tool registry status: {tool_registry_status}; registered stages: {}",
                if registry_stage_ids.is_empty() {
                    "<none>".to_string()
                } else {
                    registry_stage_ids.join(", ")
                }
            ),
        });
    }
    for (stage_id, tool_id) in benchmark_ready_registry_pairs.difference(&matrix_pairs) {
        let registry_record = registry_by_tool
            .get(tool_id.as_str())
            .ok_or_else(|| anyhow!("missing registry record for `{tool_id}`"))?;
        let registry_stage_ids = registry_record.stage_ids.iter().cloned().collect::<Vec<_>>();
        rows.push(VcfMatrixRegistryConsistencyRow {
            stage_id: stage_id.clone(),
            tool_id: tool_id.clone(),
            drift_kind: "benchmark_ready_registry_pair_missing_from_matrix".to_string(),
            stage_support_status: stage_support_by_id
                .get(stage_id.as_str())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            tool_registry_status: sorted_status_label(&registry_record.statuses),
            registry_stage_ids: registry_stage_ids.clone(),
            reason: format!(
                "VCF registry marks `{stage_id}` / `{tool_id}` as benchmark-ready for the governed production slice, but the VCF matrix does not benchmark it; registered stages: {}",
                registry_stage_ids.join(", ")
            ),
        });
    }
    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.drift_kind.cmp(&right.drift_kind))
    });
    ensure_vcf_matrix_registry_consistency_contract(&rows)?;

    let report = VcfMatrixRegistryConsistencyReport {
        schema_version: VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        passes_gate: rows.is_empty(),
        stage_count: stage_support_by_id.len(),
        matrix_row_count: matrix_rows.len(),
        registry_pair_count,
        benchmark_ready_registry_pair_count: benchmark_ready_registry_pairs.len(),
        unregistered_matrix_pair_count: rows
            .iter()
            .filter(|row| row.drift_kind == "matrix_row_unregistered")
            .count(),
        missing_benchmark_ready_registry_pair_count: rows
            .iter()
            .filter(|row| row.drift_kind == "benchmark_ready_registry_pair_missing_from_matrix")
            .count(),
        rows,
    };
    let payload = serde_json::to_string_pretty(&report)
        .context("render VCF matrix/registry consistency gate to JSON")?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;
    Ok(report)
}

fn load_vcf_registry_tool_records(repo_root: &Path) -> Result<Vec<VcfRegistryToolRecord>> {
    let mut records = BTreeMap::<String, VcfRegistryToolRecord>::new();
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
                .ok_or_else(|| anyhow!("tool entry in {relative_path} is missing id"))?
                .to_string();
            let stage_ids = entry
                .get("stage_ids")
                .and_then(toml::Value::as_array)
                .ok_or_else(|| anyhow!("tool `{tool_id}` in {relative_path} is missing stage_ids"))?
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            let status = entry
                .get("status")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| anyhow!("tool `{tool_id}` in {relative_path} is missing status"))?;

            let record = records.entry(tool_id.clone()).or_insert_with(|| VcfRegistryToolRecord {
                tool_id,
                stage_ids: BTreeSet::new(),
                statuses: BTreeSet::new(),
            });
            record.stage_ids.extend(stage_ids);
            record.statuses.insert(status.to_string());
        }
    }
    Ok(records.into_values().collect())
}

fn sorted_status_label(statuses: &BTreeSet<String>) -> String {
    statuses.iter().cloned().collect::<Vec<_>>().join(",")
}

fn ensure_vcf_matrix_registry_consistency_contract(
    rows: &[VcfMatrixRegistryConsistencyRow],
) -> Result<()> {
    if !rows.is_empty() {
        return Err(anyhow!(
            "VCF matrix/registry consistency drift remains; expected an empty governed failure slice, found {} rows",
            rows.len()
        ));
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_matrix_registry_consistency, DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH,
        VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_matrix_registry_consistency_gate_reports_governed_pass_state() {
        let root = repo_root();
        let report = render_vcf_matrix_registry_consistency(
            &root,
            PathBuf::from(DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH),
        )
        .expect("render VCF matrix/registry consistency");

        assert_eq!(report.schema_version, VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION);
        assert_eq!(report.domain, "vcf");
        assert!(report.passes_gate);
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.matrix_row_count, 20);
        assert_eq!(report.registry_pair_count, 46);
        assert_eq!(report.benchmark_ready_registry_pair_count, 8);
        assert_eq!(report.unregistered_matrix_pair_count, 0);
        assert_eq!(report.missing_benchmark_ready_registry_pair_count, 0);
        assert!(report.rows.is_empty());
    }
}
