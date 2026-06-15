use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::vcf_tool_serving_map::{render_vcf_tool_serving_map, DEFAULT_VCF_TOOL_SERVING_MAP_PATH};
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_ORPHAN_TOOLS_PATH: &str = "benchmarks/readiness/vcf-orphan-tools.tsv";
const VCF_ORPHAN_TOOLS_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_orphan_tools.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfOrphanToolRow {
    pub(crate) tool_id: String,
    pub(crate) registered_binary: String,
    pub(crate) served_stage_count: usize,
    pub(crate) decision: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfOrphanToolsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) orphan_count: usize,
    pub(crate) required_tool_count: usize,
    pub(crate) registered_tool_count: usize,
    pub(crate) served_tool_count: usize,
    pub(crate) rows: Vec<VcfOrphanToolRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VcfRegistryToolRecord {
    tool_id: String,
    registered_binary: String,
    stage_ids: BTreeSet<String>,
}

pub(crate) fn run_render_vcf_orphan_tools(
    args: &parse::BenchReadinessRenderVcfOrphanToolsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_orphan_tools(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ORPHAN_TOOLS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_orphan_tools(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfOrphanToolsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_orphan_tool_rows(repo_root)?;
    let required_tool_count = load_required_vcf_tool_ids(repo_root)?.len();
    let registered_tool_count = load_vcf_registry_tool_records(repo_root)?.len();
    let served_tool_count =
        render_vcf_tool_serving_map(repo_root, PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH))?
            .rows
            .iter()
            .map(|row| row.tool_id.as_str())
            .collect::<BTreeSet<_>>()
            .len();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_orphan_tools_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(VcfOrphanToolsReport {
        schema_version: VCF_ORPHAN_TOOLS_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        orphan_count: rows.len(),
        required_tool_count,
        registered_tool_count,
        served_tool_count,
        rows,
    })
}

fn collect_vcf_orphan_tool_rows(repo_root: &Path) -> Result<Vec<VcfOrphanToolRow>> {
    let required_tool_ids = load_required_vcf_tool_ids(repo_root)?;
    let registry_records = load_vcf_registry_tool_records(repo_root)?;
    let matrix_rows = build_vcf_stage_matrix_rows()?;
    let benchmark_stage_ids =
        matrix_rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>();
    let serving_map =
        render_vcf_tool_serving_map(repo_root, PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH))?;
    let served_stage_count_by_tool =
        serving_map.rows.into_iter().fold(BTreeMap::<String, usize>::new(), |mut acc, row| {
            *acc.entry(row.tool_id).or_default() += 1;
            acc
        });

    let mut rows = Vec::new();
    for record in registry_records {
        let served_stage_count =
            served_stage_count_by_tool.get(record.tool_id.as_str()).copied().unwrap_or(0);
        if served_stage_count > 0 {
            continue;
        }

        let has_benchmark_stage_overlap =
            record.stage_ids.iter().any(|stage_id| benchmark_stage_ids.contains(stage_id));
        let decision =
            if required_tool_ids.contains(record.tool_id.as_str()) && has_benchmark_stage_overlap {
                "future_not_benchmark_ready"
            } else {
                "remove_from_scope"
            };

        rows.push(VcfOrphanToolRow {
            tool_id: record.tool_id,
            registered_binary: record.registered_binary,
            served_stage_count,
            decision: decision.to_string(),
        });
    }

    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    ensure_vcf_orphan_tool_contract(&rows, &required_tool_ids)?;
    Ok(rows)
}

fn load_required_vcf_tool_ids(repo_root: &Path) -> Result<BTreeSet<String>> {
    let mut tool_ids = BTreeSet::<String>::new();
    for relative_path in [
        "configs/ci/tools/required_tools_vcf.toml",
        "configs/ci/tools/required_tools_vcf_downstream.toml",
    ] {
        let raw = fs::read_to_string(repo_root.join(relative_path))
            .with_context(|| format!("read {}", repo_root.join(relative_path).display()))?;
        let parsed: toml::Value = toml::from_str(&raw)
            .with_context(|| format!("parse {}", repo_root.join(relative_path).display()))?;
        let entries = parsed
            .get("required_tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing required_tools in {relative_path}"))?;
        for entry in entries {
            if let Some(tool_id) = entry.as_str() {
                tool_ids.insert(tool_id.to_string());
            }
        }
    }
    Ok(tool_ids)
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
            let expected_bin = entry
                .get("expected_bin")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| {
                    anyhow!("tool `{tool_id}` in {relative_path} is missing expected_bin")
                })?
                .to_string();
            let stage_ids = entry
                .get("stage_ids")
                .and_then(toml::Value::as_array)
                .ok_or_else(|| anyhow!("tool `{tool_id}` in {relative_path} is missing stage_ids"))?
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>();

            let record = records.entry(tool_id.clone()).or_insert_with(|| VcfRegistryToolRecord {
                tool_id: tool_id.clone(),
                registered_binary: expected_bin.clone(),
                stage_ids: BTreeSet::new(),
            });
            if record.registered_binary != expected_bin {
                bail!(
                    "VCF registry tool `{}` drifted across registry files (`{}` vs `{}`)",
                    tool_id,
                    record.registered_binary,
                    expected_bin
                );
            }
            record.stage_ids.extend(stage_ids);
        }
    }

    Ok(records.into_values().collect())
}

