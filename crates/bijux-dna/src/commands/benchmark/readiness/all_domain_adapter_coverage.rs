use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::all_domain_rendered_commands::{
    collect_all_domain_rendered_command_rows, AllDomainRenderedCommandRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH: &str =
    "benchmarks/readiness/all-domains/adapter-coverage.tsv";
const ALL_DOMAIN_ADAPTER_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_adapter_coverage.v1";
const COVERAGE_STATUS_COVERED: &str = "covered";
const COVERAGE_STATUS_MISSING_RENDERED_COMMAND: &str = "missing_rendered_command";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainAdapterCoverageRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) command_source: String,
    pub(crate) command_step_count: usize,
    pub(crate) script_command_count: usize,
    pub(crate) command_step_ids: Vec<String>,
    pub(crate) primary_executables: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainAdapterCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) rendered_command_binding_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) coverage_percent: f64,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) command_source_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainAdapterCoverageRow>,
    pub(crate) violations: Vec<AllDomainAdapterCoverageRow>,
}

pub(crate) fn run_render_all_domain_adapter_coverage(
    args: &parse::BenchReadinessRenderAllDomainAdapterCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_adapter_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_adapter_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainAdapterCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_adapter_coverage_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_adapter_coverage_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!(
            "all-domain active rows must keep complete executable command rendering"
        ));
    }
    Ok(report)
}

