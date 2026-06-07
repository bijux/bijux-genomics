use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_benchmark_summary::{
    render_local_benchmark_summary, BenchLocalBenchmarkSummaryReport,
};
use crate::commands::benchmark::local_corpus_fixture::{amplicon, bam, damage, edna, fastq};
use crate::commands::benchmark::local_corpus_skip_report::{
    render_corpus_skip_report_path, LocalCorpusSkipReport,
};
use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, LocalCorpusStageCompatibilityValidationReport,
    DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_dag_watchdog_simulation::{
    simulate_dag_watchdog_path, LocalDagWatchdogScenario, LocalDagWatchdogSimulationReport,
    DEFAULT_COMPLETION_RULES_REPORT_PATH, DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
    DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH, DEFAULT_PARTIAL_RESUME_REPORT_PATH,
};
use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, LocalPipelineDagValidationReport,
};
use crate::commands::benchmark::local_slurm_dependency_check::{
    validate_slurm_dependencies, BenchLocalSlurmDependencyCheckReport,
};
use crate::commands::benchmark::local_slurm_dry_run::{
    render_local_slurm_scripts, BenchLocalSlurmDryRunReport,
};
use crate::commands::benchmark::local_slurm_script_bodies::{
    validate_slurm_script_bodies, BenchLocalSlurmScriptBodyReport,
};
use crate::commands::benchmark::local_slurm_shell_syntax::{
    validate_slurm_shell_syntax, BenchLocalSlurmShellSyntaxReport,
};
use crate::commands::benchmark::local_slurm_submit_manifest::{
    render_slurm_submit_manifest, BenchLocalSlurmSubmitManifest,
};
use crate::commands::benchmark::local_stage_commands::{
    materialize_local_stage, render_local_stage_commands, BenchLocalStageCommandManifest,
};
use crate::commands::benchmark::local_stage_fake_runs::{
    fake_run_local_stage_commands, fake_run_local_stage_failures,
    BenchLocalStageFakeFailureManifest, BenchLocalStageFakeRunManifest,
    DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::benchmark::local_stage_manifest_completion::{
    check_local_stage_manifest_completion, BenchLocalStageManifestCompletionReport,
};
use crate::commands::benchmark::local_stage_output_completion::{
    check_local_stage_output_completion, BenchLocalStageOutputCompletionReport,
};
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, BenchStageResultResourceMetricSource,
};
use crate::commands::benchmark::local_stage_runtime_metrics::{
    collect_local_stage_runtime_metrics, BenchLocalStageRuntimeMetricsReport,
};
use crate::commands::benchmark::local_taxonomy_database_fixture::{
    validate_taxonomy_database_fixture_manifest_path, DEFAULT_TAXONOMY_MINI_MANIFEST_PATH,
};
use crate::commands::benchmark::local_tool_comparison_template::{
    render_local_tool_comparison_template, BenchLocalToolComparisonTemplateReport,
};
use crate::commands::cli::render;
use crate::commands::cli::SlurmSubmitCampaignArgs;
use crate::commands::hpc::{campaign_dry_run, prepare_foundation, submit_campaign};

pub(crate) const DEFAULT_HPC_SUBMISSION_READY_REPORT_PATH: &str =
    "target/local-ready/HPC_SUBMISSION_READY.json";
const LOCAL_HPC_SUBMISSION_READY_SCHEMA_VERSION: &str = "bijux.bench.local_hpc_submission_ready.v1";
const DEFAULT_STAGE_COMMANDS_PATH: &str = "target/local-ready/rendered-stage-commands.sh";
const DEFAULT_STAGE_OUTPUT_COMPLETION_REPORT_PATH: &str =
    "target/local-ready/output-completion-report.json";
const DEFAULT_STAGE_MANIFEST_COMPLETION_REPORT_PATH: &str =
    "target/local-ready/manifest-completion-report.json";
const DEFAULT_RUNTIME_METRICS_REPORT_PATH: &str = "target/local-ready/runtime-metrics.json";
const DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH: &str =
    "target/local-ready/tool-comparison-template.tsv";
const DEFAULT_BENCHMARK_SUMMARY_JSON_PATH: &str = "target/local-ready/benchmark-summary.json";
const DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH: &str = "target/local-ready/benchmark-summary.md";
const DEFAULT_LOCAL_STAGE_FAILURE_ROOT: &str = "target/local-fake-runs/failures";
const DEFAULT_SLURM_DRY_RUN_ROOT: &str = "target/slurm-dry-run";
const DEFAULT_SLURM_SUBMIT_MANIFEST_PATH: &str = "target/slurm-dry-run/submit-manifest.json";
const DEFAULT_SLURM_DEPENDENCY_CHECK_REPORT_PATH: &str =
    "target/slurm-dry-run/dependency-check.json";
const DEFAULT_SLURM_SHELL_SYNTAX_REPORT_PATH: &str = "target/slurm-dry-run/bash-n-report.json";
const DEFAULT_SLURM_SCRIPT_BODY_REPORT_PATH: &str =
    "target/slurm-dry-run/no-placeholder-report.json";
const DEFAULT_CORPUS_SKIP_REPORT_PATH: &str = "target/local-ready/corpus-skip-report.json";
const DEFAULT_HPC_SUPPORT_ROOT: &str = "artifacts/hpc/hpc-submission-ready";

const FASTQ_STAGE_GOALS: &[(&str, u32, &str)] = &[
    ("fastq.index_reference", 2, "target/local-ready/fastq.index_reference"),
    ("fastq.validate_reads", 3, "target/local-smoke/fastq.validate_reads"),
    ("fastq.profile_read_lengths", 4, "target/local-smoke/fastq.profile_read_lengths"),
    ("fastq.detect_adapters", 5, "target/local-smoke/fastq.detect_adapters"),
    ("fastq.detect_duplicates_premerge", 6, "target/local-smoke/fastq.detect_duplicates_premerge"),
    (
        "fastq.estimate_library_complexity_prealign",
        7,
        "target/local-smoke/fastq.estimate_library_complexity_prealign",
    ),
    ("fastq.trim_terminal_damage", 8, "target/local-smoke/fastq.trim_terminal_damage"),
    ("fastq.normalize_primers", 9, "target/local-smoke/fastq.normalize_primers"),
    ("fastq.trim_polyg_tails", 10, "target/local-smoke/fastq.trim_polyg_tails"),
    ("fastq.trim_reads", 11, "target/local-smoke/fastq.trim_reads"),
    ("fastq.filter_reads", 12, "target/local-smoke/fastq.filter_reads"),
    ("fastq.profile_reads", 13, "target/local-smoke/fastq.profile_reads"),
    ("fastq.deplete_rrna", 14, "target/local-ready/fastq.deplete_rrna"),
    ("fastq.merge_pairs", 15, "target/local-smoke/fastq.merge_pairs"),
    ("fastq.remove_duplicates", 16, "target/local-smoke/fastq.remove_duplicates"),
    ("fastq.filter_low_complexity", 17, "target/local-smoke/fastq.filter_low_complexity"),
    ("fastq.deplete_host", 18, "target/local-ready/fastq.deplete_host"),
    (
        "fastq.deplete_reference_contaminants",
        19,
        "target/local-ready/fastq.deplete_reference_contaminants",
    ),
    ("fastq.correct_errors", 20, "target/local-smoke/fastq.correct_errors"),
    ("fastq.extract_umis", 21, "target/local-smoke/fastq.extract_umis"),
    (
        "fastq.profile_overrepresented_sequences",
        22,
        "target/local-smoke/fastq.profile_overrepresented_sequences",
    ),
    ("fastq.remove_chimeras", 23, "target/local-smoke/fastq.remove_chimeras"),
    ("fastq.infer_asvs", 24, "target/local-smoke/fastq.infer_asvs"),
    ("fastq.cluster_otus", 25, "target/local-smoke/fastq.cluster_otus"),
    ("fastq.normalize_abundance", 26, "target/local-smoke/fastq.normalize_abundance"),
    ("fastq.screen_taxonomy", 27, "target/local-ready/fastq.screen_taxonomy"),
];

