use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::catalog::{
    load_benchmark_stage_ids, load_registry_tool_matrix, load_tool_contracts, ReadinessDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_REGISTRY_EXTRA_PAIRS_PATH: &str =
    "benchmarks/readiness/stage-registry-extra-pairs.tsv";
const STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.stage_registry_extra_pairs.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StageRegistryExtraPairRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) contract_status: String,
    pub(crate) registry_sources: Vec<String>,
    pub(crate) registered_stage_ids: Vec<String>,
    pub(crate) intentional_override_status: String,
    pub(crate) intentional_override_reason: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageRegistryExtraPairsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) extra_pair_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<StageRegistryExtraPairRow>,
}

pub(crate) fn run_render_stage_registry_extra_pairs(
    args: &parse::BenchReadinessRenderStageRegistryExtraPairsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_registry_extra_pairs(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_REGISTRY_EXTRA_PAIRS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_stage_registry_extra_pairs(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageRegistryExtraPairsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let registry = load_registry_tool_matrix(repo_root)?;

    let mut benchmark_stage_ids = BTreeMap::<ReadinessDomain, BTreeSet<String>>::new();
    let mut admitted_pairs = BTreeMap::<ReadinessDomain, BTreeSet<(String, String)>>::new();
    let mut contract_stage_ids_by_tool =
        BTreeMap::<ReadinessDomain, BTreeMap<String, Vec<String>>>::new();

    for domain in [ReadinessDomain::Fastq, ReadinessDomain::Bam] {
        let stage_ids = load_benchmark_stage_ids(repo_root, domain)?;
        let mut pairs = BTreeSet::<(String, String)>::new();
        let mut stage_ids_by_tool = BTreeMap::<String, Vec<String>>::new();
        for contract in load_tool_contracts(repo_root, domain)? {
            let admitted_stage_ids = contract.benchmark_stage_overlap(&stage_ids);
            if admitted_stage_ids.is_empty() {
                continue;
            }
            for stage_id in &admitted_stage_ids {
                pairs.insert((stage_id.clone(), contract.tool_id.clone()));
            }
            stage_ids_by_tool.insert(contract.tool_id.clone(), admitted_stage_ids);
        }
        benchmark_stage_ids.insert(domain, stage_ids);
        admitted_pairs.insert(domain, pairs);
        contract_stage_ids_by_tool.insert(domain, stage_ids_by_tool);
    }

    let mut rows = Vec::new();
    for domain in [ReadinessDomain::Fastq, ReadinessDomain::Bam] {
        let stage_ids = benchmark_stage_ids.get(&domain).expect("benchmark stage ids");
        let domain_pairs = admitted_pairs.get(&domain).expect("admitted pairs");
        let stage_ids_by_tool =
            contract_stage_ids_by_tool.get(&domain).expect("contract stage ids by tool");

        for (stage_id, tool_id) in &registry.tool_stage_pairs {
            if !stage_ids.contains(stage_id) {
                continue;
            }
            if domain_pairs.contains(&(stage_id.clone(), tool_id.clone())) {
                continue;
            }
            let contract_status = if stage_ids_by_tool.contains_key(tool_id) {
                "pair_missing_from_contract"
            } else {
                "tool_missing_contract"
            };
            let intentional_override_reason = String::new();
            let intentional_override_status =
                if intentional_override_reason.is_empty() { "none" } else { "explicit_intent" };
            let registered_stage_ids = stage_ids_by_tool.get(tool_id).cloned().unwrap_or_default();
            let registry_sources = registry
                .pair_sources
                .get(&(stage_id.clone(), tool_id.clone()))
                .cloned()
                .unwrap_or_default();
            let stage_rationale =
                registry.stage_default_rationales.get(stage_id).cloned().unwrap_or_default();
            rows.push(StageRegistryExtraPairRow {
                domain: domain.as_str().to_string(),
                stage_id: stage_id.clone(),
                tool_id: tool_id.clone(),
                contract_status: contract_status.to_string(),
                registry_sources: registry_sources.clone(),
                registered_stage_ids: registered_stage_ids.clone(),
                intentional_override_status: intentional_override_status.to_string(),
                intentional_override_reason: intentional_override_reason.clone(),
                reason: format!(
                    "stage registry admits `{}` / `{}` inside the benchmark scope but domain contracts do not; contract status: {}; registry sources: {}; admitted contract stages for `{}`: {}; stage rationale: {}",
                    stage_id,
                    tool_id,
                    contract_status,
                    if registry_sources.is_empty() {
                        "<none>".to_string()
                    } else {
                        registry_sources.join(", ")
                    },
                    tool_id,
                    if registered_stage_ids.is_empty() {
                        "<none>".to_string()
                    } else {
                        registered_stage_ids.join(", ")
                    },
                    if stage_rationale.is_empty() {
                        "<none>".to_string()
                    } else {
                        stage_rationale
                    }
                ),
            });
        }
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_stage_registry_extra_pairs_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }
    let ok = rows.iter().all(|row| row.intentional_override_status == "explicit_intent");

    Ok(StageRegistryExtraPairsReport {
        schema_version: STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        extra_pair_count: rows.len(),
        domain_counts,
        ok,
        rows,
    })
}

fn render_stage_registry_extra_pairs_tsv(rows: &[StageRegistryExtraPairRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcontract_status\tregistry_sources\tregistered_stage_ids\tintentional_override_status\tintentional_override_reason\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.contract_status),
            sanitize_tsv(&row.registry_sources.join(",")),
            sanitize_tsv(&row.registered_stage_ids.join(",")),
            sanitize_tsv(&row.intentional_override_status),
            sanitize_tsv(&row.intentional_override_reason),
            sanitize_tsv(&row.reason),
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
        render_stage_registry_extra_pairs, DEFAULT_STAGE_REGISTRY_EXTRA_PAIRS_PATH,
        STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn stage_registry_extra_pairs_report_retains_registry_domain_drift() {
        let root = repo_root();
        let report = render_stage_registry_extra_pairs(
            &root,
            PathBuf::from(DEFAULT_STAGE_REGISTRY_EXTRA_PAIRS_PATH),
        )
        .expect("render stage registry extra pairs");

        assert_eq!(report.schema_version, STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_VERSION);
        assert_eq!(report.extra_pair_count, 1);
        assert!(!report.ok, "report must fail while registry-only benchmark pairs remain");
        assert_eq!(report.domain_counts.get("bam"), Some(&1));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.haplogroups"
                && row.tool_id == "samtools"
                && row.contract_status == "pair_missing_from_contract"
                && row.intentional_override_status == "none"
        }));
        assert!(
            !report.rows.iter().any(|row| {
                row.domain == "bam" && row.stage_id == "bam.qc_pre" && row.tool_id == "multiqc"
            }),
            "bam.qc_pre / multiqc must no longer remain a registry-only pair once the BAM tool contract exists"
        );
    }
}
