use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::all_domain_rendered_commands::{
    render_all_domain_commands, AllDomainRenderedCommandRow,
    DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH: &str =
    "benchmarks/readiness/all-domains/local-job-coverage.tsv";
const ALL_DOMAIN_LOCAL_JOB_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_local_job_coverage.v1";
const COVERAGE_STATUS_COVERED: &str = "covered";
const COVERAGE_STATUS_MISSING_LOCAL_JOB: &str = "missing_local_job";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainLocalJobCoverageRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_status: String,
    pub(crate) command_source: String,
    pub(crate) command_step_count: usize,
    pub(crate) script_command_count: usize,
    pub(crate) command_step_ids: Vec<String>,
    pub(crate) primary_executables: Vec<String>,
    pub(crate) script_output_path: String,
    pub(crate) argv_output_path: String,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainLocalJobCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) script_output_path: String,
    pub(crate) argv_output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) local_job_binding_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) coverage_percent: f64,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) command_source_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainLocalJobCoverageRow>,
    pub(crate) violations: Vec<AllDomainLocalJobCoverageRow>,
}

pub(crate) fn run_render_all_domain_local_job_coverage(
    args: &parse::BenchReadinessRenderAllDomainLocalJobCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_local_job_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_local_job_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainLocalJobCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_local_job_coverage_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_local_job_coverage_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!(
            "all-domain active rows must keep complete local benchmark job coverage"
        ));
    }
    Ok(report)
}

fn build_all_domain_local_job_coverage_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainLocalJobCoverageReport> {
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let rendered_report = render_all_domain_commands(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
    )?;
    let rendered_by_key = rendered_report
        .rows
        .iter()
        .cloned()
        .map(|row| (coverage_key_from_rendered_row(&row), row))
        .collect::<BTreeMap<_, _>>();
    let active_keys = active_rows.iter().map(coverage_key_from_active_row).collect::<BTreeSet<_>>();
    let rendered_keys = rendered_by_key.keys().cloned().collect::<BTreeSet<_>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in &active_rows {
        rows.push(render_row(
            active_row,
            rendered_by_key.get(&coverage_key_from_active_row(active_row)),
            &rendered_report.output_path,
            &rendered_report.argv_output_path,
        ));
    }
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let row_count = rows.len();
    let covered_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COVERED).count();
    let missing_row_count = row_count.saturating_sub(covered_row_count);
    let coverage_percent =
        if row_count == 0 { 0.0 } else { covered_row_count as f64 * 100.0 / row_count as f64 };
    let result_id_count = rows
        .iter()
        .map(|row| row.result_id.as_str())
        .filter(|result_id| *result_id != NO_VALUE)
        .collect::<BTreeSet<_>>()
        .len();
    let stage_count = rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut command_source_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *command_source_counts.entry(row.command_source.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COVERED)
        .cloned()
        .collect::<Vec<_>>();

    let report = AllDomainLocalJobCoverageReport {
        schema_version: ALL_DOMAIN_LOCAL_JOB_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        script_output_path: rendered_report.output_path,
        argv_output_path: rendered_report.argv_output_path,
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        local_job_binding_count: rendered_keys.len(),
        covered_row_count,
        missing_row_count,
        coverage_percent,
        domain_counts,
        command_source_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_local_job_coverage_contract(
        &active_rows,
        &active_keys,
        &rendered_keys,
        &report,
    )?;
    Ok(report)
}

