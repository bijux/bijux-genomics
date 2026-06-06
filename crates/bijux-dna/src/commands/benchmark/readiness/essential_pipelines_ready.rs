use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::essential_pipeline_corpus_assets::{
    render_essential_pipeline_corpus_assets, DEFAULT_ESSENTIAL_PIPELINE_CORPUS_ASSETS_PATH,
};
use super::essential_pipeline_failure_isolation::{
    render_essential_pipeline_failure_isolation,
    DEFAULT_ESSENTIAL_PIPELINE_FAILURE_ISOLATION_REPORT_PATH,
};
use super::essential_pipeline_partial_resume::{
    render_essential_pipeline_partial_resume, DEFAULT_ESSENTIAL_PIPELINE_PARTIAL_RESUME_REPORT_PATH,
};
use super::essential_pipeline_rendered_commands::{
    render_essential_pipeline_commands, DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH,
};
use super::essential_pipeline_report_map::{
    render_essential_pipeline_report_map, DEFAULT_ESSENTIAL_PIPELINE_REPORT_MAP_PATH,
};
use crate::commands::benchmark::local_essential_pipeline_fake_runs::{
    fake_run_essential_pipelines, DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_pipeline_dag::validate_pipeline_dag_path;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINES_READY_PATH: &str =
    "target/bench-readiness/ESSENTIAL_PIPELINES_READY.json";
const ESSENTIAL_PIPELINES_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.essential_pipelines_ready.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelinesReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelinesReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) pipeline_count: usize,
    pub(crate) dag_node_count: usize,
    pub(crate) corpus_asset_row_count: usize,
    pub(crate) rendered_command_row_count: usize,
    pub(crate) fake_run_node_count: usize,
    pub(crate) fake_run_output_count: usize,
    pub(crate) report_map_row_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<EssentialPipelinesReadyGoalCheck>,
}

struct PipelineGoalExpectation {
    goal_id: u32,
    pipeline_id: &'static str,
    domain: &'static str,
    default_corpus_id: &'static str,
    node_count: usize,
    edge_count: usize,
    profile_id: Option<&'static str>,
    check_count: usize,
}