fn build_all_domain_adapter_coverage_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainAdapterCoverageReport> {
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let rendered_rows = collect_all_domain_rendered_command_rows(repo_root)?;
    let active_keys = active_rows.iter().map(coverage_key_from_active_row).collect::<BTreeSet<_>>();
    let rendered_by_key = rendered_rows
        .into_iter()
        .map(|row| (coverage_key_from_rendered_row(&row), row))
        .collect::<BTreeMap<_, _>>();
    let rendered_keys = rendered_by_key.keys().cloned().collect::<BTreeSet<_>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in &active_rows {
        rows.push(render_row(
            active_row,
            rendered_by_key.get(&coverage_key_from_active_row(active_row)),
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
    let result_id_count =
        rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
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

    let report = AllDomainAdapterCoverageReport {
        schema_version: ALL_DOMAIN_ADAPTER_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        rendered_command_binding_count: rendered_keys.len(),
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
    ensure_all_domain_adapter_coverage_contract(
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
) -> AllDomainAdapterCoverageRow {
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
            AllDomainAdapterCoverageRow {
                result_id: rendered_row.result_id.clone(),
                domain: active_row.domain.clone(),
                stage_id: active_row.stage_id.clone(),
                tool_id: active_row.tool_id.clone(),
                corpus_id: active_row.corpus_id.clone(),
                asset_profile_id: active_row.asset_profile_id.clone(),
                adapter_id: active_row.adapter_id.clone(),
                readiness_kind: rendered_row.readiness_kind.clone(),
                command_source: rendered_row.command_source.clone(),
                command_step_count: rendered_row.command_steps.len(),
                script_command_count: rendered_row.script_commands.len(),
                command_step_ids,
                primary_executables,
                coverage_status: COVERAGE_STATUS_COVERED.to_string(),
                reason: format!(
                    "active row `{}` / `{}` / `{}` keeps executable command rendering through `{}` with {} command step(s)",
                    active_row.domain,
                    active_row.stage_id,
                    active_row.tool_id,
                    rendered_row.command_source,
                    rendered_row.command_steps.len()
                ),
            }
        }
        None => AllDomainAdapterCoverageRow {
            result_id: NO_VALUE.to_string(),
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            readiness_kind: NO_VALUE.to_string(),
            command_source: NO_VALUE.to_string(),
            command_step_count: 0,
            script_command_count: 0,
            command_step_ids: Vec::new(),
            primary_executables: Vec::new(),
            coverage_status: COVERAGE_STATUS_MISSING_RENDERED_COMMAND.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` is missing all-domain rendered command proof",
                active_row.domain, active_row.stage_id, active_row.tool_id
            ),
        },
    }
}

fn ensure_all_domain_adapter_coverage_contract(
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    active_keys: &BTreeSet<CoverageKey>,
    rendered_keys: &BTreeSet<CoverageKey>,
    report: &AllDomainAdapterCoverageReport,
) -> Result<()> {
    if report.row_count != active_rows.len() || report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain adapter coverage must keep exactly one row per active binding"
        ));
    }
    if report.covered_row_count + report.missing_row_count != report.row_count {
        return Err(anyhow!("all-domain adapter coverage drifted from its covered/missing counts"));
    }
    if active_keys != rendered_keys {
        let missing_keys =
            active_keys.difference(rendered_keys).map(format_coverage_key).collect::<Vec<_>>();
        let extra_keys =
            rendered_keys.difference(active_keys).map(format_coverage_key).collect::<Vec<_>>();
        return Err(anyhow!(
            "all-domain adapter coverage drifted from active scope, missing rendered keys: [{}], extra rendered keys: [{}]",
            missing_keys.join(", "),
            extra_keys.join(", ")
        ));
    }

    let reported_keys =
        report.rows.iter().map(coverage_key_from_report_row).collect::<BTreeSet<_>>();
    if reported_keys != *active_keys {
        return Err(anyhow!(
            "all-domain adapter coverage TSV must match the governed active coverage-key set"
        ));
    }
    if report.result_id_count != report.row_count {
        return Err(anyhow!(
            "all-domain adapter coverage must keep exactly one unique result_id per active binding"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!("all-domain adapter coverage drifted from its violation set"));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!("all-domain adapter coverage cannot be ok while violations remain"));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "all-domain adapter coverage must keep explicit violations when failing"
        ));
    }
    if report.ok
        && (report.covered_row_count != report.row_count || report.coverage_percent != 100.0)
    {
        return Err(anyhow!(
            "all-domain adapter coverage must reach 100% when no violations remain"
        ));
    }
    for row in &report.rows {
        if row.result_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.readiness_kind.trim().is_empty()
            || row.command_source.trim().is_empty()
            || row.coverage_status.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain adapter coverage row `{}` / `{}` / `{}` is missing a required field",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COVERED {
            if row.command_step_count == 0
                || row.script_command_count == 0
                || row.command_step_ids.is_empty()
                || row.primary_executables.is_empty()
            {
                return Err(anyhow!(
                    "all-domain adapter coverage row `{}` / `{}` / `{}` is missing executable command detail",
                    row.domain,
                    row.stage_id,
                    row.tool_id
                ));
            }
            if row
                .primary_executables
                .iter()
                .any(|value| value.trim().is_empty() || value == NO_VALUE)
            {
                return Err(anyhow!(
                    "all-domain adapter coverage row `{}` / `{}` / `{}` has an empty executable",
                    row.domain,
                    row.stage_id,
                    row.tool_id
                ));
            }
        }
    }
    Ok(())
}

fn render_all_domain_adapter_coverage_tsv(rows: &[AllDomainAdapterCoverageRow]) -> String {
    let mut rendered = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\treadiness_kind\tcommand_source\tcommand_step_count\tscript_command_count\tcommand_step_ids\tprimary_executables\tcoverage_status\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.result_id),
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.command_source),
            row.command_step_count,
            row.script_command_count,
            sanitize_tsv(&row.command_step_ids.join(",")),
            sanitize_tsv(&row.primary_executables.join(",")),
            sanitize_tsv(&row.coverage_status),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn coverage_key_from_active_row(row: &AllDomainActiveStageToolMatrixRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn coverage_key_from_rendered_row(row: &AllDomainRenderedCommandRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn coverage_key_from_report_row(row: &AllDomainAdapterCoverageRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn format_coverage_key(key: &CoverageKey) -> String {
    format!("{}/{}/{}", key.domain, key.stage_id, key.tool_id)
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_all_domain_adapter_coverage, ALL_DOMAIN_ADAPTER_COVERAGE_SCHEMA_VERSION,
        DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_all_domain_adapter_coverage_reports_complete_active_rows() {
        let root = repo_root();
        let report = render_all_domain_adapter_coverage(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH),
        )
        .expect("render all-domain adapter coverage");

        assert_eq!(report.schema_version, ALL_DOMAIN_ADAPTER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH);
        assert_eq!(report.row_count, 125);
        assert_eq!(report.result_id_count, 125);
        assert_eq!(report.stage_count, 58);
        assert_eq!(report.tool_count, 66);
        assert_eq!(report.rendered_command_binding_count, 125);
        assert_eq!(report.covered_row_count, 125);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.coverage_percent, 100.0);
        assert_eq!(report.domain_counts.get("fastq"), Some(&63));
        assert_eq!(report.domain_counts.get("bam"), Some(&49));
        assert_eq!(report.domain_counts.get("vcf"), Some(&13));
        assert_eq!(report.command_source_counts.get("fastq_bam_command_adapter"), Some(&112));
        assert_eq!(report.command_source_counts.get("vcf_bcftools_adapter"), Some(&11));
        assert_eq!(report.command_source_counts.get("vcf_plink_family_adapter"), Some(&2));
        assert_eq!(report.coverage_status_counts.get("covered"), Some(&125));
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert!(report.violations.is_empty());

        assert!(report.rows.iter().any(|row| {
            row.result_id == "fastq:corpus-01-mini:fastq.trim_reads:sample-set:trimmomatic"
                && row.command_source == "fastq_bam_command_adapter"
                && row.command_step_count == 1
                && row.script_command_count == 1
                && row.command_step_ids == vec!["invoke"]
                && row.primary_executables == vec!["sh"]
        }));
        assert!(report.rows.iter().any(|row| {
            row.result_id == "bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:schmutzi"
                && row.command_source == "fastq_bam_command_adapter"
                && row.command_step_count == 1
                && row.script_command_count == 1
                && row.command_step_ids == vec!["invoke"]
                && row.primary_executables == vec!["/bin/sh"]
        }));
        assert!(report.rows.iter().any(|row| {
            row.result_id
                == "vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools"
                && row.command_source == "vcf_bcftools_adapter"
                && row.command_step_count >= 1
                && row.script_command_count >= 1
                && row.primary_executables.iter().any(|value| value == "bcftools")
        }));
    }
}