const BAM_STAGE_GOALS: &[(&str, u32, &str)] = &[
    ("bam.align", 29, "target/local-ready/bam.align"),
    ("bam.validate", 30, "target/local-smoke/bam.validate"),
    ("bam.qc_pre", 31, "target/local-smoke/bam.qc_pre"),
    ("bam.mapping_summary", 32, "target/local-smoke/bam.mapping_summary"),
    ("bam.filter", 33, "target/local-smoke/bam.filter"),
    ("bam.mapq_filter", 34, "target/local-smoke/bam.mapq_filter"),
    ("bam.length_filter", 35, "target/local-smoke/bam.length_filter"),
    ("bam.markdup", 36, "target/local-smoke/bam.markdup"),
    ("bam.duplication_metrics", 37, "target/local-smoke/bam.duplication_metrics"),
    ("bam.complexity", 38, "target/local-smoke/bam.complexity"),
    ("bam.coverage", 39, "target/local-smoke/bam.coverage"),
    ("bam.insert_size", 40, "target/local-smoke/bam.insert_size"),
    ("bam.gc_bias", 41, "target/local-smoke/bam.gc_bias"),
    ("bam.endogenous_content", 42, "target/local-smoke/bam.endogenous_content"),
    ("bam.overlap_correction", 43, "target/local-smoke/bam.overlap_correction"),
    ("bam.damage", 44, "target/local-smoke/bam.damage"),
    ("bam.authenticity", 45, "target/local-smoke/bam.authenticity"),
    ("bam.contamination", 46, "target/local-ready/bam.contamination"),
    ("bam.sex", 47, "target/local-smoke/bam.sex"),
    ("bam.bias_mitigation", 48, "target/local-smoke/bam.bias_mitigation"),
    ("bam.recalibration", 49, "target/local-smoke/bam.recalibration"),
    ("bam.haplogroups", 50, "target/local-ready/bam.haplogroups"),
    ("bam.genotyping", 51, "target/local-ready/bam.genotyping"),
    ("bam.kinship", 52, "target/local-smoke/bam.kinship"),
];

const PIPELINE_DAG_GOALS: &[(u32, &str, &str)] = &[
    (
        77,
        "benchmarks/configs/pipelines/local/fastq-core-preprocess.toml",
        "target/local-ready/pipeline-dag/fastq-core-preprocess.json",
    ),
    (
        78,
        "benchmarks/configs/pipelines/local/fastq-paired-merge.toml",
        "target/local-ready/pipeline-dag/fastq-paired-merge.json",
    ),
    (
        79,
        "benchmarks/configs/pipelines/local/fastq-edna-taxonomy.toml",
        "target/local-ready/pipeline-dag/fastq-edna-taxonomy.json",
    ),
    (
        80,
        "benchmarks/configs/pipelines/local/fastq-amplicon.toml",
        "target/local-ready/pipeline-dag/fastq-amplicon.json",
    ),
    (
        81,
        "benchmarks/configs/pipelines/local/fastq-umi.toml",
        "target/local-ready/pipeline-dag/fastq-umi.json",
    ),
    (
        82,
        "benchmarks/configs/pipelines/local/bam-core-qc.toml",
        "target/local-ready/pipeline-dag/bam-core-qc.json",
    ),
    (
        83,
        "benchmarks/configs/pipelines/local/bam-authenticity.toml",
        "target/local-ready/pipeline-dag/bam-authenticity.json",
    ),
    (
        84,
        "benchmarks/configs/pipelines/local/bam-genotyping.toml",
        "target/local-ready/pipeline-dag/bam-genotyping.json",
    ),
    (
        85,
        "benchmarks/configs/pipelines/local/bam-kinship.toml",
        "target/local-ready/pipeline-dag/bam-kinship.json",
    ),
    (
        86,
        "benchmarks/configs/pipelines/local/fastq-to-bam.toml",
        "target/local-ready/pipeline-dag/fastq-to-bam.json",
    ),
];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalHpcSubmissionReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) category: String,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalHpcSubmissionReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
}

