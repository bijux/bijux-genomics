use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_micro_benchmark_run::{
    render_micro_benchmark_run, MicroBenchmarkRunManifest,
    DEFAULT_MICRO_BENCHMARK_RUN_MANIFEST_PATH,
};
use crate::commands::benchmark::local_stage_result_manifest::path_relative_to_repo;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_MICRO_BENCHMARK_EXECUTION_READY_PATH: &str =
    "benchmarks/readiness/micro/MICRO_BENCHMARK_EXECUTION_READY.json";
const MICRO_BENCHMARK_EXECUTION_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.micro_benchmark_execution_ready.v1";

const REAL_SMOKE_CORE_SUMMARY_PATH: &str = "runs/bench/micro/core/REAL_SMOKE_CORE_SUMMARY.json";
const FASTQ_MICRO_SUMMARY_PATH: &str = "runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json";
const BAM_MICRO_SUMMARY_PATH: &str = "runs/bench/micro/bam/MICRO_BAM_SUMMARY.json";
const VCF_MICRO_SUMMARY_PATH: &str = "runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json";
const AMPLICON_MICRO_SUMMARY_PATH: &str =
    "runs/bench/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json";
const ADNA_MICRO_SUMMARY_PATH: &str = "runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json";
const EDNA_MICRO_SUMMARY_PATH: &str = "runs/bench/micro/pipelines/edna/MICRO_EDNA_SUMMARY.json";
const CORE_GERMLINE_MICRO_SUMMARY_PATH: &str =
    "runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json";
const MICRO_BENCHMARK_REPORT_PATH: &str = "runs/bench/micro/MICRO_BENCHMARK_REPORT.json";

const REQUIRED_MICRO_COMPONENT_IDS: &[&str] = &[
    "amplicon_micro_pipeline",
    "adna_micro_pipeline",
    "bam_micro_smoke_subset",
    "core_germline_micro_pipeline",
    "edna_micro_pipeline",
    "fastq_micro_smoke_subset",
    "real_smoke_core_subset",
    "vcf_micro_smoke_subset",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MicroBenchmarkExecutionReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MicroBenchmarkRepresentativeCoverageCheck {
    pub(crate) coverage_id: String,
    pub(crate) category: String,
    pub(crate) surface: String,
    pub(crate) output_path: String,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkExecutionReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) coverage_check_count: usize,
    pub(crate) passed_coverage_check_count: usize,
    pub(crate) failed_coverage_check_count: usize,
    pub(crate) failing_coverage_ids: Vec<String>,
    pub(crate) result_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) unavailable_row_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<MicroBenchmarkExecutionReadyGoalCheck>,
    pub(crate) coverage_checks: Vec<MicroBenchmarkRepresentativeCoverageCheck>,
}

#[derive(Debug, Deserialize)]
struct RealSmokeCoreSummary {
    schema_version: String,
    output_path: String,
    execution_count: usize,
    stage_execution_count: usize,
    pipeline_bridge_count: usize,
    domain_counts: BTreeMap<String, usize>,
    passes_behavior_test: bool,
}

#[derive(Debug, Deserialize)]
struct FamilyMicroSummary {
    schema_version: String,
    output_path: String,
    family_count: usize,
    local_smoke_count: usize,
    container_needed_count: usize,
    unavailable_count: usize,
    passes_behavior_test: bool,
}

#[derive(Debug, Deserialize)]
struct PipelineMicroSummary {
    schema_version: String,
    output_path: String,
    stage_count: usize,
    passes_behavior_test: bool,
}

#[derive(Debug, Deserialize)]
struct AdnaPipelineMicroSummary {
    schema_version: String,
    output_path: String,
    stage_count: usize,
    skipped_count: usize,
    passes_behavior_test: bool,
}

#[derive(Debug, Deserialize)]
struct MicroBenchmarkReportSummary {
    schema_version: String,
    json_output_path: String,
    micro_run_manifest_path: String,
    result_row_count: usize,
    complete_row_count: usize,
    failed_row_count: usize,
    missing_row_count: usize,
    unavailable_row_count: usize,
    insufficient_data_row_count: usize,
    runtime_row_count: usize,
    memory_source_row_count: usize,
    science_threshold_row_count: usize,
    passes_behavior_test: bool,
}

