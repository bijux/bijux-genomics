use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::{
    benchmark_readiness_for_stage_tool, stage_tool_bindings_for_stage,
    stage_tool_capability_contract, BenchmarkReadinessLevel, RuntimeNormalizationLevel,
};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH: &str =
    "target/bench-readiness/fastq-tool-serving-map.tsv";
const FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_tool_serving_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqToolServingMapRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqToolServingMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<FastqToolServingMapRow>,
}

pub(crate) fn run_render_fastq_tool_serving_map(
    args: &parse::BenchReadinessRenderFastqToolServingMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_tool_serving_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_tool_serving_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqToolServingMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let matrix_path = repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH);
    let corpus_compatibility = validate_corpus_stage_compatibility_path(repo_root, &matrix_path)?;
    let corpus_status_by_stage = corpus_compatibility
        .stages
        .iter()
        .map(|entry| {
            let corpus_status = match entry.compatibility_kind.as_str() {
                "fixture" => {
                    let fixture_id = entry.fixture_id.as_deref().ok_or_else(|| {
                        anyhow!(
                            "stage `{}` is marked fixture-backed without a fixture_id",
                            entry.stage_id
                        )
                    })?;
                    Ok(format!("fixture:{fixture_id}"))
                }
                "planner_only" => Ok("planner_only".to_string()),
                other => Err(anyhow!(
                    "stage `{}` declares unsupported corpus compatibility kind `{other}`",
                    entry.stage_id
                )),
            }?;
            Ok((entry.stage_id.clone(), corpus_status))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    let inventory = load_local_stage_inventory(repo_root, BenchLocalDomain::Fastq)?;
    let mut rows = Vec::new();
    let mut tool_ids = BTreeSet::new();
    for inventory_stage in &inventory.stages {
        let stage_id = StageId::new(inventory_stage.stage_id.clone());
        let corpus_status =
            corpus_status_by_stage.get(inventory_stage.stage_id.as_str()).ok_or_else(|| {
                anyhow!(
                    "FASTQ local corpus compatibility report is missing stage `{}`",
                    inventory_stage.stage_id
                )
            })?;
        for binding in stage_tool_bindings_for_stage(&stage_id) {
            let runtime_normalization =
                runtime_normalization_for_stage_tool(&binding.stage_id, &binding.tool_id);
            let capability = stage_tool_capability_contract(
                &binding.stage_id,
                &binding.tool_id,
                runtime_normalization,
            )
            .ok_or_else(|| {
                anyhow!(
                    "missing FASTQ capability contract for `{}` / `{}`",
                    binding.stage_id.as_str(),
                    binding.tool_id.as_str()
                )
            })?;
            let readiness = benchmark_readiness_for_stage_tool(
                &binding.stage_id,
                &binding.tool_id,
                runtime_normalization,
            )
            .ok_or_else(|| {
                anyhow!(
                    "missing FASTQ benchmark readiness for `{}` / `{}`",
                    binding.stage_id.as_str(),
                    binding.tool_id.as_str()
                )
            })?;
            tool_ids.insert(binding.tool_id.as_str().to_string());
            rows.push(FastqToolServingMapRow {
                tool_id: binding.tool_id.as_str().to_string(),
                stage_id: binding.stage_id.as_str().to_string(),
                support_status: benchmark_readiness_label(readiness).to_string(),
                adapter_status: adapter_status_label(&capability).to_string(),
                parser_status: parser_status_label(&capability).to_string(),
                corpus_status: corpus_status.clone(),
            });
        }
    }
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_tool_serving_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(FastqToolServingMapReport {
        schema_version: FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION,
        domain: "fastq",
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: inventory.stage_count,
        tool_count: tool_ids.len(),
        row_count: rows.len(),
        rows,
    })
}

fn runtime_normalization_for_stage_tool(
    stage_id: &StageId,
    tool_id: &bijux_dna_core::ids::ToolId,
) -> RuntimeNormalizationLevel {
    match bijux_dna_stages_fastq::runtime_interpretation_for_stage_tool(stage_id, tool_id)
        .unwrap_or(bijux_dna_stages_fastq::RuntimeInterpretationLevel::GenericEnvelope)
    {
        bijux_dna_stages_fastq::RuntimeInterpretationLevel::GenericEnvelope => {
            RuntimeNormalizationLevel::GenericEnvelope
        }
        bijux_dna_stages_fastq::RuntimeInterpretationLevel::ObserverSpecialized => {
            RuntimeNormalizationLevel::ObserverSpecialized
        }
    }
}

fn benchmark_readiness_label(readiness: BenchmarkReadinessLevel) -> &'static str {
    match readiness {
        BenchmarkReadinessLevel::PlannedContract => "planned_contract",
        BenchmarkReadinessLevel::GovernedExecution => "governed_execution",
        BenchmarkReadinessLevel::GovernedBenchmarkCohort => "governed_benchmark_cohort",
        BenchmarkReadinessLevel::ObserverSpecializedBenchmark => "observer_specialized_benchmark",
    }
}

fn adapter_status_label(
    capability: &bijux_dna_domain_fastq::StageToolCapabilityContract,
) -> &'static str {
    if capability.runnable {
        "runnable"
    } else if capability.plannable {
        "plannable"
    } else if capability.declared {
        "declared_only"
    } else {
        "missing"
    }
}

fn parser_status_label(
    capability: &bijux_dna_domain_fastq::StageToolCapabilityContract,
) -> &'static str {
    if capability.comparable {
        "comparable"
    } else if capability.benchmark_normalized {
        "benchmark_normalized"
    } else if capability.parse_normalized {
        "parse_normalized"
    } else {
        "not_normalized"
    }
}

fn render_fastq_tool_serving_map_tsv(rows: &[FastqToolServingMapRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
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
        render_fastq_tool_serving_map, DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
        FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn fastq_tool_serving_map_reports_governed_fastq_stage_scope() {
        let root = repo_root();
        let report = render_fastq_tool_serving_map(
            &root,
            PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
        )
        .expect("render FASTQ tool serving map");

        assert_eq!(report.schema_version, FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION);
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.stage_count, 27);
        assert!(!report.rows.is_empty(), "FASTQ tool serving map must contain rows");
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.validate_reads"
                && row.support_status == "observer_specialized_benchmark"
                && row.adapter_status == "runnable"
                && row.parser_status == "comparable"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "diamond"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.support_status == "planned_contract"
                && row.adapter_status == "declared_only"
                && row.parser_status == "not_normalized"
                && row.corpus_status == "fixture:corpus-02-edna-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "multiqc"
                && row.stage_id == "fastq.report_qc"
                && row.corpus_status == "planner_only"
        }));
    }
}