pub(crate) fn run_validate_hpc_submission_ready(output: Option<PathBuf>, json: bool) -> Result<()> {
    validate_hpc_submission_ready_feature_gate()?;

    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_hpc_submission_ready(
        &repo_root,
        output.unwrap_or_else(|| PathBuf::from(DEFAULT_HPC_SUBMISSION_READY_REPORT_PATH)),
    )?;
    if json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_hpc_submission_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BenchLocalHpcSubmissionReadyReport> {
    validate_hpc_submission_ready_feature_gate()?;

    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let fastq_inventory = evaluate_fastq_inventory_goal(repo_root, &mut checks);
    let bam_inventory = evaluate_bam_inventory_goal(repo_root, &mut checks);
    evaluate_local_stage_goals(repo_root, &mut checks);
    evaluate_benchmark_harness_goals(repo_root, &mut checks, fastq_inventory, bam_inventory);
    evaluate_corpus_goals(repo_root, &mut checks);
    evaluate_pipeline_dag_goals(repo_root, &mut checks);
    evaluate_watchdog_goals(repo_root, &mut checks);
    evaluate_hpc_campaign_goals(repo_root, &mut checks);
    evaluate_slurm_goals(repo_root, &mut checks);

    let failing_goal_ids = checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.goal_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let report = BenchLocalHpcSubmissionReadyReport {
        schema_version: LOCAL_HPC_SUBMISSION_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count: checks.iter().filter(|check| check.ok).count(),
        failed_goal_count: failing_goal_ids.len(),
        ok: failing_goal_ids.is_empty(),
        failing_goal_ids,
        checks,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;

    if report.ok {
        Ok(report)
    } else {
        Err(anyhow!("local HPC submission readiness failed; see {}", report.output_path))
    }
}

#[cfg(feature = "bam_downstream")]
fn validate_hpc_submission_ready_feature_gate() -> Result<()> {
    Ok(())
}

#[cfg(not(feature = "bam_downstream"))]
fn validate_hpc_submission_ready_feature_gate() -> Result<()> {
    Err(anyhow!(
        "local HPC submission readiness requires the `bam_downstream` feature; rerun with `cargo run -p bijux-dna --features bam_downstream -- bench local validate-hpc-submission-ready`"
    ))
}

fn evaluate_fastq_inventory_goal(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<usize> {
    match load_local_stage_inventory(repo_root, BenchLocalDomain::Fastq) {
        Ok(inventory) if inventory.stage_count == 27 => {
            checks.push(ok_check(
                1,
                "stage_matrices",
                "FASTQ stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("FASTQ inventory contains {} stages", inventory.stage_count),
            ));
            Some(inventory.stage_count)
        }
        Ok(inventory) => {
            checks.push(fail_check(
                1,
                "stage_matrices",
                "FASTQ stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("expected 27 FASTQ stages but found {}", inventory.stage_count),
            ));
            None
        }
        Err(err) => {
            checks.push(fail_check(
                1,
                "stage_matrices",
                "FASTQ stage matrix covers the governed local inventory",
                Some("benchmarks/configs/local/fastq-stage-matrix.toml".to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}

fn evaluate_bam_inventory_goal(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<usize> {
    match load_local_stage_inventory(repo_root, BenchLocalDomain::Bam) {
        Ok(inventory) if inventory.stage_count == 24 => {
            checks.push(ok_check(
                28,
                "stage_matrices",
                "BAM stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("BAM inventory contains {} stages", inventory.stage_count),
            ));
            Some(inventory.stage_count)
        }
        Ok(inventory) => {
            checks.push(fail_check(
                28,
                "stage_matrices",
                "BAM stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("expected 24 BAM stages but found {}", inventory.stage_count),
            ));
            None
        }
        Err(err) => {
            checks.push(fail_check(
                28,
                "stage_matrices",
                "BAM stage matrix covers the governed local inventory",
                Some("benchmarks/configs/local/bam-stage-matrix.toml".to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}

fn evaluate_local_stage_goals(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    for &(stage_id, goal_id, expected_root) in
        FASTQ_STAGE_GOALS.iter().chain(BAM_STAGE_GOALS.iter())
    {
        let expected_root_path = repo_root.join(expected_root);
        match materialize_local_stage(repo_root, stage_id) {
            Ok(path) => {
                let absolute_path = absolutize(repo_root, &path);
                if !absolute_path.exists() {
                    checks.push(fail_check(
                        goal_id,
                        "local_smokes",
                        format!("{} local artifact materializes under governed target", stage_id),
                        Some(path_relative_to_repo(repo_root, &absolute_path)),
                        format!("materialized path `{}` does not exist", absolute_path.display()),
                    ));
                } else if !absolute_path.starts_with(&expected_root_path) {
                    checks.push(fail_check(
                        goal_id,
                        "local_smokes",
                        format!("{} local artifact materializes under governed target", stage_id),
                        Some(path_relative_to_repo(repo_root, &absolute_path)),
                        format!(
                            "materialized path `{}` is outside governed root `{}`",
                            absolute_path.display(),
                            expected_root_path.display()
                        ),
                    ));
                } else {
                    checks.push(ok_check(
                        goal_id,
                        "local_smokes",
                        format!("{} local artifact materializes under governed target", stage_id),
                        Some(path_relative_to_repo(repo_root, &absolute_path)),
                        format!(
                            "materialized `{}` under `{}`",
                            path_relative_to_repo(repo_root, &absolute_path),
                            expected_root
                        ),
                    ));
                }
            }
            Err(err) => {
                checks.push(fail_check(
                    goal_id,
                    "local_smokes",
                    format!("{} local artifact materializes under governed target", stage_id),
                    Some(expected_root.to_string()),
                    format!("{err:#}"),
                ));
            }
        }
    }
}

fn evaluate_benchmark_harness_goals(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    fastq_stage_count: Option<usize>,
    bam_stage_count: Option<usize>,
) {
    match (fastq_stage_count, bam_stage_count) {
        (Some(fastq_count), Some(bam_count)) if fastq_count == 27 && bam_count == 24 => {
            checks.push(ok_check(
                53,
                "benchmark_harness",
                "local stage listing covers the governed FASTQ and BAM slices",
                None,
                format!(
                    "list-stages surface resolves {fastq_count} FASTQ stages and {bam_count} BAM stages"
                ),
            ));
        }
        (Some(fastq_count), Some(bam_count)) => {
            checks.push(fail_check(
                53,
                "benchmark_harness",
                "local stage listing covers the governed FASTQ and BAM slices",
                None,
                format!(
                    "expected 27 FASTQ and 24 BAM stages but found {fastq_count} FASTQ and {bam_count} BAM"
                ),
            ));
        }
        _ => {
            checks.push(fail_check(
                53,
                "benchmark_harness",
                "local stage listing covers the governed FASTQ and BAM slices",
                None,
                "stage inventory loading failed earlier".to_string(),
            ));
        }
    }

    let rendered_stage_commands =
        match render_local_stage_commands(repo_root, PathBuf::from(DEFAULT_STAGE_COMMANDS_PATH)) {
            Ok(manifest) => {
                evaluate_stage_command_render_goals(repo_root, checks, &manifest);
                Some(manifest)
            }
            Err(err) => {
                checks.push(fail_check(
                    54,
                    "benchmark_harness",
                    "local stage commands render into one governed shell script",
                    Some(DEFAULT_STAGE_COMMANDS_PATH.to_string()),
                    format!("{err:#}"),
                ));
                checks.push(fail_check(
                    55,
                    "benchmark_harness",
                    "rendered stage command manifest carries declared fields for every stage",
                    Some("target/local-ready/rendered-stage-commands.json".to_string()),
                    "stage command rendering failed earlier".to_string(),
                ));
                None
            }
        };

    let fake_run_manifest = match fake_run_local_stage_commands(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT),
    ) {
        Ok(manifest) => {
            evaluate_fake_run_success_goal(repo_root, checks, &manifest);
            Some(manifest)
        }
        Err(err) => {
            checks.push(fail_check(
                56,
                "benchmark_harness",
                "fake-run stage harness materializes every governed stage output and manifest",
                Some(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    match fake_run_local_stage_failures(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAILURE_ROOT),
        &[],
        7,
    ) {
        Ok(manifest) => evaluate_fake_run_failure_goal(repo_root, checks, &manifest),
        Err(err) => checks.push(fail_check(
            57,
            "benchmark_harness",
            "fake-run failure harness records structured stage failures",
            Some(DEFAULT_LOCAL_STAGE_FAILURE_ROOT.to_string()),
            format!("{err:#}"),
        )),
    }

    let output_completion = match check_local_stage_output_completion(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT),
        PathBuf::from(DEFAULT_STAGE_OUTPUT_COMPLETION_REPORT_PATH),
    ) {
        Ok(report) => {
            evaluate_output_completion_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                58,
                "benchmark_harness",
                "output completion requires every declared stage output",
                Some(DEFAULT_STAGE_OUTPUT_COMPLETION_REPORT_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    let manifest_completion = match check_local_stage_manifest_completion(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT),
        PathBuf::from(DEFAULT_STAGE_MANIFEST_COMPLETION_REPORT_PATH),
    ) {
        Ok(report) => {
            evaluate_manifest_completion_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                59,
                "benchmark_harness",
                "manifest completion requires every stage-result manifest",
                Some(DEFAULT_STAGE_MANIFEST_COMPLETION_REPORT_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    evaluate_stage_result_manifest_goal(repo_root, checks, fake_run_manifest.as_ref());

    let runtime_metrics = match collect_local_stage_runtime_metrics(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT),
        PathBuf::from(DEFAULT_RUNTIME_METRICS_REPORT_PATH),
    ) {
        Ok(report) => {
            evaluate_runtime_metrics_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                61,
                "benchmark_harness",
                "runtime metrics extract validated start, finish, elapsed, exit, and status fields",
                Some(DEFAULT_RUNTIME_METRICS_REPORT_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    evaluate_resource_metrics_goal(repo_root, checks, fake_run_manifest.as_ref());

    let comparison_template = match render_local_tool_comparison_template(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT),
        PathBuf::from(DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH),
    ) {
        Ok(report) => {
            evaluate_tool_comparison_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                63,
                "benchmark_harness",
                "tool comparison template renders one governed row per benchmark stage",
                Some(DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    match render_local_benchmark_summary(
        repo_root,
        PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT),
        PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_JSON_PATH),
        PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH),
    ) {
        Ok(report) => evaluate_benchmark_summary_goal(
            checks,
            &report,
            rendered_stage_commands.as_ref(),
            output_completion.as_ref(),
            manifest_completion.as_ref(),
            runtime_metrics.as_ref(),
            comparison_template.as_ref(),
        ),
        Err(err) => checks.push(fail_check(
            64,
            "benchmark_harness",
            "benchmark summary renders governed JSON and Markdown surfaces",
            Some(DEFAULT_BENCHMARK_SUMMARY_JSON_PATH.to_string()),
            format!("{err:#}"),
        )),
    }
}

fn evaluate_stage_command_render_goals(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    manifest: &BenchLocalStageCommandManifest,
) {
    let script_path = repo_root.join(&manifest.script_output_path);
    let manifest_path = repo_root.join(&manifest.manifest_output_path);
    if script_path.is_file() && manifest.command_count == 51 {
        checks.push(ok_check(
            54,
            "benchmark_harness",
            "local stage commands render into one governed shell script",
            Some(manifest.script_output_path.clone()),
            format!("rendered {} stage commands", manifest.command_count),
        ));
    } else {
        checks.push(fail_check(
            54,
            "benchmark_harness",
            "local stage commands render into one governed shell script",
            Some(manifest.script_output_path.clone()),
            format!(
                "expected script file and 51 commands but found script_exists={} command_count={}",
                script_path.is_file(),
                manifest.command_count
            ),
        ));
    }

    let command_rows_are_complete = manifest.commands.iter().all(|entry| {
        !entry.stage_id.trim().is_empty()
            && !entry.tool_id.trim().is_empty()
            && !entry.command.trim().is_empty()
            && entry.threads > 0
            && entry.memory_mb > 0
    });
    if manifest_path.is_file() && manifest.commands.len() == 51 && command_rows_are_complete {
        checks.push(ok_check(
            55,
            "benchmark_harness",
            "rendered stage command manifest carries declared fields for every stage",
            Some(manifest.manifest_output_path.clone()),
            "all 51 rendered stage command rows carry stage, tool, resource, and command fields"
                .to_string(),
        ));
    } else {
        checks.push(fail_check(
            55,
            "benchmark_harness",
            "rendered stage command manifest carries declared fields for every stage",
            Some(manifest.manifest_output_path.clone()),
            format!(
                "manifest_exists={} row_count={} complete_rows={command_rows_are_complete}",
                manifest_path.is_file(),
                manifest.commands.len()
            ),
        ));
    }
}

fn evaluate_fake_run_success_goal(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    manifest: &BenchLocalStageFakeRunManifest,
) {
    let all_stage_outputs_exist = manifest
        .stages
        .iter()
        .all(|stage| stage.created_output_count == stage.declared_output_count);
    let all_manifests_exist =
        manifest.stages.iter().all(|stage| repo_root.join(&stage.stage_manifest_path).is_file());
    if manifest.stage_count == 51 && all_stage_outputs_exist && all_manifests_exist {
        checks.push(ok_check(
            56,
            "benchmark_harness",
            "fake-run stage harness materializes every governed stage output and manifest",
            Some(manifest.fake_run_root.clone()),
            format!("fake-run manifest covers {} stages", manifest.stage_count),
        ));
    } else {
        checks.push(fail_check(
            56,
            "benchmark_harness",
            "fake-run stage harness materializes every governed stage output and manifest",
            Some(manifest.fake_run_root.clone()),
            format!(
                "stage_count={} output_complete={} manifest_complete={}",
                manifest.stage_count, all_stage_outputs_exist, all_manifests_exist
            ),
        ));
    }
}

fn evaluate_fake_run_failure_goal(
    _repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    manifest: &BenchLocalStageFakeFailureManifest,
) {
    let fields_are_present = manifest.failures.iter().all(|failure| {
        !failure.stage_id.trim().is_empty()
            && !failure.tool_id.trim().is_empty()
            && !failure.command.trim().is_empty()
            && !failure.stderr_path.trim().is_empty()
            && !failure.failure_record_path.trim().is_empty()
            && failure.exit_code == 7
    });
    if manifest.stage_count == 51 && fields_are_present {
        checks.push(ok_check(
            57,
            "benchmark_harness",
            "fake-run failure harness records structured stage failures",
            Some(manifest.failure_root.clone()),
            format!("fake-run failure manifest covers {} stages", manifest.stage_count),
        ));
    } else {
        checks.push(fail_check(
            57,
            "benchmark_harness",
            "fake-run failure harness records structured stage failures",
            Some(manifest.failure_root.clone()),
            format!("stage_count={} structured_fields={fields_are_present}", manifest.stage_count),
        ));
    }
}

fn evaluate_output_completion_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalStageOutputCompletionReport,
) {
    if report.complete && report.stage_count == 51 && report.incomplete_stage_count == 0 {
        checks.push(ok_check(
            58,
            "benchmark_harness",
            "output completion requires every declared stage output",
            Some(report.report_output_path.clone()),
            format!("output completion marks all {} governed stages complete", report.stage_count),
        ));
    } else {
        checks.push(fail_check(
            58,
            "benchmark_harness",
            "output completion requires every declared stage output",
            Some(report.report_output_path.clone()),
            format!(
                "complete={} stage_count={} incomplete_stage_count={}",
                report.complete, report.stage_count, report.incomplete_stage_count
            ),
        ));
    }
}

fn evaluate_manifest_completion_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalStageManifestCompletionReport,
) {
    if report.complete && report.stage_count == 51 && report.incomplete_stage_count == 0 {
        checks.push(ok_check(
            59,
            "benchmark_harness",
            "manifest completion requires every stage-result manifest",
            Some(report.report_output_path.clone()),
            format!(
                "manifest completion marks all {} governed stages complete",
                report.stage_count
            ),
        ));
    } else {
        checks.push(fail_check(
            59,
            "benchmark_harness",
            "manifest completion requires every stage-result manifest",
            Some(report.report_output_path.clone()),
            format!(
                "complete={} stage_count={} incomplete_stage_count={}",
                report.complete, report.stage_count, report.incomplete_stage_count
            ),
        ));
    }
}

fn evaluate_stage_result_manifest_goal(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    fake_run_manifest: Option<&BenchLocalStageFakeRunManifest>,
) {
    let Some(fake_run_manifest) = fake_run_manifest else {
        checks.push(fail_check(
            60,
            "benchmark_harness",
            "stage-result manifests validate the governed runtime contract",
            Some("target/local-fake-runs/stages/*/stage-result.json".to_string()),
            "fake-run stage generation failed earlier".to_string(),
        ));
        return;
    };

    let mut validation_errors = Vec::new();
    for stage in &fake_run_manifest.stages {
        let manifest_path = repo_root.join(&stage.stage_manifest_path);
        if let Err(err) = load_validated_stage_result_manifest_path(&manifest_path) {
            validation_errors.push(format!("{}: {err:#}", stage.stage_id));
        }
    }
    if validation_errors.is_empty() {
        checks.push(ok_check(
            60,
            "benchmark_harness",
            "stage-result manifests validate the governed runtime contract",
            Some("target/local-fake-runs/stages/*/stage-result.json".to_string()),
            format!("validated {} governed stage-result manifests", fake_run_manifest.stage_count),
        ));
    } else {
        checks.push(fail_check(
            60,
            "benchmark_harness",
            "stage-result manifests validate the governed runtime contract",
            Some("target/local-fake-runs/stages/*/stage-result.json".to_string()),
            validation_errors.join("; "),
        ));
    }
}

fn evaluate_runtime_metrics_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalStageRuntimeMetricsReport,
) {
    let runtime_fields_are_present = report.stages.iter().all(|stage| {
        !stage.runtime_mode.trim().is_empty()
            && !stage.started_at.trim().is_empty()
            && !stage.finished_at.trim().is_empty()
    });
    if report.stage_count == 51 && runtime_fields_are_present {
        checks.push(ok_check(
            61,
            "benchmark_harness",
            "runtime metrics extract validated start, finish, elapsed, exit, and status fields",
            Some(report.report_output_path.clone()),
            format!("runtime metrics report covers {} stages", report.stage_count),
        ));
    } else {
        checks.push(fail_check(
            61,
            "benchmark_harness",
            "runtime metrics extract validated start, finish, elapsed, exit, and status fields",
            Some(report.report_output_path.clone()),
            format!(
                "stage_count={} runtime_fields_present={runtime_fields_are_present}",
                report.stage_count
            ),
        ));
    }
}

fn evaluate_resource_metrics_goal(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    fake_run_manifest: Option<&BenchLocalStageFakeRunManifest>,
) {
    let Some(fake_run_manifest) = fake_run_manifest else {
        checks.push(fail_check(
            62,
            "benchmark_harness",
            "stage-result manifests carry explicit resource metric provenance",
            Some("target/local-fake-runs/stages/*/stage-result.json".to_string()),
            "fake-run stage generation failed earlier".to_string(),
        ));
        return;
    };

    let mut invalid_sources = Vec::new();
    for stage in &fake_run_manifest.stages {
        let manifest_path = repo_root.join(&stage.stage_manifest_path);
        match load_validated_stage_result_manifest_path(&manifest_path) {
            Ok(manifest) => match manifest.resource_metrics.source {
                BenchStageResultResourceMetricSource::Measured
                | BenchStageResultResourceMetricSource::Estimated
                | BenchStageResultResourceMetricSource::NotAvailable => {}
            },
            Err(err) => invalid_sources.push(format!("{}: {err:#}", stage.stage_id)),
        }
    }
    if invalid_sources.is_empty() {
        checks.push(ok_check(
            62,
            "benchmark_harness",
            "stage-result manifests carry explicit resource metric provenance",
            Some("target/local-fake-runs/stages/*/stage-result.json".to_string()),
            format!(
                "resource_metrics.source is explicit for {} governed stage results",
                fake_run_manifest.stage_count
            ),
        ));
    } else {
        checks.push(fail_check(
            62,
            "benchmark_harness",
            "stage-result manifests carry explicit resource metric provenance",
            Some("target/local-fake-runs/stages/*/stage-result.json".to_string()),
            invalid_sources.join("; "),
        ));
    }
}

fn evaluate_tool_comparison_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalToolComparisonTemplateReport,
) {
    if report.row_count == 51 && report.rows.len() == 51 {
        checks.push(ok_check(
            63,
            "benchmark_harness",
            "tool comparison template renders one governed row per benchmark stage",
            Some(report.tsv_output_path.clone()),
            format!("tool comparison template contains {} rows", report.row_count),
        ));
    } else {
        checks.push(fail_check(
            63,
            "benchmark_harness",
            "tool comparison template renders one governed row per benchmark stage",
            Some(report.tsv_output_path.clone()),
            format!(
                "expected 51 rows but found row_count={} rows_len={}",
                report.row_count,
                report.rows.len()
            ),
        ));
    }
}

fn evaluate_benchmark_summary_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalBenchmarkSummaryReport,
    rendered_stage_commands: Option<&BenchLocalStageCommandManifest>,
    output_completion: Option<&BenchLocalStageOutputCompletionReport>,
    manifest_completion: Option<&BenchLocalStageManifestCompletionReport>,
    runtime_metrics: Option<&BenchLocalStageRuntimeMetricsReport>,
    comparison_template: Option<&BenchLocalToolComparisonTemplateReport>,
) {
    let upstream_reports_are_aligned = rendered_stage_commands.is_some()
        && output_completion.is_some()
        && manifest_completion.is_some()
        && runtime_metrics.is_some()
        && comparison_template.is_some();
    if report.stage_count == 51
        && report.ready_stage_count == 51
        && report.failed_stage_count == 0
        && upstream_reports_are_aligned
    {
        checks.push(ok_check(
            64,
            "benchmark_harness",
            "benchmark summary renders governed JSON and Markdown surfaces",
            Some(report.report_output_path.clone()),
            format!(
                "benchmark summary renders {} ready stages with governed Markdown companion",
                report.stage_count
            ),
        ));
    } else {
        checks.push(fail_check(
            64,
            "benchmark_harness",
            "benchmark summary renders governed JSON and Markdown surfaces",
            Some(report.report_output_path.clone()),
            format!(
                "stage_count={} ready_stage_count={} failed_stage_count={} upstream_reports_aligned={upstream_reports_are_aligned}",
                report.stage_count, report.ready_stage_count, report.failed_stage_count
            ),
        ));
    }
}

fn evaluate_corpus_goals(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let fastq_report = check_goal(
        65,
        "corpora",
        "corpus-01 mini FASTQ fixture validates sample identity and read counts",
        Some(fastq::DEFAULT_CORPUS_01_MINI_MANIFEST_PATH),
        || {
            let report = fastq::validate_fastq_corpus_fixture_manifest_path(
                repo_root,
                &repo_root.join(fastq::DEFAULT_CORPUS_01_MINI_MANIFEST_PATH),
            )?;
            if report.sample_count == 4 && report.valid {
                Ok(format!(
                    "corpus-01 mini FASTQ fixture validates {} samples",
                    report.sample_count
                ))
            } else {
                Err(anyhow!(
                    "expected 4 FASTQ samples and valid fixture but found sample_count={} valid={}",
                    report.sample_count,
                    report.valid
                ))
            }
        },
    );
    checks.push(fastq_report);

    let bam_report = check_goal(
        66,
        "corpora",
        "corpus-01 mini BAM fixture validates aligned sample contracts",
        Some(bam::DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH),
        || {
            let report = bam::validate_bam_corpus_fixture_manifest_path(
                repo_root,
                &repo_root.join(bam::DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH),
            )?;
            if report.sample_count == 2 && report.valid {
                Ok(format!(
                    "corpus-01 mini BAM fixture validates {} aligned samples",
                    report.sample_count
                ))
            } else {
                Err(anyhow!(
                    "expected 2 BAM samples and valid fixture but found sample_count={} valid={}",
                    report.sample_count,
                    report.valid
                ))
            }
        },
    );
    checks.push(bam_report);

    let damage_report = check_goal(
        67,
        "corpora",
        "corpus-01 damage fixture validates expected aDNA limitations and evidence",
        Some(damage::DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH),
        || {
            let report = damage::validate_bam_damage_fixture_manifest_path(
                repo_root,
                &repo_root.join(damage::DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH),
            )?;
            if report.valid && !report.expected_damage_path.trim().is_empty() {
                Ok(format!(
                    "damage fixture validates expected evidence at {}",
                    report.expected_damage_path
                ))
            } else {
                Err(anyhow!("damage fixture did not produce a valid expected-damage contract"))
            }
        },
    );
    checks.push(damage_report);

    let edna_report = match edna::validate_edna_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(edna::DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH),
    ) {
        Ok(report) => Some(report),
        Err(err) => {
            checks.push(fail_check(
                68,
                "corpora",
                "corpus-02 eDNA fixture validates mock-community FASTQ inputs",
                Some(edna::DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH.to_string()),
                format!("{err:#}"),
            ));
            checks.push(fail_check(
                70,
                "corpora",
                "corpus-02 expected taxonomy output validates sample-taxon truth rows",
                Some("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv".to_string()),
                "corpus-02 eDNA fixture validation failed earlier".to_string(),
            ));
            None
        }
    };
    if let Some(report) = edna_report {
        if report.valid && report.sample_count == 2 {
            checks.push(ok_check(
                68,
                "corpora",
                "corpus-02 eDNA fixture validates mock-community FASTQ inputs",
                Some(report.manifest_path.clone()),
                format!("corpus-02 eDNA fixture validates {} samples", report.sample_count),
            ));
        } else {
            checks.push(fail_check(
                68,
                "corpora",
                "corpus-02 eDNA fixture validates mock-community FASTQ inputs",
                Some(report.manifest_path.clone()),
                format!(
                    "expected 2 samples and valid fixture but found sample_count={} valid={}",
                    report.sample_count, report.valid
                ),
            ));
        }
        if report.expected_taxa_count == 3
            && report.expected_taxa_output_row_count == 6
            && !report.expected_taxa_path.trim().is_empty()
        {
            checks.push(ok_check(
                70,
                "corpora",
                "corpus-02 expected taxonomy output validates sample-taxon truth rows",
                Some(report.expected_taxa_path.clone()),
                format!(
                    "expected taxonomy contract carries {} taxa across {} rows",
                    report.expected_taxa_count, report.expected_taxa_output_row_count
                ),
            ));
        } else {
            checks.push(fail_check(
                70,
                "corpora",
                "corpus-02 expected taxonomy output validates sample-taxon truth rows",
                Some(report.expected_taxa_path.clone()),
                format!(
                    "expected_taxa_count={} expected_taxa_output_row_count={}",
                    report.expected_taxa_count, report.expected_taxa_output_row_count
                ),
            ));
        }
    }

    let taxonomy_report = check_goal(
        69,
        "corpora",
        "taxonomy-mini fixture validates lineage, index, and backend bundle ownership",
        Some(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH),
        || {
            let report = validate_taxonomy_database_fixture_manifest_path(
                repo_root,
                &repo_root.join(DEFAULT_TAXONOMY_MINI_MANIFEST_PATH),
            )?;
            if report.valid && report.taxa_count == 3 {
                Ok(format!(
                    "taxonomy-mini fixture validates {} taxa with {} source records",
                    report.taxa_count, report.source_record_count
                ))
            } else {
                Err(anyhow!(
                    "expected valid taxonomy fixture with 3 taxa but found taxa_count={} valid={}",
                    report.taxa_count,
                    report.valid
                ))
            }
        },
    );
    checks.push(taxonomy_report);

    let amplicon_report = match amplicon::validate_amplicon_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(amplicon::DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH),
    ) {
        Ok(report) => Some(report),
        Err(err) => {
            for (goal_id, surface, output_path) in [
                (
                    71,
                    "corpus-03 amplicon fixture validates FASTQ sample and control ownership",
                    amplicon::DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH.to_string(),
                ),
                (
                    72,
                    "corpus-03 primer metadata validates governed primer normalization inputs",
                    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv".to_string(),
                ),
                (
                    73,
                    "corpus-03 expected ASV table validates sample-aware amplicon truth rows",
                    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/expected_asvs.tsv".to_string(),
                ),
                (
                    74,
                    "corpus-03 chimera controls validate governed positive-control expectations",
                    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
                        .to_string(),
                ),
            ] {
                checks.push(fail_check(
                    goal_id,
                    "corpora",
                    surface,
                    Some(output_path),
                    format!("{err:#}"),
                ));
            }
            None
        }
    };
    if let Some(report) = amplicon_report {
        if report.valid && report.sample_count >= 3 && report.control_count >= 1 {
            checks.push(ok_check(
                71,
                "corpora",
                "corpus-03 amplicon fixture validates FASTQ sample and control ownership",
                Some(report.manifest_path.clone()),
                format!(
                    "amplicon fixture validates {} samples and {} controls",
                    report.sample_count, report.control_count
                ),
            ));
        } else {
            checks.push(fail_check(
                71,
                "corpora",
                "corpus-03 amplicon fixture validates FASTQ sample and control ownership",
                Some(report.manifest_path.clone()),
                format!(
                    "sample_count={} control_count={} valid={}",
                    report.sample_count, report.control_count, report.valid
                ),
            ));
        }
        if report.primer_table_row_count >= 1 && !report.primers_tsv_path.trim().is_empty() {
            checks.push(ok_check(
                72,
                "corpora",
                "corpus-03 primer metadata validates governed primer normalization inputs",
                Some(report.primers_tsv_path.clone()),
                format!("primer metadata table contains {} rows", report.primer_table_row_count),
            ));
        } else {
            checks.push(fail_check(
                72,
                "corpora",
                "corpus-03 primer metadata validates governed primer normalization inputs",
                Some(report.primers_tsv_path.clone()),
                format!("primer_table_row_count={}", report.primer_table_row_count),
            ));
        }
        if report.expected_asv_row_count >= 1 && !report.expected_asvs_path.trim().is_empty() {
            checks.push(ok_check(
                73,
                "corpora",
                "corpus-03 expected ASV table validates sample-aware amplicon truth rows",
                Some(report.expected_asvs_path.clone()),
                format!("expected ASV table contains {} rows", report.expected_asv_row_count),
            ));
        } else {
            checks.push(fail_check(
                73,
                "corpora",
                "corpus-03 expected ASV table validates sample-aware amplicon truth rows",
                Some(report.expected_asvs_path.clone()),
                format!("expected_asv_row_count={}", report.expected_asv_row_count),
            ));
        }
        if report.chimera_expectation_row_count >= 1
            && !report.chimera_controls_fasta_path.trim().is_empty()
            && !report.chimera_expectations_path.trim().is_empty()
        {
            checks.push(ok_check(
                74,
                "corpora",
                "corpus-03 chimera controls validate governed positive-control expectations",
                Some(report.chimera_expectations_path.clone()),
                format!(
                    "chimera expectations contain {} rows",
                    report.chimera_expectation_row_count
                ),
            ));
        } else {
            checks.push(fail_check(
                74,
                "corpora",
                "corpus-03 chimera controls validate governed positive-control expectations",
                Some(report.chimera_expectations_path.clone()),
                format!("chimera_expectation_row_count={}", report.chimera_expectation_row_count),
            ));
        }
    }

    let compatibility_report = match validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    ) {
        Ok(report) => {
            evaluate_corpus_stage_compatibility_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                75,
                "corpora",
                "corpus-stage compatibility matrix validates all 51 governed stages",
                Some(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    match render_corpus_skip_report_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
        &repo_root.join(DEFAULT_CORPUS_SKIP_REPORT_PATH),
    ) {
        Ok(report) => evaluate_corpus_skip_report_goal(checks, &report, compatibility_report),
        Err(err) => checks.push(fail_check(
            76,
            "corpora",
            "corpus skip report keeps incompatible fixtures and planner-only stages explicit",
            Some(DEFAULT_CORPUS_SKIP_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }
}

fn evaluate_corpus_stage_compatibility_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &LocalCorpusStageCompatibilityValidationReport,
) {
    if report.valid && report.stage_count == 51 {
        checks.push(ok_check(
            75,
            "corpora",
            "corpus-stage compatibility matrix validates all 51 governed stages",
            Some(report.matrix_path.clone()),
            format!(
                "compatibility matrix validates {} stages across {} fixtures",
                report.stage_count, report.fixture_count
            ),
        ));
    } else {
        checks.push(fail_check(
            75,
            "corpora",
            "corpus-stage compatibility matrix validates all 51 governed stages",
            Some(report.matrix_path.clone()),
            format!(
                "valid={} stage_count={} fixture_count={}",
                report.valid, report.stage_count, report.fixture_count
            ),
        ));
    }
}

fn evaluate_corpus_skip_report_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &LocalCorpusSkipReport,
    compatibility_report: Option<LocalCorpusStageCompatibilityValidationReport>,
) {
    let planner_only_count_matches = compatibility_report
        .as_ref()
        .map(|compatibility| {
            compatibility.planner_only_stage_count == report.planner_only_stage_count
        })
        .unwrap_or(true);
    let skip_rows_are_complete = report.skips.iter().all(|skip| {
        !skip.stage_id.trim().is_empty()
            && !skip.corpus_id.trim().is_empty()
            && !skip.reason.trim().is_empty()
            && !skip.replacement_corpus_id.trim().is_empty()
    });
    if report.stage_count == 51 && planner_only_count_matches && skip_rows_are_complete {
        checks.push(ok_check(
            76,
            "corpora",
            "corpus skip report keeps incompatible fixtures and planner-only stages explicit",
            Some(report.output_path.clone()),
            format!(
                "skip report records {} fixture skips and {} planner-only stages",
                report.skip_count, report.planner_only_stage_count
            ),
        ));
    } else {
        checks.push(fail_check(
            76,
            "corpora",
            "corpus skip report keeps incompatible fixtures and planner-only stages explicit",
            Some(report.output_path.clone()),
            format!(
                "stage_count={} planner_only_count_matches={} skip_rows_are_complete={}",
                report.stage_count, planner_only_count_matches, skip_rows_are_complete
            ),
        ));
    }
}

fn evaluate_pipeline_dag_goals(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    for &(goal_id, config_path, output_path) in PIPELINE_DAG_GOALS {
        match validate_pipeline_dag_path(
            repo_root,
            &repo_root.join(config_path),
            &repo_root.join(output_path),
        ) {
            Ok(report) => evaluate_pipeline_dag_goal(checks, goal_id, &report),
            Err(err) => checks.push(fail_check(
                goal_id,
                "pipeline_dags",
                format!("{} validates as an acyclic governed local DAG", config_path),
                Some(output_path.to_string()),
                format!("{err:#}"),
            )),
        }
    }
}

fn evaluate_pipeline_dag_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    goal_id: u32,
    report: &LocalPipelineDagValidationReport,
) {
    if report.valid && report.acyclic && report.node_count > 0 {
        checks.push(ok_check(
            goal_id,
            "pipeline_dags",
            format!("{} validates as an acyclic governed local DAG", report.pipeline_id),
            Some(report.output_path.clone()),
            format!(
                "pipeline `{}` validates {} nodes and {} edges",
                report.pipeline_id, report.node_count, report.edge_count
            ),
        ));
    } else {
        checks.push(fail_check(
            goal_id,
            "pipeline_dags",
            format!("{} validates as an acyclic governed local DAG", report.pipeline_id),
            Some(report.output_path.clone()),
            format!(
                "valid={} acyclic={} node_count={}",
                report.valid, report.acyclic, report.node_count
            ),
        ));
    }
}

fn evaluate_watchdog_goals(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    evaluate_watchdog_goal(
        repo_root,
        checks,
        87,
        LocalDagWatchdogScenario::NoGlobalWait,
        DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH,
        |report| report.no_global_wait_proven,
        "no-global-wait watchdog simulation keeps ready work moving while an unrelated branch is slow",
    );
    evaluate_watchdog_goal(
        repo_root,
        checks,
        88,
        LocalDagWatchdogScenario::FailureIsolation,
        DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
        |report| report.failure_isolation_proven,
        "failure-isolation watchdog simulation blocks only dependent work for the failed sample",
    );
    evaluate_watchdog_goal(
        repo_root,
        checks,
        89,
        LocalDagWatchdogScenario::PartialResume,
        DEFAULT_PARTIAL_RESUME_REPORT_PATH,
        |report| report.partial_resume_proven,
        "partial-resume watchdog simulation reuses valid nodes and replans only invalid or missing work",
    );
    evaluate_watchdog_goal(
        repo_root,
        checks,
        90,
        LocalDagWatchdogScenario::CompletionRules,
        DEFAULT_COMPLETION_RULES_REPORT_PATH,
        |report| report.completion_rules_proven,
        "completion-rules watchdog simulation requires zero exit, outputs, and result manifest together",
    );
}

fn evaluate_watchdog_goal<F>(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    goal_id: u32,
    scenario: LocalDagWatchdogScenario,
    output_path: &str,
    predicate: F,
    surface: &str,
) where
    F: Fn(&LocalDagWatchdogSimulationReport) -> bool,
{
    match simulate_dag_watchdog_path(repo_root, scenario, &repo_root.join(output_path)) {
        Ok(report) if predicate(&report) => checks.push(ok_check(
            goal_id,
            "pipeline_dags",
            surface,
            Some(report.output_path.clone()),
            format!("watchdog scenario `{}` produced a governed proof report", report.scenario),
        )),
        Ok(report) => checks.push(fail_check(
            goal_id,
            "pipeline_dags",
            surface,
            Some(report.output_path.clone()),
            format!("watchdog scenario `{}` did not prove the required invariant", report.scenario),
        )),
        Err(err) => checks.push(fail_check(
            goal_id,
            "pipeline_dags",
            surface,
            Some(output_path.to_string()),
            format!("{err:#}"),
        )),
    }
}

fn evaluate_hpc_campaign_goals(
    repo_root: &Path,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let cross_campaign_path = repo_root.join("benchmarks/configs/hpc/campaign/cross-mini.toml");
    let generic_campaign_path =
        repo_root.join("benchmarks/configs/hpc/campaign/generic-small.toml");
    match (
        campaign_dry_run(&cross_campaign_path, None, None),
        campaign_dry_run(&generic_campaign_path, None, None),
    ) {
        (Ok(cross_report), Ok(generic_report)) => checks.push(ok_check(
            91,
            "slurm_dry_run",
            "HPC campaign configs reference only real governed stage IDs",
            Some("benchmarks/configs/hpc/campaign/cross-mini.toml".to_string()),
            format!(
                "campaign stage validation passes for `{}` and `{}`",
                cross_report.campaign_id, generic_report.campaign_id
            ),
        )),
        (cross_result, generic_result) => {
            let mut details = Vec::new();
            if let Err(err) = cross_result {
                details.push(format!("cross-mini: {err:#}"));
            }
            if let Err(err) = generic_result {
                details.push(format!("generic-small: {err:#}"));
            }
            checks.push(fail_check(
                91,
                "slurm_dry_run",
                "HPC campaign configs reference only real governed stage IDs",
                Some("benchmarks/configs/hpc/campaign/cross-mini.toml".to_string()),
                details.join("; "),
            ));
        }
    }

    let support_root = repo_root.join(DEFAULT_HPC_SUPPORT_ROOT);
    match prepare_local_hpc_profile_support(&support_root) {
        Ok(support) => {
            let config_path =
                repo_root.join("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml");
            let dry_run = campaign_dry_run(
                &config_path,
                Some(&support.env_file_path),
                Some(&support.policy_path),
            );
            let preparation = prepare_foundation(
                &config_path,
                Some(&support.env_file_path),
                Some(&support.policy_path),
                false,
            );
            let submission = submit_campaign(&SlurmSubmitCampaignArgs {
                config: config_path.clone(),
                env_file: Some(support.env_file_path.clone()),
                user_policies: Some(support.policy_path.clone()),
                mock_submit: true,
                json: false,
            });

            match (dry_run, preparation, submission) {
                (Ok(dry_run), Ok(preparation), Ok(submission))
                    if dry_run.campaign_id == "adna-equus-caballus-local-ready"
                        && dry_run.planned_jobs.len() == 4
                        && preparation.actions.len() == 3
                        && submission.jobs.len() == 4
                        && submission.jobs.iter().all(|job| {
                            repo_root.join(&job.script_path).is_file()
                                && repo_root.join(&job.code_path).is_file()
                        }) =>
                {
                    let first_script_path = repo_root.join(&submission.jobs[0].script_path);
                    let script_body = fs::read_to_string(&first_script_path)
                        .with_context(|| format!("read {}", first_script_path.display()));
                    match script_body {
                        Ok(script_body)
                            if script_body.contains("BIJUX_CORPORA_PREPARE_LOCK=")
                                && script_body.contains("BIJUX_DATABASES_PREPARE_LOCK=")
                                && script_body.contains("BIJUX_IMAGES_PREPARE_LOCK=") =>
                        {
                            checks.push(ok_check(
                                99,
                                "slurm_dry_run",
                                "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                                Some(
                                    "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                        .to_string(),
                                ),
                                format!(
                                    "LUNARC profile dry-run planned {} jobs and mock submission wrote {} scripts",
                                    dry_run.planned_jobs.len(),
                                    submission.jobs.len()
                                ),
                            ));
                        }
                        Ok(_) => checks.push(fail_check(
                            99,
                            "slurm_dry_run",
                            "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                            Some(
                                "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                    .to_string(),
                            ),
                            "mock submission scripts are missing prepared foundation lock exports"
                                .to_string(),
                        )),
                        Err(err) => checks.push(fail_check(
                            99,
                            "slurm_dry_run",
                            "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                            Some(
                                "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                    .to_string(),
                            ),
                            format!("{err:#}"),
                        )),
                    }
                }
                (dry_run, preparation, submission) => {
                    let mut details = Vec::new();
                    match dry_run {
                        Ok(report) => details.push(format!(
                            "dry-run planned {} jobs for `{}`",
                            report.planned_jobs.len(),
                            report.campaign_id
                        )),
                        Err(err) => details.push(format!("campaign dry-run: {err:#}")),
                    }
                    match preparation {
                        Ok(report) => details.push(format!(
                            "prepare-foundation emitted {} actions",
                            report.actions.len()
                        )),
                        Err(err) => details.push(format!("prepare-foundation: {err:#}")),
                    }
                    match submission {
                        Ok(report) => details.push(format!(
                            "submit-campaign emitted {} jobs",
                            report.jobs.len()
                        )),
                        Err(err) => details.push(format!("submit-campaign: {err:#}")),
                    }
                    checks.push(fail_check(
                        99,
                        "slurm_dry_run",
                        "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
                        Some(
                            "benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml"
                                .to_string(),
                        ),
                        details.join("; "),
                    ));
                }
            }
        }
        Err(err) => checks.push(fail_check(
            99,
            "slurm_dry_run",
            "LUNARC local-ready profile dry-runs against prepared roots and prepared lock contracts",
            Some("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-local-ready.toml".to_string()),
            format!("{err:#}"),
        )),
    }
}

fn evaluate_slurm_goals(repo_root: &Path, checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>) {
    match render_local_slurm_scripts(
        repo_root,
        BenchLocalDomain::Fastq,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT).join("fastq"),
    ) {
        Ok(report) => evaluate_fastq_slurm_goal(checks, &report),
        Err(err) => checks.push(fail_check(
            92,
            "slurm_dry_run",
            "FASTQ local benchmark stages render governed SLURM dry-run scripts",
            Some("target/slurm-dry-run/fastq".to_string()),
            format!("{err:#}"),
        )),
    }

    let bam_slurm_report = match render_local_slurm_scripts(
        repo_root,
        BenchLocalDomain::Bam,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT).join("bam"),
    ) {
        Ok(report) => {
            evaluate_bam_slurm_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                93,
                "slurm_dry_run",
                "BAM local benchmark stages render governed SLURM dry-run scripts",
                Some("target/slurm-dry-run/bam".to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    match validate_slurm_script_bodies(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SCRIPT_BODY_REPORT_PATH),
    ) {
        Ok(report) => evaluate_slurm_script_body_goal(checks, &report),
        Err(err) => checks.push(fail_check(
            94,
            "slurm_dry_run",
            "generated SLURM script bodies call real repo commands and contain no placeholders",
            Some(DEFAULT_SLURM_SCRIPT_BODY_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }

    match validate_slurm_shell_syntax(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SHELL_SYNTAX_REPORT_PATH),
    ) {
        Ok(report) => evaluate_slurm_shell_syntax_goal(checks, &report),
        Err(err) => checks.push(fail_check(
            95,
            "slurm_dry_run",
            "generated SLURM scripts pass bash syntax validation",
            Some(DEFAULT_SLURM_SHELL_SYNTAX_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }

    let submit_manifest = match render_slurm_submit_manifest(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SUBMIT_MANIFEST_PATH),
    ) {
        Ok(report) => {
            evaluate_slurm_submit_manifest_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                96,
                "slurm_dry_run",
                "SLURM dry-run submit manifest records complete job metadata",
                Some(DEFAULT_SLURM_SUBMIT_MANIFEST_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    match validate_slurm_dependencies(
        repo_root,
        PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT),
        PathBuf::from(DEFAULT_SLURM_SUBMIT_MANIFEST_PATH),
        PathBuf::from(DEFAULT_SLURM_DEPENDENCY_CHECK_REPORT_PATH),
    ) {
        Ok(report) => evaluate_slurm_dependency_goal(checks, &report),
        Err(err) => checks.push(fail_check(
            97,
            "slurm_dry_run",
            "SLURM dependencies stay in exactly one source of truth",
            Some(DEFAULT_SLURM_DEPENDENCY_CHECK_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }

    evaluate_slurm_run_path_goal(checks, bam_slurm_report, submit_manifest);
}

fn evaluate_fastq_slurm_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalSlurmDryRunReport,
) {
    if report.script_count == 27 {
        checks.push(ok_check(
            92,
            "slurm_dry_run",
            "FASTQ local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("FASTQ SLURM dry-run rendered {} scripts", report.script_count),
        ));
    } else {
        checks.push(fail_check(
            92,
            "slurm_dry_run",
            "FASTQ local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("expected 27 FASTQ scripts but found {}", report.script_count),
        ));
    }
}

fn evaluate_bam_slurm_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalSlurmDryRunReport,
) {
    if report.script_count == 24 {
        checks.push(ok_check(
            93,
            "slurm_dry_run",
            "BAM local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("BAM SLURM dry-run rendered {} scripts", report.script_count),
        ));
    } else {
        checks.push(fail_check(
            93,
            "slurm_dry_run",
            "BAM local benchmark stages render governed SLURM dry-run scripts",
            Some(report.output_root.clone()),
            format!("expected 24 BAM scripts but found {}", report.script_count),
        ));
    }
}

fn evaluate_slurm_script_body_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalSlurmScriptBodyReport,
) {
    if report.ok && report.script_count == 264 {
        checks.push(ok_check(
            94,
            "slurm_dry_run",
            "generated SLURM script bodies call real repo commands and contain no placeholders",
            Some(report.report_path.clone()),
            format!("SLURM script body validation passed across {} scripts", report.script_count),
        ));
    } else {
        checks.push(fail_check(
            94,
            "slurm_dry_run",
            "generated SLURM script bodies call real repo commands and contain no placeholders",
            Some(report.report_path.clone()),
            format!(
                "ok={} script_count={} findings_count={}",
                report.ok, report.script_count, report.findings_count
            ),
        ));
    }
}

fn evaluate_slurm_shell_syntax_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalSlurmShellSyntaxReport,
) {
    if report.ok && report.script_count == 264 {
        checks.push(ok_check(
            95,
            "slurm_dry_run",
            "generated SLURM scripts pass bash syntax validation",
            Some(report.report_path.clone()),
            format!("SLURM shell syntax validation passed across {} scripts", report.script_count),
        ));
    } else {
        checks.push(fail_check(
            95,
            "slurm_dry_run",
            "generated SLURM scripts pass bash syntax validation",
            Some(report.report_path.clone()),
            format!(
                "ok={} script_count={} findings_count={}",
                report.ok, report.script_count, report.findings_count
            ),
        ));
    }
}

fn evaluate_slurm_submit_manifest_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalSlurmSubmitManifest,
) {
    let required_fields_are_present = report.jobs.iter().all(|job| {
        !job.job_name.trim().is_empty()
            && !job.domain.trim().is_empty()
            && !job.tool_id.trim().is_empty()
            && !job.script_path.trim().is_empty()
            && !job.logs.stdout_path.trim().is_empty()
            && !job.logs.stderr_path.trim().is_empty()
    });
    if report.job_count == 51 && required_fields_are_present {
        checks.push(ok_check(
            96,
            "slurm_dry_run",
            "SLURM dry-run submit manifest records complete job metadata",
            Some(report.manifest_path.clone()),
            format!("submit manifest records {} jobs", report.job_count),
        ));
    } else {
        checks.push(fail_check(
            96,
            "slurm_dry_run",
            "SLURM dry-run submit manifest records complete job metadata",
            Some(report.manifest_path.clone()),
            format!(
                "job_count={} required_fields_are_present={required_fields_are_present}",
                report.job_count
            ),
        ));
    }
}

fn evaluate_slurm_dependency_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &BenchLocalSlurmDependencyCheckReport,
) {
    if report.ok && report.findings_count == 0 {
        checks.push(ok_check(
            97,
            "slurm_dry_run",
            "SLURM dependencies stay in exactly one source of truth",
            Some(report.report_path.clone()),
            format!("dependency ownership is clean across {} dry-run jobs", report.job_count),
        ));
    } else {
        checks.push(fail_check(
            97,
            "slurm_dry_run",
            "SLURM dependencies stay in exactly one source of truth",
            Some(report.report_path.clone()),
            format!(
                "ok={} findings_count={} job_count={}",
                report.ok, report.findings_count, report.job_count
            ),
        ));
    }
}

fn evaluate_slurm_run_path_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    bam_slurm_report: Option<BenchLocalSlurmDryRunReport>,
    submit_manifest: Option<BenchLocalSlurmSubmitManifest>,
) {
    let Some(bam_slurm_report) = bam_slurm_report else {
        checks.push(fail_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("target/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "BAM SLURM script rendering failed earlier".to_string(),
        ));
        return;
    };
    let Some(submit_manifest) = submit_manifest else {
        checks.push(fail_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("target/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "SLURM submit manifest rendering failed earlier".to_string(),
        ));
        return;
    };

    let path_fields_are_present = bam_slurm_report.scripts.iter().all(|script| {
        script.stdout_path.contains("local-benchmark-dry-run")
            && script.stderr_path.contains("local-benchmark-dry-run")
            && script.result_root.contains("local-benchmark-dry-run")
            && script.stage_result_manifest_path.contains("local-benchmark-dry-run")
    }) && submit_manifest.jobs.iter().all(|job| {
        job.logs.stdout_path.contains("local-benchmark-dry-run")
            && job.logs.stderr_path.contains("local-benchmark-dry-run")
            && job.result_root.contains("local-benchmark-dry-run")
    });
    if path_fields_are_present {
        checks.push(ok_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("target/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "SLURM dry-run scripts and submit manifest both carry governed run-path fields"
                .to_string(),
        ));
    } else {
        checks.push(fail_check(
            98,
            "slurm_dry_run",
            "SLURM dry-run scripts resolve predictable log and result paths",
            Some("target/slurm-dry-run/runs/local-benchmark-dry-run".to_string()),
            "one or more generated SLURM path fields are missing the governed run-id layout"
                .to_string(),
        ));
    }
}

fn prepare_local_hpc_profile_support(root: &Path) -> Result<HpcProfileSupportPaths> {
    for name in [
        "corpora",
        "databases",
        "images",
        "scratch",
        "logs",
        "results",
        "code",
        "imports",
        "baselines",
    ] {
        fs::create_dir_all(root.join(name))
            .with_context(|| format!("create {}", root.join(name).display()))?;
    }
    let env_file_path = root.join("campaign.env");
    bijux_dna_infra::write_bytes(&env_file_path, [])?;

    let policy_path = root.join("layout.policy.toml");
    let rendered = format!(
        "[layout]\n\
corpora_root = \"{root}/corpora\"\n\
databases_root = \"{root}/databases\"\n\
images_root = \"{root}/images\"\n\
scratch_root = \"{root}/scratch\"\n\
logs_root = \"{root}/logs\"\n\
encrypted_results_root = \"{root}/results\"\n\
encrypted_code_root = \"{root}/code\"\n\
appraiser_imports_root = \"{root}/imports\"\n\
baselines_root = \"{root}/baselines\"\n",
        root = root.display()
    );
    bijux_dna_infra::write_bytes(&policy_path, rendered)?;

    Ok(HpcProfileSupportPaths { env_file_path, policy_path })
}

#[derive(Debug, Clone)]
struct HpcProfileSupportPaths {
    env_file_path: PathBuf,
    policy_path: PathBuf,
}

fn check_goal<F>(
    goal_id: u32,
    category: &str,
    surface: &str,
    output_path: Option<&str>,
    check: F,
) -> BenchLocalHpcSubmissionReadyGoalCheck
where
    F: FnOnce() -> Result<String>,
{
    match check() {
        Ok(detail) => {
            ok_check(goal_id, category, surface, output_path.map(ToString::to_string), detail)
        }
        Err(err) => fail_check(
            goal_id,
            category,
            surface,
            output_path.map(ToString::to_string),
            format!("{err:#}"),
        ),
    }
}

fn ok_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> BenchLocalHpcSubmissionReadyGoalCheck {
    BenchLocalHpcSubmissionReadyGoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: true,
        detail: detail.into(),
    }
}

fn fail_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> BenchLocalHpcSubmissionReadyGoalCheck {
    BenchLocalHpcSubmissionReadyGoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: false,
        detail: detail.into(),
    }
}

fn absolutize(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::{BAM_STAGE_GOALS, FASTQ_STAGE_GOALS, PIPELINE_DAG_GOALS};
    use std::collections::BTreeSet;

    #[test]
    fn local_stage_goal_mappings_cover_every_numbered_stage_goal_once() {
        let mapped_goal_ids = FASTQ_STAGE_GOALS
            .iter()
            .chain(BAM_STAGE_GOALS.iter())
            .map(|(_, goal_id, _)| *goal_id)
            .collect::<BTreeSet<_>>();

        assert_eq!(FASTQ_STAGE_GOALS.len(), 26);
        assert_eq!(BAM_STAGE_GOALS.len(), 24);
        assert_eq!(mapped_goal_ids.len(), 50);
        assert!(mapped_goal_ids.contains(&2));
        assert!(mapped_goal_ids.contains(&27));
        assert!(mapped_goal_ids.contains(&29));
        assert!(mapped_goal_ids.contains(&52));
    }

    #[test]
    fn pipeline_dag_goal_mappings_cover_every_governed_pipeline_goal_once() {
        let mapped_goal_ids =
            PIPELINE_DAG_GOALS.iter().map(|(goal_id, _, _)| *goal_id).collect::<BTreeSet<_>>();

        assert_eq!(PIPELINE_DAG_GOALS.len(), 10);
        assert_eq!(mapped_goal_ids.len(), 10);
        assert!(mapped_goal_ids.contains(&77));
        assert!(mapped_goal_ids.contains(&86));
    }
}