const PIPELINE_GOAL_EXPECTATIONS: &[PipelineGoalExpectation] = &[
    PipelineGoalExpectation {
        goal_id: 261,
        pipeline_id: "core-germline-fastq-bam-vcf",
        domain: "cross",
        default_corpus_id: "corpus-01-mini",
        node_count: 12,
        edge_count: 15,
        profile_id: None,
        check_count: 0,
    },
    PipelineGoalExpectation {
        goal_id: 262,
        pipeline_id: "adna-pseudohaploid-fastq-bam-vcf",
        domain: "cross",
        default_corpus_id: "corpus-01-mini",
        node_count: 15,
        edge_count: 24,
        profile_id: Some("ancient_dna_pseudohaploid"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 263,
        pipeline_id: "adna-gl-fastq-bam-vcf",
        domain: "cross",
        default_corpus_id: "corpus-01-mini",
        node_count: 15,
        edge_count: 23,
        profile_id: Some("ancient_dna_gl"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 264,
        pipeline_id: "diploid-small-fastq-bam-vcf",
        domain: "cross",
        default_corpus_id: "corpus-01-mini",
        node_count: 16,
        edge_count: 24,
        profile_id: Some("diploid_small_sample"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 265,
        pipeline_id: "reference-panel-imputation",
        domain: "vcf",
        default_corpus_id: "vcf_production_regression",
        node_count: 5,
        edge_count: 7,
        profile_id: Some("reference_panel_imputation"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 266,
        pipeline_id: "popgen-structure-vcf",
        domain: "vcf",
        default_corpus_id: "vcf_production_regression",
        node_count: 4,
        edge_count: 6,
        profile_id: Some("population_structure_vcf"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 267,
        pipeline_id: "relatedness-segments-vcf",
        domain: "vcf",
        default_corpus_id: "vcf_production_regression",
        node_count: 4,
        edge_count: 3,
        profile_id: Some("relatedness_segments_vcf"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 268,
        pipeline_id: "bam-genotyping-to-vcf-downstream",
        domain: "cross",
        default_corpus_id: "corpus-01-bam-mini",
        node_count: 9,
        edge_count: 11,
        profile_id: Some("bam_genotyping_vcf_downstream"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 269,
        pipeline_id: "edna-taxonomy-no-vcf",
        domain: "fastq",
        default_corpus_id: "corpus-02-edna-mini",
        node_count: 6,
        edge_count: 10,
        profile_id: Some("edna_taxonomy_no_vcf"),
        check_count: 8,
    },
    PipelineGoalExpectation {
        goal_id: 270,
        pipeline_id: "amplicon-asv-otu-no-vcf",
        domain: "fastq",
        default_corpus_id: "corpus-03-amplicon-mini",
        node_count: 7,
        edge_count: 12,
        profile_id: Some("amplicon_asv_otu_no_vcf"),
        check_count: 8,
    },
];

pub(crate) fn run_render_essential_pipelines_ready(
    args: &parse::BenchReadinessRenderEssentialPipelinesReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_essential_pipelines_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ESSENTIAL_PIPELINES_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_essential_pipelines_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EssentialPipelinesReadyReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let mut dag_node_count = 0usize;
    let mut corpus_asset_row_count = 0usize;
    let mut rendered_command_row_count = 0usize;
    let mut fake_run_node_count = 0usize;
    let mut fake_run_output_count = 0usize;
    let mut report_map_row_count = 0usize;

    for expectation in PIPELINE_GOAL_EXPECTATIONS {
        let output_path =
            format!("target/local-ready/pipeline-dag/{}.json", expectation.pipeline_id);
        record_goal_check(
            &mut checks,
            expectation.goal_id,
            format!("pipeline {}", expectation.pipeline_id),
            Some(output_path),
            || {
                let report = validate_pipeline_dag_path(
                    repo_root,
                    &repo_root
                        .join("configs/pipelines/local")
                        .join(format!("{}.toml", expectation.pipeline_id)),
                    &repo_root
                        .join("target/local-ready/pipeline-dag")
                        .join(format!("{}.json", expectation.pipeline_id)),
                )?;
                if !report.valid
                    || !report.acyclic
                    || report.pipeline_id != expectation.pipeline_id
                    || report.domain != expectation.domain
                    || report.default_corpus_id != expectation.default_corpus_id
                    || report.node_count != expectation.node_count
                    || report.edge_count != expectation.edge_count
                {
                    bail!("pipeline validation drifted for `{}`", expectation.pipeline_id);
                }
                match expectation.profile_id {
                    None => {
                        if !report.validation_profiles.is_empty() {
                            bail!(
                                "pipeline `{}` unexpectedly gained validation profiles",
                                expectation.pipeline_id
                            );
                        }
                    }
                    Some(profile_id) => {
                        if report.validation_profiles.len() != 1
                            || report.validation_profiles[0].profile_id != profile_id
                            || report.validation_profiles[0].check_count != expectation.check_count
                        {
                            bail!(
                                "pipeline `{}` validation profile drifted",
                                expectation.pipeline_id
                            );
                        }
                    }
                }
                dag_node_count += report.node_count;
                Ok(format!(
                    "validated `{}` with {} nodes and {} edges",
                    expectation.pipeline_id, report.node_count, report.edge_count
                ))
            },
        );
    }

    record_goal_check(
        &mut checks,
        271,
        "essential pipeline corpus/assets",
        Some(DEFAULT_ESSENTIAL_PIPELINE_CORPUS_ASSETS_PATH.to_string()),
        || {
            let report = render_essential_pipeline_corpus_assets(
                repo_root,
                PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_CORPUS_ASSETS_PATH),
            )?;
            if report.pipeline_count != 10
                || report.row_count != 93
                || report.resolved_row_count != 93
                || report.corpus_count != 11
                || report.asset_profile_count != 25
            {
                bail!("essential pipeline corpus/assets report drifted");
            }
            corpus_asset_row_count = report.row_count;
            Ok("validated 93 governed corpus/assets bindings across the essential 10-pipeline set"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        272,
        "essential pipeline commands",
        Some(DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH.to_string()),
        || {
            let report = render_essential_pipeline_commands(
                repo_root,
                PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH),
            )?;
            if report.pipeline_count != 10
                || report.row_count != 93
                || report.rendered_row_count != 93
                || report.structured_skip_row_count != 0
            {
                bail!("essential pipeline command rendering drifted");
            }
            rendered_command_row_count = report.row_count;
            Ok("validated executable command rendering for all 93 essential pipeline nodes"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        273,
        "essential pipeline fake-runner",
        Some(DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT.to_string()),
        || {
            let report = fake_run_essential_pipelines(
                repo_root,
                PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT),
            )?;
            if report.pipeline_count != 10
                || report.node_count != 93
                || report.created_output_count != 267
            {
                bail!("essential pipeline fake-runner drifted");
            }
            fake_run_node_count = report.node_count;
            fake_run_output_count = report.created_output_count;
            Ok("validated fake-run materialization for all essential pipeline nodes and outputs"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        274,
        "essential pipeline partial resume",
        Some(DEFAULT_ESSENTIAL_PIPELINE_PARTIAL_RESUME_REPORT_PATH.to_string()),
        || {
            let report = render_essential_pipeline_partial_resume(
                repo_root,
                PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_PARTIAL_RESUME_REPORT_PATH),
            )?;
            if report.pipeline_count != 10
                || report.node_count != 93
                || report.valid_completed_node_count != 92
                || report.invalid_manifest_node_count != 1
                || report.missing_manifest_node_count != 0
                || report.skip_node_count != 91
                || report.rerun_node_count != 2
                || !report.passes_behavior_test
            {
                bail!("essential pipeline partial-resume simulation drifted");
            }
            Ok("validated partial-resume behavior with one invalid manifest and two reruns"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        275,
        "essential pipeline failure isolation",
        Some(DEFAULT_ESSENTIAL_PIPELINE_FAILURE_ISOLATION_REPORT_PATH.to_string()),
        || {
            let report = render_essential_pipeline_failure_isolation(
                repo_root,
                PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_FAILURE_ISOLATION_REPORT_PATH),
            )?;
            if report.pipeline_count != 10
                || report.node_count != 93
                || report.completed_node_count != 91
                || report.failed_node_count != 1
                || report.blocked_node_count != 1
                || !report.passes_behavior_test
            {
                bail!("essential pipeline failure-isolation simulation drifted");
            }
            Ok("validated isolated failure propagation with one failed node and one blocked descendant".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        276,
        "essential pipeline report map",
        Some(DEFAULT_ESSENTIAL_PIPELINE_REPORT_MAP_PATH.to_string()),
        || {
            let report = render_essential_pipeline_report_map(
                repo_root,
                PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_REPORT_MAP_PATH),
            )?;
            if report.pipeline_count != 10
                || report.stage_count != 45
                || report.tool_count != 24
                || report.row_count != 267
                || report.report_section_count != 22
                || report.failure_column_count != 1
            {
                bail!("essential pipeline report map drifted");
            }
            report_map_row_count = report.row_count;
            Ok("validated report mapping for all 267 declared essential pipeline outputs"
                .to_string())
        },
    );

    if dag_node_count != 93 {
        bail!("essential pipeline readiness gate expected 93 DAG nodes, found {dag_node_count}");
    }
    if corpus_asset_row_count != dag_node_count
        || rendered_command_row_count != dag_node_count
        || fake_run_node_count != dag_node_count
    {
        bail!(
            "essential pipeline readiness gate node counts drifted: dag={dag_node_count}, corpus_assets={corpus_asset_row_count}, commands={rendered_command_row_count}, fake_runs={fake_run_node_count}"
        );
    }
    if fake_run_output_count != report_map_row_count {
        bail!(
            "essential pipeline readiness gate output counts drifted: fake_run_outputs={fake_run_output_count}, report_map_rows={report_map_row_count}"
        );
    }

    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let checked_goal_count = checks.len();
    let failed_goal_count = checked_goal_count.saturating_sub(passed_goal_count);
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect();
    let report = EssentialPipelinesReadyReport {
        schema_version: ESSENTIAL_PIPELINES_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        pipeline_count: 10,
        dag_node_count,
        corpus_asset_row_count,
        rendered_command_row_count,
        fake_run_node_count,
        fake_run_output_count,
        report_map_row_count,
        ok: failed_goal_count == 0,
        checks,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}

fn record_goal_check<F>(
    checks: &mut Vec<EssentialPipelinesReadyGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    let surface = surface.into();
    match check() {
        Ok(detail) => checks.push(EssentialPipelinesReadyGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(EssentialPipelinesReadyGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: false,
            detail: format!("{error:#}"),
        }),
    }
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
