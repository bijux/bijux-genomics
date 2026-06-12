use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::bam_corpus_assignment::collect_bam_corpus_assignment_rows;
use super::bam_report_map::{collect_bam_report_map_rows, DEFAULT_BAM_REPORT_MAP_PATH};
use super::benchmark_command_rows::collect_benchmark_command_rows;
use super::corpus_asset_coverage_gate::{
    render_corpus_asset_coverage_gate, CorpusAssetCoverageGateStatus,
    DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH,
};
use super::expected_benchmark_results::{
    collect_expected_benchmark_result_rows, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::fastq_corpus_assignment::{
    collect_fastq_corpus_assignment_rows, FastqCorpusAssignmentStatus,
};
use super::fastq_report_map::{collect_fastq_report_map_rows, DEFAULT_FASTQ_REPORT_MAP_PATH};
use super::missing_benchmark_pairs::{
    render_missing_benchmark_pairs, DEFAULT_MISSING_BENCHMARK_PAIRS_PATH,
};
use super::pair_readiness::{
    collect_pair_readiness_rows, PairAssetStatus, PairReadinessGap, PairReadinessRow,
    DEFAULT_PAIR_READINESS_PATH,
};
use super::parser_completeness_gate::{
    render_parser_completeness_gate, ParserCompletenessGateStatus,
    DEFAULT_PARSER_COMPLETENESS_GATE_PATH,
};
use super::stage_registry_extra_pairs::{
    render_stage_registry_extra_pairs, DEFAULT_STAGE_REGISTRY_EXTRA_PAIRS_PATH,
};
use super::undercovered_stages::{render_undercovered_stages, DEFAULT_UNDERCOVERED_STAGES_PATH};
use super::unregistered_benchmark_pairs::{
    render_unregistered_benchmark_pairs, DEFAULT_UNREGISTERED_BENCHMARK_PAIRS_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_TOOL_BENCHMARK_READY_PATH: &str =
    "benchmarks/readiness/FASTQ_BAM_STAGE_TOOL_BENCHMARK_READY.json";
const STAGE_TOOL_BENCHMARK_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_bam_stage_tool_benchmark_ready.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StageToolBenchmarkReadySurfaceStatus {
    ReadySliceComplete,
    FailingPairsPresent,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BenchmarkBindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StageToolBenchmarkReadySurfaceSummary {
    pub(crate) surface_id: String,
    pub(crate) surface_status: StageToolBenchmarkReadySurfaceStatus,
    pub(crate) measured_scope: String,
    pub(crate) expected_count: usize,
    pub(crate) covered_count: usize,
    pub(crate) excluded_count: usize,
    pub(crate) failing_count: usize,
    pub(crate) evidence_paths: Vec<String>,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StageToolBenchmarkReadyFailingPair {
    pub(crate) row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) failure_surfaces: Vec<String>,
    pub(crate) benchmark_status: String,
    pub(crate) readiness_gap: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) asset_status: String,
    pub(crate) registry_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StageToolBenchmarkReadyExcludedPair {
    pub(crate) row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_gap: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) asset_status: String,
    pub(crate) registry_status: String,
    pub(crate) excluded_from_generated_jobs: bool,
    pub(crate) excluded_from_expected_results: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageToolBenchmarkReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) passes_gate: bool,
    pub(crate) expected_pair_count: usize,
    pub(crate) benchmark_ready_pair_count: usize,
    pub(crate) excluded_pair_count: usize,
    pub(crate) failing_pair_count: usize,
    pub(crate) generated_job_pair_count: usize,
    pub(crate) expected_result_pair_count: usize,
    pub(crate) benchmark_ready_stage_count: usize,
    pub(crate) excluded_registry_gap_count: usize,
    pub(crate) surface_summaries: Vec<StageToolBenchmarkReadySurfaceSummary>,
    pub(crate) failing_pairs: Vec<StageToolBenchmarkReadyFailingPair>,
    pub(crate) excluded_pairs: Vec<StageToolBenchmarkReadyExcludedPair>,
}

pub(crate) fn run_render_stage_tool_benchmark_ready(
    args: &parse::BenchReadinessRenderStageToolBenchmarkReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_tool_benchmark_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_TOOL_BENCHMARK_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_stage_tool_benchmark_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageToolBenchmarkReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);

    let pair_rows = collect_pair_readiness_rows(repo_root)?;
    let pair_index = pair_rows
        .iter()
        .cloned()
        .map(|row| (binding_key(&row.domain, &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    let benchmark_ready_keys = pair_rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    let excluded_keys = pair_rows
        .iter()
        .filter(|row| row.benchmark_status != "benchmark_ready")
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();

    let missing_pairs = render_missing_benchmark_pairs(
        repo_root,
        repo_root.join(DEFAULT_MISSING_BENCHMARK_PAIRS_PATH),
    )?;
    let undercovered_stages =
        render_undercovered_stages(repo_root, repo_root.join(DEFAULT_UNDERCOVERED_STAGES_PATH))?;
    let unregistered_pairs = render_unregistered_benchmark_pairs(
        repo_root,
        repo_root.join(DEFAULT_UNREGISTERED_BENCHMARK_PAIRS_PATH),
    )?;
    let registry_extra_pairs = render_stage_registry_extra_pairs(
        repo_root,
        repo_root.join(DEFAULT_STAGE_REGISTRY_EXTRA_PAIRS_PATH),
    )?;
    let parser_gate = render_parser_completeness_gate(
        repo_root,
        repo_root.join(DEFAULT_PARSER_COMPLETENESS_GATE_PATH),
    )?;
    let corpus_asset_gate = render_corpus_asset_coverage_gate(
        repo_root,
        repo_root.join(DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH),
    )?;

    let generated_job_keys = collect_benchmark_command_rows(repo_root)?
        .into_iter()
        .filter_map(|row| pair_index_by_stage_tool(&pair_index, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    let expected_result_keys = collect_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();

    let ready_corpus_keys = collect_fastq_corpus_assignment_rows(repo_root)?
        .2
        .into_iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && matches!(
                    row.assignment_status,
                    FastqCorpusAssignmentStatus::Assigned
                        | FastqCorpusAssignmentStatus::AssetBacked
                )
        })
        .map(|row| binding_key("fastq", &row.stage_id, &row.tool_id))
        .chain(
            collect_bam_corpus_assignment_rows(repo_root)?
                .2
                .into_iter()
                .filter(|row| row.benchmark_status == "benchmark_ready")
                .map(|row| binding_key("bam", &row.stage_id, &row.tool_id)),
        )
        .collect::<BTreeSet<_>>();

    let fastq_report_stage_ids = collect_fastq_report_map_rows(repo_root)?
        .into_iter()
        .map(|row| row.stage_id)
        .collect::<BTreeSet<_>>();
    let bam_report_stage_ids = collect_bam_report_map_rows(repo_root)?
        .into_iter()
        .map(|row| row.stage_id)
        .collect::<BTreeSet<_>>();
    let ready_stage_count = benchmark_ready_keys
        .iter()
        .map(|key| (key.domain.clone(), key.stage_id.clone()))
        .collect::<BTreeSet<_>>()
        .len();

    let unregistered_by_key = unregistered_pairs
        .rows
        .iter()
        .map(|row| {
            (binding_key(&row.domain, &row.stage_id, &row.tool_id), row.registry_status.clone())
        })
        .collect::<BTreeMap<_, _>>();

    let mut failures = BTreeMap::<BenchmarkBindingKey, Vec<String>>::new();
    let mut failure_reasons = BTreeMap::<BenchmarkBindingKey, Vec<String>>::new();

    for row in &missing_pairs.rows {
        push_failure(
            &mut failures,
            &mut failure_reasons,
            binding_key(&row.domain, &row.stage_id, &row.tool_id),
            "stage_tool_matrix",
            row.reason.clone(),
        );
    }
    for row in &undercovered_stages.rows {
        for tool_id in &row.missing_tool_ids {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                binding_key(&row.domain, &row.stage_id, tool_id),
                "stage_tool_matrix",
                row.reason.clone(),
            );
        }
    }
    for row in &unregistered_pairs.rows {
        let key = binding_key(&row.domain, &row.stage_id, &row.tool_id);
        if benchmark_ready_keys.contains(&key) {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                key,
                "tool_registry",
                row.reason.clone(),
            );
        }
    }
    for row in &registry_extra_pairs.rows {
        push_failure(
            &mut failures,
            &mut failure_reasons,
            binding_key(&row.domain, &row.stage_id, &row.tool_id),
            "tool_registry",
            row.reason.clone(),
        );
    }
    for row in &parser_gate.rows {
        if row.gate_status == ParserCompletenessGateStatus::Fail {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                binding_key(&row.domain, &row.stage_id, &row.tool_id),
                "parsers",
                row.reason.clone(),
            );
        }
    }
    for row in &corpus_asset_gate.rows {
        if row.gate_status == CorpusAssetCoverageGateStatus::Fail {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                binding_key(&row.domain, &row.stage_id, &row.tool_id),
                "asset_assignments",
                row.reason.clone(),
            );
        }
    }

    for key in &benchmark_ready_keys {
        if !generated_job_keys.contains(key) {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                key.clone(),
                "command_adapters",
                format!(
                    "benchmark-ready pair `{}` / `{}` / `{}` did not materialize into generated benchmark command rows",
                    key.domain, key.stage_id, key.tool_id
                ),
            );
        }
        if !ready_corpus_keys.contains(key) {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                key.clone(),
                "corpus_assignments",
                format!(
                    "benchmark-ready pair `{}` / `{}` / `{}` is missing a governed corpus assignment row",
                    key.domain, key.stage_id, key.tool_id
                ),
            );
        }
        if !expected_result_keys.contains(key) {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                key.clone(),
                "expected_results",
                format!(
                    "benchmark-ready pair `{}` / `{}` / `{}` is missing an expected benchmark result row",
                    key.domain, key.stage_id, key.tool_id
                ),
            );
        }
        let stage_is_report_mapped = match key.domain.as_str() {
            "fastq" => fastq_report_stage_ids.contains(&key.stage_id),
            "bam" => bam_report_stage_ids.contains(&key.stage_id),
            _ => false,
        };
        if !stage_is_report_mapped {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                key.clone(),
                "report_maps",
                format!(
                    "benchmark-ready pair `{}` / `{}` / `{}` is missing a governed report-map stage",
                    key.domain, key.stage_id, key.tool_id
                ),
            );
        }
    }

    for key in &generated_job_keys {
        if !benchmark_ready_keys.contains(key) {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                key.clone(),
                "command_adapters",
                format!(
                    "generated benchmark command rows unexpectedly include excluded pair `{}` / `{}` / `{}`",
                    key.domain, key.stage_id, key.tool_id
                ),
            );
        }
    }
    for key in &expected_result_keys {
        if !benchmark_ready_keys.contains(key) {
            push_failure(
                &mut failures,
                &mut failure_reasons,
                key.clone(),
                "expected_results",
                format!(
                    "expected benchmark results unexpectedly include excluded pair `{}` / `{}` / `{}`",
                    key.domain, key.stage_id, key.tool_id
                ),
            );
        }
    }

    let failing_pairs = failures
        .into_iter()
        .map(|(key, surfaces)| {
            let row = pair_index.get(&key);
            StageToolBenchmarkReadyFailingPair {
                row_id: pair_row_id(&key.domain, &key.stage_id, &key.tool_id),
                domain: key.domain.clone(),
                stage_id: key.stage_id.clone(),
                tool_id: key.tool_id.clone(),
                failure_surfaces: surfaces,
                benchmark_status: row
                    .map(|row| row.benchmark_status.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                readiness_gap: row
                    .map(|row| pair_readiness_gap_label(row.readiness_gap).to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                support_status: row
                    .map(|row| row.support_status.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                adapter_status: row
                    .map(|row| row.adapter_status.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                parser_status: row
                    .map(|row| row.parser_status.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                corpus_status: row
                    .map(|row| row.corpus_status.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                asset_status: row
                    .map(|row| pair_asset_status_label(row.asset_status).to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                registry_status: unregistered_by_key
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| "registered".to_string()),
                reason: failure_reasons.get(&key).cloned().unwrap_or_default().join(" "),
            }
        })
        .collect::<Vec<_>>();

    let excluded_pairs = pair_rows
        .iter()
        .filter(|row| row.benchmark_status != "benchmark_ready")
        .map(|row| {
            let key = binding_key(&row.domain, &row.stage_id, &row.tool_id);
            StageToolBenchmarkReadyExcludedPair {
                row_id: pair_row_id(&row.domain, &row.stage_id, &row.tool_id),
                domain: row.domain.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                readiness_gap: pair_readiness_gap_label(row.readiness_gap).to_string(),
                support_status: row.support_status.clone(),
                adapter_status: row.adapter_status.clone(),
                parser_status: row.parser_status.clone(),
                corpus_status: row.corpus_status.clone(),
                asset_status: pair_asset_status_label(row.asset_status).to_string(),
                registry_status: unregistered_by_key
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| "registered".to_string()),
                excluded_from_generated_jobs: !generated_job_keys.contains(&key),
                excluded_from_expected_results: !expected_result_keys.contains(&key),
                reason: row.reason.clone(),
            }
        })
        .collect::<Vec<_>>();

    let excluded_registry_gap_count =
        excluded_pairs.iter().filter(|row| row.registry_status != "registered").count();
    let passes_gate = failing_pairs.is_empty();

    let surface_summaries = vec![
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "stage_tool_matrix".to_string(),
            surface_status: if missing_pairs.ok && undercovered_stages.ok {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "admitted_pairs_and_multi_tool_stages".to_string(),
            expected_count: benchmark_ready_keys.len(),
            covered_count: benchmark_ready_keys.len().saturating_sub(missing_pairs.rows.len()),
            excluded_count: excluded_keys.len(),
            failing_count: missing_pairs.rows.len()
                + undercovered_stages
                    .rows
                    .iter()
                    .map(|row| row.missing_tool_ids.len())
                    .sum::<usize>(),
            evidence_paths: vec![
                DEFAULT_MISSING_BENCHMARK_PAIRS_PATH.to_string(),
                DEFAULT_UNDERCOVERED_STAGES_PATH.to_string(),
            ],
            detail: format!(
                "missing admitted benchmark pairs: {}; undercovered multi-tool stages: {}",
                missing_pairs.missing_pair_count, undercovered_stages.undercovered_stage_count
            ),
        },
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "tool_registry".to_string(),
            surface_status: if registry_extra_pairs.ok
                && unregistered_pairs.rows.iter().all(|row| {
                    !benchmark_ready_keys.contains(&binding_key(
                        &row.domain,
                        &row.stage_id,
                        &row.tool_id,
                    ))
                }) {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "benchmark_ready_pairs".to_string(),
            expected_count: benchmark_ready_keys.len(),
            covered_count: benchmark_ready_keys.len().saturating_sub(
                unregistered_pairs
                    .rows
                    .iter()
                    .filter(|row| {
                        benchmark_ready_keys.contains(&binding_key(
                            &row.domain,
                            &row.stage_id,
                            &row.tool_id,
                        ))
                    })
                    .count(),
            ),
            excluded_count: excluded_registry_gap_count,
            failing_count: unregistered_pairs
                .rows
                .iter()
                .filter(|row| {
                    benchmark_ready_keys.contains(&binding_key(
                        &row.domain,
                        &row.stage_id,
                        &row.tool_id,
                    ))
                })
                .count()
                + registry_extra_pairs.extra_pair_count,
            evidence_paths: vec![
                DEFAULT_UNREGISTERED_BENCHMARK_PAIRS_PATH.to_string(),
                DEFAULT_STAGE_REGISTRY_EXTRA_PAIRS_PATH.to_string(),
            ],
            detail: format!(
                "benchmark-ready registry gaps: {}; excluded registry gaps: {}; extra registry pairs: {}",
                unregistered_pairs
                    .rows
                    .iter()
                    .filter(|row| benchmark_ready_keys.contains(&binding_key(
                        &row.domain,
                        &row.stage_id,
                        &row.tool_id
                    )))
                    .count(),
                excluded_registry_gap_count,
                registry_extra_pairs.extra_pair_count
            ),
        },
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "command_adapters".to_string(),
            surface_status: if benchmark_ready_keys == generated_job_keys {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "benchmark_ready_pairs".to_string(),
            expected_count: benchmark_ready_keys.len(),
            covered_count: generated_job_keys.intersection(&benchmark_ready_keys).count(),
            excluded_count: excluded_keys.len(),
            failing_count: generated_job_keys.symmetric_difference(&benchmark_ready_keys).count(),
            evidence_paths: vec![DEFAULT_PAIR_READINESS_PATH.to_string()],
            detail: format!(
                "generated benchmark command rows: {}; excluded pairs kept out of generated jobs: {}",
                generated_job_keys.len(),
                excluded_pairs.iter().filter(|row| row.excluded_from_generated_jobs).count()
            ),
        },
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "parsers".to_string(),
            surface_status: if parser_gate.passes_gate {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "benchmark_ready_pairs".to_string(),
            expected_count: parser_gate.gate_row_count,
            covered_count: parser_gate.gate_passed_row_count,
            excluded_count: parser_gate.excluded_row_count,
            failing_count: parser_gate.gate_failed_row_count,
            evidence_paths: vec![DEFAULT_PARSER_COMPLETENESS_GATE_PATH.to_string()],
            detail: format!(
                "parser gate rows: {}; excluded rows: {}",
                parser_gate.gate_row_count, parser_gate.excluded_row_count
            ),
        },
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "corpus_assignments".to_string(),
            surface_status: if benchmark_ready_keys == ready_corpus_keys {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "benchmark_ready_pairs".to_string(),
            expected_count: benchmark_ready_keys.len(),
            covered_count: ready_corpus_keys.intersection(&benchmark_ready_keys).count(),
            excluded_count: excluded_keys.len(),
            failing_count: benchmark_ready_keys.difference(&ready_corpus_keys).count(),
            evidence_paths: vec![
                super::fastq_corpus_assignment::DEFAULT_FASTQ_CORPUS_ASSIGNMENT_PATH.to_string(),
                super::bam_corpus_assignment::DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH.to_string(),
            ],
            detail: format!(
                "benchmark-ready corpus assignment rows: {}; excluded pairs remain outside the generated job slice",
                ready_corpus_keys.len()
            ),
        },
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "asset_assignments".to_string(),
            surface_status: if corpus_asset_gate.passes_gate {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "benchmark_ready_pairs".to_string(),
            expected_count: corpus_asset_gate.benchmark_ready_row_count,
            covered_count: corpus_asset_gate.benchmark_ready_asset_assigned_row_count,
            excluded_count: corpus_asset_gate.excluded_row_count,
            failing_count: corpus_asset_gate.gate_failed_row_count,
            evidence_paths: vec![DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH.to_string()],
            detail: format!(
                "asset-required benchmark-ready pairs: {}; assigned: {}",
                corpus_asset_gate.benchmark_ready_asset_required_row_count,
                corpus_asset_gate.benchmark_ready_asset_assigned_row_count
            ),
        },
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "expected_results".to_string(),
            surface_status: if benchmark_ready_keys == expected_result_keys {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "benchmark_ready_pairs".to_string(),
            expected_count: benchmark_ready_keys.len(),
            covered_count: expected_result_keys.intersection(&benchmark_ready_keys).count(),
            excluded_count: excluded_pairs
                .iter()
                .filter(|row| row.excluded_from_expected_results)
                .count(),
            failing_count: expected_result_keys.symmetric_difference(&benchmark_ready_keys).count(),
            evidence_paths: vec![DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.to_string()],
            detail: format!(
                "expected benchmark result rows: {}; excluded pairs kept out of expected results: {}",
                expected_result_keys.len(),
                excluded_pairs.iter().filter(|row| row.excluded_from_expected_results).count()
            ),
        },
        StageToolBenchmarkReadySurfaceSummary {
            surface_id: "report_maps".to_string(),
            surface_status: if benchmark_ready_keys.iter().all(|key| match key.domain.as_str() {
                "fastq" => fastq_report_stage_ids.contains(&key.stage_id),
                "bam" => bam_report_stage_ids.contains(&key.stage_id),
                _ => false,
            }) {
                StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            } else {
                StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            },
            measured_scope: "benchmark_ready_stages".to_string(),
            expected_count: ready_stage_count,
            covered_count: benchmark_ready_keys
                .iter()
                .map(|key| (key.domain.clone(), key.stage_id.clone()))
                .filter(|(domain, stage_id)| match domain.as_str() {
                    "fastq" => fastq_report_stage_ids.contains(stage_id),
                    "bam" => bam_report_stage_ids.contains(stage_id),
                    _ => false,
                })
                .collect::<BTreeSet<_>>()
                .len(),
            excluded_count: 0,
            failing_count: benchmark_ready_keys
                .iter()
                .map(|key| (key.domain.clone(), key.stage_id.clone()))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter(|(domain, stage_id)| match domain.as_str() {
                    "fastq" => !fastq_report_stage_ids.contains(stage_id),
                    "bam" => !bam_report_stage_ids.contains(stage_id),
                    _ => true,
                })
                .count(),
            evidence_paths: vec![
                DEFAULT_FASTQ_REPORT_MAP_PATH.to_string(),
                DEFAULT_BAM_REPORT_MAP_PATH.to_string(),
            ],
            detail: format!(
                "ready-stage report map coverage: fastq={} bam={}",
                fastq_report_stage_ids.len(),
                bam_report_stage_ids.len()
            ),
        },
    ];

    let report = StageToolBenchmarkReadyReport {
        schema_version: STAGE_TOOL_BENCHMARK_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        passes_gate,
        expected_pair_count: pair_rows.len(),
        benchmark_ready_pair_count: benchmark_ready_keys.len(),
        excluded_pair_count: excluded_keys.len(),
        failing_pair_count: failing_pairs.len(),
        generated_job_pair_count: generated_job_keys.len(),
        expected_result_pair_count: expected_result_keys.len(),
        benchmark_ready_stage_count: ready_stage_count,
        excluded_registry_gap_count,
        surface_summaries,
        failing_pairs,
        excluded_pairs,
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&report)?)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BenchmarkBindingKey {
    BenchmarkBindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

fn pair_index_by_stage_tool(
    pair_index: &BTreeMap<BenchmarkBindingKey, PairReadinessRow>,
    stage_id: &str,
    tool_id: &str,
) -> Option<BenchmarkBindingKey> {
    pair_index.keys().find(|key| key.stage_id == stage_id && key.tool_id == tool_id).cloned()
}

fn push_failure(
    failures: &mut BTreeMap<BenchmarkBindingKey, Vec<String>>,
    failure_reasons: &mut BTreeMap<BenchmarkBindingKey, Vec<String>>,
    key: BenchmarkBindingKey,
    surface_id: &str,
    reason: String,
) {
    failures.entry(key.clone()).or_default().push(surface_id.to_string());
    failure_reasons.entry(key).or_default().push(reason);
}

fn pair_row_id(domain: &str, stage_id: &str, tool_id: &str) -> String {
    format!("{domain}:{stage_id}:{tool_id}")
}

fn pair_readiness_gap_label(gap: PairReadinessGap) -> &'static str {
    match gap {
        PairReadinessGap::None => "none",
        PairReadinessGap::Asset => "asset",
        PairReadinessGap::Corpus => "corpus",
        PairReadinessGap::Parser => "parser",
        PairReadinessGap::Adapter => "adapter",
        PairReadinessGap::Support => "support",
    }
}

fn pair_asset_status_label(status: PairAssetStatus) -> &'static str {
    match status {
        PairAssetStatus::Assigned => "assigned",
        PairAssetStatus::Missing => "missing",
        PairAssetStatus::NotRequired => "not_required",
    }
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
        render_stage_tool_benchmark_ready, DEFAULT_STAGE_TOOL_BENCHMARK_READY_PATH,
        STAGE_TOOL_BENCHMARK_READY_SCHEMA_VERSION,
    };

    struct CurrentDirGuard(PathBuf);

    impl CurrentDirGuard {
        fn enter(path: &std::path::Path) -> Self {
            let original = std::env::current_dir().expect("capture current dir");
            std::env::set_current_dir(path).expect("set current dir");
            Self(original)
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.0).expect("restore current dir");
        }
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn stage_tool_benchmark_ready_gate_tracks_ready_slice_and_excluded_pairs() {
        let root = repo_root();
        let _cwd = CurrentDirGuard::enter(&root);
        let report = render_stage_tool_benchmark_ready(
            &root,
            PathBuf::from(DEFAULT_STAGE_TOOL_BENCHMARK_READY_PATH),
        )
        .expect("render stage-tool benchmark-ready gate");

        assert_eq!(report.schema_version, STAGE_TOOL_BENCHMARK_READY_SCHEMA_VERSION);
        assert!(report.passes_gate);
        assert_eq!(report.expected_pair_count, 123);
        assert_eq!(report.benchmark_ready_pair_count, 118);
        assert_eq!(report.excluded_pair_count, 5);
        assert_eq!(report.failing_pair_count, 0);
        assert_eq!(report.generated_job_pair_count, 118);
        assert_eq!(report.expected_result_pair_count, 118);
        assert_eq!(report.benchmark_ready_stage_count, 50);
        assert_eq!(report.excluded_registry_gap_count, 4);
        assert!(
            report.surface_summaries.iter().any(|surface| {
                surface.surface_id == "tool_registry"
                    && surface.failing_count == 0
                    && surface.excluded_count == 4
            }),
            "tool registry surface must keep excluded registry drift visible without failing the ready slice"
        );
        assert!(report.excluded_pairs.iter().any(|row| {
            row.row_id == "fastq:fastq.trim_reads:seqpurge"
                && row.registry_status == "tool_missing"
                && row.excluded_from_generated_jobs
                && row.excluded_from_expected_results
        }));
        assert!(
            report.excluded_pairs.iter().all(|row| !row.row_id.starts_with("fastq:fastq.index_reference:")),
            "asset-backed index-reference pairs must stay out of the excluded slice once corpus assignments are governed"
        );
    }
}