fn render_row(
    active_row: &AllDomainActiveStageToolMatrixRow,
    rendered_row: Option<&AllDomainRenderedCommandRow>,
    script_output_path: &str,
    argv_output_path: &str,
) -> AllDomainLocalJobCoverageRow {
    match rendered_row {
        Some(rendered_row) => {
            let command_step_ids = rendered_row
                .command_steps
                .iter()
                .map(|step| step.step_id.clone())
                .collect::<Vec<_>>();
            let primary_executables = rendered_row
                .command_steps
                .iter()
                .map(|step| {
                    step.argv
                        .first()
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .unwrap_or(NO_VALUE)
                        .to_string()
                })
                .collect::<Vec<_>>();
            AllDomainLocalJobCoverageRow {
                result_id: rendered_row.result_id.clone(),
                domain: active_row.domain.clone(),
                stage_id: active_row.stage_id.clone(),
                tool_id: active_row.tool_id.clone(),
                corpus_id: active_row.corpus_id.clone(),
                asset_profile_id: active_row.asset_profile_id.clone(),
                adapter_id: active_row.adapter_id.clone(),
                parser_id: active_row.parser_id.clone(),
                schema_id: active_row.schema_id.clone(),
                readiness_kind: rendered_row.readiness_kind.clone(),
                benchmark_status: rendered_row.benchmark_status.clone(),
                command_source: rendered_row.command_source.clone(),
                command_step_count: rendered_row.command_steps.len(),
                script_command_count: rendered_row.script_commands.len(),
                command_step_ids,
                primary_executables,
                script_output_path: script_output_path.to_string(),
                argv_output_path: argv_output_path.to_string(),
                coverage_status: COVERAGE_STATUS_COVERED.to_string(),
                reason: format!(
                    "active row `{}` / `{}` / `{}` keeps one local benchmark job row in `{}` and `{}` through `{}`",
                    active_row.domain,
                    active_row.stage_id,
                    active_row.tool_id,
                    script_output_path,
                    argv_output_path,
                    rendered_row.command_source
                ),
            }
        }
        None => AllDomainLocalJobCoverageRow {
            result_id: NO_VALUE.to_string(),
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            parser_id: active_row.parser_id.clone(),
            schema_id: active_row.schema_id.clone(),
            readiness_kind: NO_VALUE.to_string(),
            benchmark_status: NO_VALUE.to_string(),
            command_source: NO_VALUE.to_string(),
            command_step_count: 0,
            script_command_count: 0,
            command_step_ids: Vec::new(),
            primary_executables: Vec::new(),
            script_output_path: script_output_path.to_string(),
            argv_output_path: argv_output_path.to_string(),
            coverage_status: COVERAGE_STATUS_MISSING_LOCAL_JOB.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` / `{}` / `{}` is missing a governed all-domain local benchmark job row",
                active_row.domain,
                active_row.stage_id,
                active_row.tool_id,
                active_row.corpus_id,
                active_row.asset_profile_id
            ),
        },
    }
}

fn ensure_all_domain_local_job_coverage_contract(
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    active_keys: &BTreeSet<CoverageKey>,
    rendered_keys: &BTreeSet<CoverageKey>,
    report: &AllDomainLocalJobCoverageReport,
) -> Result<()> {
    if report.row_count != active_rows.len() || report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain local-job coverage must keep exactly one row per active binding"
        ));
    }
    if report.covered_row_count + report.missing_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain local-job coverage drifted from its covered/missing counts"
        ));
    }
    let missing = active_keys.difference(rendered_keys).count();
    let extra = rendered_keys.difference(active_keys).count();
    if missing != report.missing_row_count || extra != 0 {
        return Err(anyhow!(
            "all-domain local-job coverage drifted from the governed active slice; missing={}, extra={}",
            missing,
            extra
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!(
            "all-domain local-job coverage violation count drifted from the violation rows"
        ));
    }
    if !report.ok || report.violation_count != 0 {
        return Err(anyhow!(
            "all-domain local-job coverage must keep governed local benchmark job coverage for every active row"
        ));
    }
    if report.row_count != 125
        || report.result_id_count != 125
        || report.local_job_binding_count != 125
    {
        return Err(anyhow!(
            "all-domain local-job coverage must retain exactly 125 governed active result rows, found {} rows, {} result ids, and {} local job rows",
            report.row_count,
            report.result_id_count,
            report.local_job_binding_count
        ));
    }
    if report.script_output_path != DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH
        || report.argv_output_path
            != "benchmarks/readiness/rendered-commands-all-domains.argv.jsonl"
    {
        return Err(anyhow!(
            "all-domain local-job coverage drifted from the governed rendered-command artifact paths"
        ));
    }
    require_governed_row(
        &report.rows,
        "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
        "fastq",
        "fastq.screen_taxonomy",
        "kraken2",
        "fastq_bam_command_adapter",
    )?;
    require_governed_row(
        &report.rows,
        "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
        "bam",
        "bam.kinship",
        "king",
        "fastq_bam_command_adapter",
    )?;
    require_governed_row(
        &report.rows,
        "vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools",
        "vcf",
        "vcf.postprocess",
        "bcftools",
        "vcf_bcftools_adapter",
    )?;
    Ok(())
}

fn require_governed_row(
    rows: &[AllDomainLocalJobCoverageRow],
    result_id: &str,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    command_source: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.result_id == result_id)
        .ok_or_else(|| anyhow!("all-domain local-job coverage is missing `{result_id}`"))?;
    if row.domain != domain
        || row.stage_id != stage_id
        || row.tool_id != tool_id
        || row.command_source != command_source
        || row.command_step_count == 0
        || row.script_command_count == 0
        || row.coverage_status != COVERAGE_STATUS_COVERED
    {
        return Err(anyhow!(
            "all-domain local-job coverage row `{result_id}` drifted from the governed contract"
        ));
    }
    Ok(())
}

fn coverage_key_from_active_row(active_row: &AllDomainActiveStageToolMatrixRow) -> CoverageKey {
    CoverageKey {
        domain: active_row.domain.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id.clone(),
        asset_profile_id: active_row.asset_profile_id.clone(),
    }
}

fn coverage_key_from_rendered_row(rendered_row: &AllDomainRenderedCommandRow) -> CoverageKey {
    CoverageKey {
        domain: rendered_row.domain.clone(),
        stage_id: rendered_row.stage_id.clone(),
        tool_id: rendered_row.tool_id.clone(),
        corpus_id: rendered_row.corpus_id.clone(),
        asset_profile_id: rendered_row.asset_profile_id.clone(),
    }
}

fn render_all_domain_local_job_coverage_tsv(rows: &[AllDomainLocalJobCoverageRow]) -> String {
    let mut rendered = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\treadiness_kind\tbenchmark_status\tcommand_source\tcommand_step_count\tscript_command_count\tcommand_step_ids\tprimary_executables\tscript_output_path\targv_output_path\tcoverage_status\treason\n",
    );
    for row in rows {
        let command_step_ids = row.command_step_ids.join(",");
        let primary_executables = row.primary_executables.join(",");
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.result_id,
            row.domain,
            row.stage_id,
            row.tool_id,
            row.corpus_id,
            row.asset_profile_id,
            row.adapter_id,
            row.parser_id,
            row.schema_id,
            row.readiness_kind,
            row.benchmark_status,
            row.command_source,
            row.command_step_count,
            row.script_command_count,
            command_step_ids,
            primary_executables,
            row.script_output_path,
            row.argv_output_path,
            row.coverage_status,
            row.reason
        ));
    }
    rendered
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