fn ensure_vcf_orphan_tool_contract(
    rows: &[VcfOrphanToolRow],
    required_tool_ids: &BTreeSet<String>,
) -> Result<()> {
    let expected_rows = [
        ("angsd", "angsd", 0usize, "future_not_benchmark_ready"),
        ("eagle", "eagle", 0usize, "future_not_benchmark_ready"),
        ("glimpse", "glimpse", 0usize, "future_not_benchmark_ready"),
        ("ibdhap", "ibdhap", 0usize, "future_not_benchmark_ready"),
        ("ibdseq", "ibdseq", 0usize, "future_not_benchmark_ready"),
        ("impute5", "impute5", 0usize, "future_not_benchmark_ready"),
        ("minimac4", "minimac4", 0usize, "future_not_benchmark_ready"),
        ("shapeit", "shapeit", 0usize, "future_not_benchmark_ready"),
    ];

    if rows.len() != expected_rows.len() {
        return Err(anyhow!(
            "VCF orphan tool report drifted from the governed orphan slice (expected {}, found {})",
            expected_rows.len(),
            rows.len()
        ));
    }

    for (tool_id, registered_binary, served_stage_count, decision) in expected_rows {
        let row = rows
            .iter()
            .find(|row| row.tool_id == tool_id)
            .ok_or_else(|| anyhow!("VCF orphan tool report is missing `{tool_id}`"))?;
        if row.registered_binary != registered_binary
            || row.served_stage_count != served_stage_count
            || row.decision != decision
        {
            return Err(anyhow!(
                "VCF orphan tool `{}` drifted from its governed orphan decision contract",
                tool_id
            ));
        }
        if !required_tool_ids.contains(tool_id) {
            return Err(anyhow!(
                "VCF orphan tool `{}` must remain visible only while it is still required by the governed VCF tool scope",
                tool_id
            ));
        }
    }

    for row in rows {
        if row.served_stage_count != 0 {
            return Err(anyhow!(
                "VCF orphan tool `{}` must have served_stage_count 0",
                row.tool_id
            ));
        }
        if row.decision != "future_not_benchmark_ready" && row.decision != "remove_from_scope" {
            return Err(anyhow!(
                "VCF orphan tool `{}` declared unsupported decision `{}`",
                row.tool_id,
                row.decision
            ));
        }
    }

    Ok(())
}

fn render_vcf_orphan_tools_tsv(rows: &[VcfOrphanToolRow]) -> String {
    let mut rendered = String::from("tool_id\tregistered_binary\tserved_stage_count\tdecision\n");
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.registered_binary),
            row.served_stage_count,
            sanitize_tsv(&row.decision),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
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
        render_vcf_orphan_tools, DEFAULT_VCF_ORPHAN_TOOLS_PATH, VCF_ORPHAN_TOOLS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_orphan_tools_report_tracks_governed_decisions() {
        let root = repo_root();
        let report = render_vcf_orphan_tools(&root, PathBuf::from(DEFAULT_VCF_ORPHAN_TOOLS_PATH))
            .expect("render VCF orphan tools");

        assert_eq!(report.schema_version, VCF_ORPHAN_TOOLS_SCHEMA_VERSION);
        assert_eq!(report.domain, "vcf");
        assert_eq!(report.orphan_count, 9);
        assert_eq!(report.required_tool_count, 17);
        assert_eq!(report.registered_tool_count, 17);
        assert_eq!(report.served_tool_count, 8);
        assert!(report.rows.iter().all(|row| row.served_stage_count == 0));
        assert!(report.rows.iter().all(|row| row.decision == "future_not_benchmark_ready"));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "angsd"
                && row.registered_binary == "angsd"
                && row.decision == "future_not_benchmark_ready"
        }));
        assert!(report.rows.iter().all(|row| row.tool_id != "plink"));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "shapeit"
                && row.registered_binary == "shapeit"
                && row.decision == "future_not_benchmark_ready"
        }));
    }
}