pub(crate) fn run_render_micro_benchmark_execution_ready(
    args: &parse::BenchReadinessRenderMicroBenchmarkExecutionReadyArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_micro_benchmark_execution_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_MICRO_BENCHMARK_EXECUTION_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_micro_benchmark_execution_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<MicroBenchmarkExecutionReadyReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let run_manifest = render_micro_benchmark_run(
        repo_root,
        PathBuf::from(DEFAULT_MICRO_BENCHMARK_RUN_MANIFEST_PATH),
    )?;
    let real_smoke =
        load_json_report::<RealSmokeCoreSummary>(repo_root, REAL_SMOKE_CORE_SUMMARY_PATH)?;
    let fastq = load_json_report::<FamilyMicroSummary>(repo_root, FASTQ_MICRO_SUMMARY_PATH)?;
    let bam = load_json_report::<FamilyMicroSummary>(repo_root, BAM_MICRO_SUMMARY_PATH)?;
    let vcf = load_json_report::<FamilyMicroSummary>(repo_root, VCF_MICRO_SUMMARY_PATH)?;
    let amplicon =
        load_json_report::<PipelineMicroSummary>(repo_root, AMPLICON_MICRO_SUMMARY_PATH)?;
    let adna = load_json_report::<AdnaPipelineMicroSummary>(repo_root, ADNA_MICRO_SUMMARY_PATH)?;
    let edna = load_json_report::<PipelineMicroSummary>(repo_root, EDNA_MICRO_SUMMARY_PATH)?;
    let core_germline =
        load_json_report::<PipelineMicroSummary>(repo_root, CORE_GERMLINE_MICRO_SUMMARY_PATH)?;
    let micro_report =
        load_json_report::<MicroBenchmarkReportSummary>(repo_root, MICRO_BENCHMARK_REPORT_PATH)?;

    let mut checks = Vec::new();
    record_goal_check(
        &mut checks,
        471,
        "local micro-benchmark command",
        Some(DEFAULT_MICRO_BENCHMARK_RUN_MANIFEST_PATH.to_string()),
        || validate_goal_471(&run_manifest),
    );
    record_goal_check(
        &mut checks,
        472,
        "fastq micro-benchmark run",
        Some(FASTQ_MICRO_SUMMARY_PATH.to_string()),
        || {
            validate_family_goal(
                &fastq,
                FASTQ_MICRO_SUMMARY_PATH,
                "bijux.bench.local_fastq_micro_smoke_subset.v1",
                "FASTQ",
            )
        },
    );
    record_goal_check(
        &mut checks,
        473,
        "bam micro-benchmark run",
        Some(BAM_MICRO_SUMMARY_PATH.to_string()),
        || {
            validate_family_goal(
                &bam,
                BAM_MICRO_SUMMARY_PATH,
                "bijux.bench.local_bam_micro_smoke_subset.v2",
                "BAM",
            )
        },
    );
    record_goal_check(
        &mut checks,
        474,
        "vcf micro-benchmark run",
        Some(VCF_MICRO_SUMMARY_PATH.to_string()),
        || {
            validate_family_goal(
                &vcf,
                VCF_MICRO_SUMMARY_PATH,
                "bijux.bench.local_vcf_micro_smoke_subset.v1",
                "VCF",
            )
        },
    );
    record_goal_check(
        &mut checks,
        475,
        "core FASTQ->BAM->VCF micro-pipeline run",
        Some(CORE_GERMLINE_MICRO_SUMMARY_PATH.to_string()),
        || {
            validate_pipeline_goal(
                &core_germline,
                CORE_GERMLINE_MICRO_SUMMARY_PATH,
                "bijux.bench.local_core_germline_micro_pipeline.v1",
                "core germline",
            )
        },
    );
    record_goal_check(
        &mut checks,
        476,
        "adna micro-pipeline run",
        Some(ADNA_MICRO_SUMMARY_PATH.to_string()),
        || validate_adna_goal(&adna),
    );
    record_goal_check(
        &mut checks,
        477,
        "edna taxonomy micro-pipeline run",
        Some(EDNA_MICRO_SUMMARY_PATH.to_string()),
        || {
            validate_pipeline_goal(
                &edna,
                EDNA_MICRO_SUMMARY_PATH,
                "bijux.bench.local_edna_micro_pipeline.v1",
                "eDNA",
            )
        },
    );
    record_goal_check(
        &mut checks,
        478,
        "amplicon micro-pipeline run",
        Some(AMPLICON_MICRO_SUMMARY_PATH.to_string()),
        || {
            validate_pipeline_goal(
                &amplicon,
                AMPLICON_MICRO_SUMMARY_PATH,
                "bijux.bench.local_amplicon_micro_pipeline.v1",
                "amplicon",
            )
        },
    );
    record_goal_check(
        &mut checks,
        479,
        "micro-benchmark report",
        Some(MICRO_BENCHMARK_REPORT_PATH.to_string()),
        || validate_goal_479(&micro_report, &run_manifest),
    );

    let mut coverage_checks = Vec::new();
    record_coverage_check(
        &mut coverage_checks,
        "report.health",
        "report",
        "micro report health",
        MICRO_BENCHMARK_REPORT_PATH,
        || validate_micro_report_health(&micro_report),
    );
    for domain in ["fastq", "bam", "vcf"] {
        record_coverage_check(
            &mut coverage_checks,
            &format!("domain.{domain}"),
            "domain",
            &format!("{domain} real execution"),
            REAL_SMOKE_CORE_SUMMARY_PATH,
            || validate_real_smoke_domain(&real_smoke, domain),
        );
    }
    record_coverage_check(
        &mut coverage_checks,
        "family.fastq",
        "family",
        "FASTQ family representative local execution",
        FASTQ_MICRO_SUMMARY_PATH,
        || validate_family_coverage(&fastq, "FASTQ"),
    );
    record_coverage_check(
        &mut coverage_checks,
        "family.bam",
        "family",
        "BAM family representative local execution",
        BAM_MICRO_SUMMARY_PATH,
        || validate_family_coverage(&bam, "BAM"),
    );
    record_coverage_check(
        &mut coverage_checks,
        "family.vcf",
        "family",
        "VCF family representative local execution",
        VCF_MICRO_SUMMARY_PATH,
        || validate_family_coverage(&vcf, "VCF"),
    );
    record_coverage_check(
        &mut coverage_checks,
        "pipeline.core_germline",
        "pipeline",
        "core germline pipeline execution",
        CORE_GERMLINE_MICRO_SUMMARY_PATH,
        || validate_pipeline_coverage(&core_germline, "core germline"),
    );
    record_coverage_check(
        &mut coverage_checks,
        "pipeline.adna",
        "pipeline",
        "adna pipeline execution",
        ADNA_MICRO_SUMMARY_PATH,
        || validate_adna_coverage(&adna),
    );
    record_coverage_check(
        &mut coverage_checks,
        "pipeline.edna",
        "pipeline",
        "edna pipeline execution",
        EDNA_MICRO_SUMMARY_PATH,
        || validate_pipeline_coverage(&edna, "eDNA"),
    );
    record_coverage_check(
        &mut coverage_checks,
        "pipeline.amplicon",
        "pipeline",
        "amplicon pipeline execution",
        AMPLICON_MICRO_SUMMARY_PATH,
        || validate_pipeline_coverage(&amplicon, "amplicon"),
    );

    let checked_goal_count = checks.len();
    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let failed_goal_count = checked_goal_count.saturating_sub(passed_goal_count);
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();

    let coverage_check_count = coverage_checks.len();
    let passed_coverage_check_count = coverage_checks.iter().filter(|check| check.ok).count();
    let failed_coverage_check_count =
        coverage_check_count.saturating_sub(passed_coverage_check_count);
    let failing_coverage_ids = coverage_checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.coverage_id.clone())
        .collect::<Vec<_>>();

    let report = MicroBenchmarkExecutionReadyReport {
        schema_version: MICRO_BENCHMARK_EXECUTION_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        coverage_check_count,
        passed_coverage_check_count,
        failed_coverage_check_count,
        failing_coverage_ids,
        result_row_count: micro_report.result_row_count,
        complete_row_count: micro_report.complete_row_count,
        unavailable_row_count: micro_report.unavailable_row_count,
        ok: failed_goal_count == 0 && failed_coverage_check_count == 0,
        checks,
        coverage_checks,
    };

    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn validate_goal_471(report: &MicroBenchmarkRunManifest) -> Result<String> {
    if report.schema_version != "bijux.bench.local_micro_benchmark_run.v1" {
        bail!("micro run schema drifted to `{}`", report.schema_version);
    }
    if report.manifest_path != DEFAULT_MICRO_BENCHMARK_RUN_MANIFEST_PATH {
        bail!("micro run manifest path drifted to `{}`", report.manifest_path);
    }
    if !report.passes_behavior_test {
        bail!("micro run behavior contract did not pass");
    }
    if report.result_row_count == 0
        || report.output_row_count == 0
        || report.log_row_count == 0
        || report.normalized_metric_row_count == 0
    {
        bail!("micro run must emit non-empty result, output, log, and normalized metric rows");
    }

    let component_ids = report
        .component_reports
        .iter()
        .map(|component| component.component_id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_component_ids =
        REQUIRED_MICRO_COMPONENT_IDS.iter().copied().collect::<BTreeSet<_>>();
    if component_ids != expected_component_ids {
        bail!("micro run component set drifted from the governed 8-surface slice");
    }

    Ok(format!(
        "validated {} result rows, {} outputs, {} logs, {} normalized metrics across {} component reports",
        report.result_row_count,
        report.output_row_count,
        report.log_row_count,
        report.normalized_metric_row_count,
        report.component_reports.len()
    ))
}

fn validate_family_goal(
    report: &FamilyMicroSummary,
    expected_output_path: &str,
    expected_schema_version: &str,
    domain_label: &str,
) -> Result<String> {
    if report.schema_version != expected_schema_version {
        bail!("{domain_label} micro summary schema drifted to `{}`", report.schema_version);
    }
    if report.output_path != expected_output_path {
        bail!("{domain_label} micro summary output path drifted to `{}`", report.output_path);
    }
    if !report.passes_behavior_test {
        bail!("{domain_label} micro summary behavior contract did not pass");
    }
    if report.family_count == 0 {
        bail!("{domain_label} micro summary must keep at least one retained family row");
    }
    if report.local_smoke_count + report.container_needed_count + report.unavailable_count
        != report.family_count
    {
        bail!("{domain_label} micro summary status counts drifted from family_count");
    }

    Ok(format!(
        "validated {} retained families with local={} container_needed={} unavailable={}",
        report.family_count,
        report.local_smoke_count,
        report.container_needed_count,
        report.unavailable_count
    ))
}

fn validate_pipeline_goal(
    report: &PipelineMicroSummary,
    expected_output_path: &str,
    expected_schema_version: &str,
    pipeline_label: &str,
) -> Result<String> {
    if report.schema_version != expected_schema_version {
        bail!("{pipeline_label} micro pipeline schema drifted to `{}`", report.schema_version);
    }
    if report.output_path != expected_output_path {
        bail!("{pipeline_label} micro pipeline output path drifted to `{}`", report.output_path);
    }
    if !report.passes_behavior_test {
        bail!("{pipeline_label} micro pipeline behavior contract did not pass");
    }
    if report.stage_count == 0 {
        bail!("{pipeline_label} micro pipeline must keep at least one stage row");
    }

    Ok(format!("validated {} executed stages", report.stage_count))
}

fn validate_adna_goal(report: &AdnaPipelineMicroSummary) -> Result<String> {
    if report.schema_version != "bijux.bench.local_adna_micro_pipeline.v1" {
        bail!("adna micro pipeline schema drifted to `{}`", report.schema_version);
    }
    if report.output_path != ADNA_MICRO_SUMMARY_PATH {
        bail!("adna micro pipeline output path drifted to `{}`", report.output_path);
    }
    if !report.passes_behavior_test {
        bail!("adna micro pipeline behavior contract did not pass");
    }
    if report.stage_count == 0 || report.skipped_count >= report.stage_count {
        bail!("adna micro pipeline must keep executed stage evidence alongside structured skips");
    }

    Ok(format!(
        "validated {} stage rows with {} structured skips",
        report.stage_count, report.skipped_count
    ))
}

fn validate_goal_479(
    report: &MicroBenchmarkReportSummary,
    run_manifest: &MicroBenchmarkRunManifest,
) -> Result<String> {
    if report.schema_version != "bijux.bench.local_micro_benchmark_report.v1" {
        bail!("micro benchmark report schema drifted to `{}`", report.schema_version);
    }
    if report.json_output_path != MICRO_BENCHMARK_REPORT_PATH {
        bail!("micro benchmark report output path drifted to `{}`", report.json_output_path);
    }
    if report.micro_run_manifest_path != run_manifest.manifest_path {
        bail!("micro benchmark report detached from the governed micro run manifest");
    }
    if !report.passes_behavior_test {
        bail!("micro benchmark report behavior contract did not pass");
    }
    if report.result_row_count != run_manifest.result_row_count
        || report.runtime_row_count != report.result_row_count
        || report.memory_source_row_count != report.result_row_count
        || report.science_threshold_row_count == 0
    {
        bail!("micro benchmark report counts drifted from the governed micro run contract");
    }

    Ok(format!(
        "validated result={} complete={} unavailable={} failed={} missing={} insufficient={}",
        report.result_row_count,
        report.complete_row_count,
        report.unavailable_row_count,
        report.failed_row_count,
        report.missing_row_count,
        report.insufficient_data_row_count
    ))
}

fn validate_micro_report_health(report: &MicroBenchmarkReportSummary) -> Result<String> {
    if report.failed_row_count != 0
        || report.missing_row_count != 0
        || report.insufficient_data_row_count != 0
    {
        bail!(
            "micro report keeps failed={} missing={} insufficient={} rows",
            report.failed_row_count,
            report.missing_row_count,
            report.insufficient_data_row_count
        );
    }

    Ok(format!(
        "complete={} unavailable={} with no failed, missing, or insufficient rows",
        report.complete_row_count, report.unavailable_row_count
    ))
}

fn validate_real_smoke_domain(report: &RealSmokeCoreSummary, domain: &str) -> Result<String> {
    if report.schema_version != "bijux.bench.local_real_smoke_core_subset.v1" {
        bail!("real-smoke core summary schema drifted to `{}`", report.schema_version);
    }
    if report.output_path != REAL_SMOKE_CORE_SUMMARY_PATH {
        bail!("real-smoke core summary output path drifted to `{}`", report.output_path);
    }
    if !report.passes_behavior_test {
        bail!("real-smoke core summary behavior contract did not pass");
    }
    if report.execution_count != 4
        || report.stage_execution_count != 3
        || report.pipeline_bridge_count != 1
    {
        bail!("real-smoke core summary execution shape drifted from the governed 4-row slice");
    }
    let domain_count = report.domain_counts.get(domain).copied().unwrap_or(0);
    if domain_count == 0 {
        bail!("real-smoke core summary is missing representative `{domain}` execution evidence");
    }

    Ok(format!("validated {domain_count} representative `{domain}` execution rows"))
}

fn validate_family_coverage(report: &FamilyMicroSummary, domain_label: &str) -> Result<String> {
    if !report.passes_behavior_test {
        bail!("{domain_label} family coverage cannot pass without the governed behavior contract");
    }
    if report.local_smoke_count == 0 {
        bail!("{domain_label} family coverage is missing representative local real execution");
    }

    Ok(format!(
        "local={} container_needed={} unavailable={} across {} retained families",
        report.local_smoke_count,
        report.container_needed_count,
        report.unavailable_count,
        report.family_count
    ))
}

fn validate_pipeline_coverage(
    report: &PipelineMicroSummary,
    pipeline_label: &str,
) -> Result<String> {
    if !report.passes_behavior_test || report.stage_count == 0 {
        bail!("{pipeline_label} pipeline is missing representative local real execution");
    }

    Ok(format!("validated {} stage rows", report.stage_count))
}

fn validate_adna_coverage(report: &AdnaPipelineMicroSummary) -> Result<String> {
    if !report.passes_behavior_test
        || report.stage_count == 0
        || report.skipped_count >= report.stage_count
    {
        bail!("adna pipeline is missing representative local real execution");
    }

    Ok(format!(
        "validated {} stage rows with {} governed skips",
        report.stage_count, report.skipped_count
    ))
}

fn record_goal_check<F>(
    checks: &mut Vec<MicroBenchmarkExecutionReadyGoalCheck>,
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    match check() {
        Ok(detail) => checks.push(MicroBenchmarkExecutionReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(MicroBenchmarkExecutionReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: false,
            detail: error.to_string(),
        }),
    }
}

fn record_coverage_check<F>(
    checks: &mut Vec<MicroBenchmarkRepresentativeCoverageCheck>,
    coverage_id: &str,
    category: &str,
    surface: &str,
    output_path: &str,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    match check() {
        Ok(detail) => checks.push(MicroBenchmarkRepresentativeCoverageCheck {
            coverage_id: coverage_id.to_string(),
            category: category.to_string(),
            surface: surface.to_string(),
            output_path: output_path.to_string(),
            ok: true,
            detail,
        }),
        Err(error) => checks.push(MicroBenchmarkRepresentativeCoverageCheck {
            coverage_id: coverage_id.to_string(),
            category: category.to_string(),
            surface: surface.to_string(),
            output_path: output_path.to_string(),
            ok: false,
            detail: error.to_string(),
        }),
    }
}

fn load_json_report<T>(repo_root: &Path, relative_path: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let absolute_path = repo_root.join(relative_path);
    serde_json::from_slice(
        &std::fs::read(&absolute_path)
            .with_context(|| format!("read {}", absolute_path.display()))?,
    )
    .with_context(|| format!("parse {}", absolute_path.display()))
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).map_or_else(|_| path.to_path_buf(), PathBuf::from)
}
