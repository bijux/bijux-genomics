use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::{
    bam_scientific_report_contract_for_stage, bam_stage_completeness, BamStage,
};
use bijux_dna_domain_fastq::{
    benchmark_readiness_for_stage_tool, stage_tool_bindings_for_stage,
    stage_tool_capability_contract, BenchmarkReadinessLevel, RuntimeNormalizationLevel,
};
use bijux_dna_planner_bam::stage_api::{
    allowed_tools_for_stage as allowed_bam_tools_for_stage, load_bam_domain_tool_contract_metadata,
    load_bam_domain_tool_execution_spec, load_bam_domain_tool_planning_spec,
};
use serde::Serialize;

use super::active_scope::include_fastq_active_benchmark_pair;
use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH: &str =
    "benchmarks/readiness/fastq-tool-serving-map.tsv";
pub(crate) const DEFAULT_BAM_TOOL_SERVING_MAP_PATH: &str =
    "benchmarks/readiness/bam-tool-serving-map.tsv";
const FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_tool_serving_map.v1";
const BAM_TOOL_SERVING_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_tool_serving_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ToolServingMapRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolServingMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<ToolServingMapRow>,
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

pub(crate) fn run_render_bam_tool_serving_map(
    args: &parse::BenchReadinessRenderBamToolServingMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_tool_serving_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH)),
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
) -> Result<ToolServingMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let corpus_status_by_stage = load_corpus_status_by_stage(repo_root)?;
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
            if !include_fastq_readiness_binding(&binding.stage_id, &binding.tool_id) {
                continue;
            }
            tool_ids.insert(binding.tool_id.as_str().to_string());
            rows.push(ToolServingMapRow {
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
    ensure_fastq_amplicon_fixture_coverage(&rows)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_tool_serving_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(ToolServingMapReport {
        schema_version: FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION,
        domain: "fastq",
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: inventory.stage_count,
        tool_count: tool_ids.len(),
        row_count: rows.len(),
        rows,
    })
}

fn include_fastq_readiness_binding(stage_id: &StageId, tool_id: &ToolId) -> bool {
    include_fastq_active_benchmark_pair(stage_id.as_str(), tool_id.as_str())
}

fn ensure_fastq_amplicon_fixture_coverage(rows: &[ToolServingMapRow]) -> Result<()> {
    let expected_rows = [
        (
            "cutadapt",
            "fastq.normalize_primers",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "vsearch",
            "fastq.remove_chimeras",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        ("dada2", "fastq.infer_asvs", "governed_execution", "runnable", "parse_normalized"),
        (
            "vsearch",
            "fastq.cluster_otus",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "seqkit",
            "fastq.normalize_abundance",
            "governed_benchmark_cohort",
            "runnable",
            "benchmark_normalized",
        ),
        (
            "seqfu",
            "fastq.normalize_abundance",
            "planned_contract",
            "declared_only",
            "not_normalized",
        ),
    ];

    for (tool_id, stage_id, support_status, adapter_status, parser_status) in expected_rows {
        let row =
            rows.iter().find(|row| row.tool_id == tool_id && row.stage_id == stage_id).ok_or_else(
                || anyhow!("FASTQ amplicon serving map is missing `{stage_id}` / `{tool_id}`"),
            )?;
        if row.support_status != support_status
            || row.adapter_status != adapter_status
            || row.parser_status != parser_status
            || row.corpus_status != "fixture:corpus-03-amplicon-mini"
        {
            return Err(anyhow!(
                "FASTQ amplicon serving row `{}` / `{}` must keep its governed corpus-03 readiness contract",
                stage_id,
                tool_id
            ));
        }
    }

    Ok(())
}

pub(crate) fn render_bam_tool_serving_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ToolServingMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let corpus_status_by_stage = load_corpus_status_by_stage(repo_root)?;
    let inventory = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?;
    let mut rows = Vec::new();
    let mut tool_ids = BTreeSet::new();
    for inventory_stage in &inventory.stages {
        let stage = BamStage::try_from(inventory_stage.stage_id.as_str())
            .with_context(|| format!("resolve BAM stage `{}`", inventory_stage.stage_id))?;
        let stage_id = StageId::new(inventory_stage.stage_id.clone());
        let corpus_status =
            corpus_status_by_stage.get(inventory_stage.stage_id.as_str()).ok_or_else(|| {
                anyhow!(
                    "BAM local corpus compatibility report is missing stage `{}`",
                    inventory_stage.stage_id
                )
            })?;
        let parser_status = bam_parser_status_label(stage);
        for tool_id in allowed_bam_tools_for_stage(stage) {
            tool_ids.insert(tool_id.as_str().to_string());
            rows.push(ToolServingMapRow {
                tool_id: tool_id.as_str().to_string(),
                stage_id: stage_id.as_str().to_string(),
                support_status: bam_support_status_label(repo_root, &stage_id, &tool_id),
                adapter_status: bam_adapter_status_label(repo_root, &stage_id, &tool_id),
                parser_status: parser_status.to_string(),
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
    fs::write(&output_path, render_tool_serving_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(ToolServingMapReport {
        schema_version: BAM_TOOL_SERVING_MAP_SCHEMA_VERSION,
        domain: "bam",
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

pub(crate) fn load_corpus_status_by_stage(repo_root: &Path) -> Result<BTreeMap<String, String>> {
    let matrix_path = repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH);
    let corpus_compatibility = validate_corpus_stage_compatibility_path(repo_root, &matrix_path)?;
    corpus_compatibility
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
        .collect::<Result<BTreeMap<_, _>>>()
}

fn bam_support_status_label(repo_root: &Path, stage_id: &StageId, tool_id: &ToolId) -> String {
    let Ok(metadata) = load_bam_domain_tool_contract_metadata(repo_root, tool_id) else {
        return "missing_contract".to_string();
    };
    let admitted = metadata.stage_ids.iter().any(|candidate| candidate == stage_id)
        || metadata.planned_stage_ids.iter().any(|candidate| candidate == stage_id);
    if !admitted {
        return "mismatched_contract".to_string();
    }
    metadata.pair_support_level(stage_id).as_str().to_string()
}

fn bam_adapter_status_label(repo_root: &Path, stage_id: &StageId, tool_id: &ToolId) -> String {
    if load_bam_domain_tool_execution_spec(repo_root, stage_id, tool_id).is_ok() {
        "runnable".to_string()
    } else if load_bam_domain_tool_planning_spec(repo_root, stage_id, tool_id).is_ok() {
        "plannable".to_string()
    } else {
        "declared_only".to_string()
    }
}

fn bam_parser_status_label(stage: BamStage) -> &'static str {
    let completeness = bam_stage_completeness(stage);
    if completeness.has_parser_fixtures {
        "parser_fixture_validated"
    } else if bam_scientific_report_contract_for_stage(stage.as_str()).is_some() {
        "scientific_report_contract"
    } else if completeness.has_artifact_contract {
        "artifact_contract_only"
    } else {
        "missing"
    }
}

fn render_tool_serving_map_tsv(rows: &[ToolServingMapRow]) -> String {
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
        render_bam_tool_serving_map, render_fastq_tool_serving_map,
        BAM_TOOL_SERVING_MAP_SCHEMA_VERSION, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
        DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH, FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION,
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
            row.tool_id == "bijux_dna"
                && row.stage_id == "fastq.detect_duplicates_premerge"
                && row.support_status == "governed_execution"
                && row.adapter_status == "runnable"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.detect_adapters"
                && row.support_status == "observer_specialized_benchmark"
                && row.adapter_status == "runnable"
                && row.parser_status == "comparable"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        for tool_id in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.validate_reads"
                    && row.support_status == "observer_specialized_benchmark"
                    && row.adapter_status == "runnable"
                    && row.parser_status == "comparable"
                    && row.corpus_status == "fixture:corpus-01-mini"
            }));
        }
        let screen_rows = report
            .rows
            .iter()
            .filter(|row| row.stage_id == "fastq.screen_taxonomy")
            .collect::<Vec<_>>();
        assert_eq!(screen_rows.len(), 4);
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(screen_rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.support_status == "governed_benchmark_cohort"
                    && row.adapter_status == "runnable"
                    && row.parser_status == "benchmark_normalized"
                    && row.corpus_status == "fixture:corpus-02-edna-mini"
            }));
        }
        for (tool_id, stage_id, support_status, adapter_status, parser_status) in [
            (
                "cutadapt",
                "fastq.normalize_primers",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
            ),
            (
                "vsearch",
                "fastq.remove_chimeras",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
            ),
            ("dada2", "fastq.infer_asvs", "governed_execution", "runnable", "parse_normalized"),
            (
                "vsearch",
                "fastq.cluster_otus",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
            ),
            (
                "seqkit",
                "fastq.normalize_abundance",
                "governed_benchmark_cohort",
                "runnable",
                "benchmark_normalized",
            ),
            (
                "seqfu",
                "fastq.normalize_abundance",
                "planned_contract",
                "declared_only",
                "not_normalized",
            ),
        ] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == stage_id
                    && row.support_status == support_status
                    && row.adapter_status == adapter_status
                    && row.parser_status == parser_status
                    && row.corpus_status == "fixture:corpus-03-amplicon-mini"
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "multiqc"
                && row.stage_id == "fastq.report_qc"
                && row.corpus_status == "planner_only"
        }));
    }

    #[test]
    fn bam_tool_serving_map_reports_governed_bam_stage_scope() {
        let root = repo_root();
        let report =
            render_bam_tool_serving_map(&root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))
                .expect("render BAM tool serving map");

        assert_eq!(report.schema_version, BAM_TOOL_SERVING_MAP_SCHEMA_VERSION);
        assert_eq!(report.domain, "bam");
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.tool_count, 25);
        assert_eq!(report.row_count, 49);
        assert!(!report.rows.is_empty(), "BAM tool serving map must contain rows");
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bwa"
                && row.stage_id == "bam.align"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2"
                && row.stage_id == "bam.align"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.validate"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.gc_bias"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.insert_size"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.mapping_summary"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.endogenous_content"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bamutil"
                && row.stage_id == "bam.overlap_correction"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.bias_mitigation"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "pydamage"
                && row.stage_id == "bam.damage"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "multiqc"
                && row.stage_id == "bam.qc_pre"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "preseq"
                && row.stage_id == "bam.complexity"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "yleaf"
                && row.stage_id == "bam.sex"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "yleaf"
                && row.stage_id == "bam.haplogroups"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
        }));
        for tool_id in ["contammix", "schmutzi", "verifybamid2"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "bam.contamination"
                    && row.support_status == "supported"
                    && row.adapter_status == "runnable"
                    && row.parser_status == "parser_fixture_validated"
                    && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "angsd"
                && row.stage_id == "bam.genotyping"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-genotyping-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "angsd"
                && row.stage_id == "bam.kinship"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-kinship-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "king"
                && row.stage_id == "bam.kinship"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-kinship-mini"
        }));
    }
}
