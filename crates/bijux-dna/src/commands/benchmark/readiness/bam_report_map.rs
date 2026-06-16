use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::pipeline_contract::{
    optional_branches, stage_criticality, StageCriticality,
};
use bijux_dna_domain_bam::{
    bam_scientific_report_contract_for_stage, BamScientificReportContractV1, BamStage,
};
use bijux_dna_planner_bam::stage_api::default_tool_for_stage;
use serde::Serialize;

use super::expected_benchmark_results::{
    collect_expected_benchmark_result_rows, ExpectedBenchmarkResultRow,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_REPORT_MAP_PATH: &str = "benchmarks/readiness/bam/bam-report-map.tsv";
const BAM_REPORT_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_report_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamReportMapRow {
    pub(crate) result_row_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_family_id: String,
    pub(crate) fixture_id: String,
    pub(crate) sample_scope: String,
    pub(crate) canonical_stage_rank: usize,
    pub(crate) readiness_kind: String,
    pub(crate) criticality: String,
    pub(crate) support_status: String,
    pub(crate) anchor_tool_id: String,
    pub(crate) anchor_support_status: String,
    pub(crate) report_section_id: String,
    pub(crate) report_section_title: String,
    pub(crate) summary_table_id: String,
    pub(crate) summary_table_title: String,
    pub(crate) workflow_branch_id: Option<String>,
    pub(crate) scientific_context_required: Vec<String>,
    pub(crate) report_focus: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamReportMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) expected_result_row_count: usize,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) section_count: usize,
    pub(crate) summary_table_count: usize,
    pub(crate) section_counts: BTreeMap<String, usize>,
    pub(crate) branch_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamReportMapRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BamReportStageMetadata {
    canonical_stage_rank: usize,
    readiness_kind: String,
    criticality: String,
    anchor_tool_id: String,
    anchor_support_status: String,
    report_section_id: String,
    report_section_title: String,
    summary_table_id: String,
    summary_table_title: String,
    workflow_branch_id: Option<String>,
    scientific_context_required: Vec<String>,
    report_focus: String,
}

#[derive(Debug, Clone, Copy)]
struct BamReportPlacement {
    section_id: &'static str,
    section_title: &'static str,
    summary_table_id: &'static str,
    summary_table_title: &'static str,
    report_focus: &'static str,
}

pub(crate) fn run_render_bam_report_map(
    args: &parse::BenchReadinessRenderBamReportMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_report_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_REPORT_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_report_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamReportMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let expected_result_row_count = collect_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "bam")
        .count();
    let rows = collect_bam_report_map_rows(repo_root)?;
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let section_count =
        rows.iter().map(|row| row.report_section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut section_counts = BTreeMap::<String, usize>::new();
    let mut branch_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *section_counts.entry(row.report_section_id.clone()).or_default() += 1;
        *branch_counts
            .entry(row.workflow_branch_id.clone().unwrap_or_else(|| "core_alignment".to_string()))
            .or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_report_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(BamReportMapReport {
        schema_version: BAM_REPORT_MAP_SCHEMA_VERSION,
        domain: "bam",
        output_path: path_relative_to_repo(repo_root, &output_path),
        expected_result_row_count,
        row_count: rows.len(),
        stage_count,
        tool_count,
        section_count,
        summary_table_count,
        section_counts,
        branch_counts,
        rows,
    })
}

pub(crate) fn collect_bam_report_map_rows(repo_root: &Path) -> Result<Vec<BamReportMapRow>> {
    let stage_metadata = collect_bam_report_stage_metadata(repo_root)?;
    let stage_admissions =
        super::catalog::load_stage_admissions(repo_root, super::catalog::ReadinessDomain::Bam)?;
    let expected_rows = collect_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "bam")
        .collect::<Vec<_>>();
    let mut rows = Vec::with_capacity(expected_rows.len());

    for row in &expected_rows {
        let metadata = stage_metadata.get(&row.stage_id).ok_or_else(|| {
            anyhow!(
                "BAM report map is missing stage metadata for expected result `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        let admissions = stage_admissions.get(row.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing admitted benchmark tools for `{}`", row.stage_id)
        })?;
        let support_status = admission_support_status(admissions, row.tool_id.as_str())?;
        rows.push(BamReportMapRow {
            result_row_id: row.result_row_id.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_family_id: row.corpus_family_id.clone(),
            fixture_id: row.fixture_id.clone(),
            sample_scope: row.sample_scope.clone(),
            canonical_stage_rank: metadata.canonical_stage_rank,
            readiness_kind: metadata.readiness_kind.clone(),
            criticality: metadata.criticality.clone(),
            support_status: support_status.to_string(),
            anchor_tool_id: metadata.anchor_tool_id.clone(),
            anchor_support_status: metadata.anchor_support_status.clone(),
            report_section_id: metadata.report_section_id.clone(),
            report_section_title: metadata.report_section_title.clone(),
            summary_table_id: metadata.summary_table_id.clone(),
            summary_table_title: metadata.summary_table_title.clone(),
            workflow_branch_id: metadata.workflow_branch_id.clone(),
            scientific_context_required: metadata.scientific_context_required.clone(),
            report_focus: metadata.report_focus.clone(),
            reason: report_reason(
                row,
                metadata.report_section_id.as_str(),
                metadata.summary_table_id.as_str(),
                metadata.report_focus.as_str(),
                support_status,
                metadata.anchor_tool_id.as_str(),
                metadata.anchor_support_status.as_str(),
                metadata.workflow_branch_id.as_deref(),
                &metadata.scientific_context_required,
            ),
        });
    }

    rows.sort_by(|left, right| {
        left.canonical_stage_rank
            .cmp(&right.canonical_stage_rank)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.fixture_id.cmp(&right.fixture_id))
    });
    ensure_bam_report_map_contract(&rows, &expected_rows)?;
    Ok(rows)
}

fn collect_bam_report_stage_metadata(
    repo_root: &Path,
) -> Result<BTreeMap<String, BamReportStageMetadata>> {
    let inventory = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?;
    let stage_admissions =
        super::catalog::load_stage_admissions(repo_root, super::catalog::ReadinessDomain::Bam)?;
    let canonical_rank_by_stage = BamStage::all()
        .iter()
        .enumerate()
        .map(|(idx, stage)| (stage.as_str().to_string(), idx + 1))
        .collect::<BTreeMap<_, _>>();
    let branch_by_stage = branch_id_by_stage();

    let mut rows = BTreeMap::new();
    for stage in inventory.stages {
        let placement = placement_for_stage(stage.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing stage placement for `{}`", stage.stage_id)
        })?;
        let stage_kind = BamStage::try_from(stage.stage_id.as_str())
            .with_context(|| format!("resolve BAM stage `{}`", stage.stage_id))?;
        let criticality = stage_criticality(stage.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing stage criticality for `{}`", stage.stage_id)
        })?;
        let admissions = stage_admissions.get(stage.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing admitted benchmark tools for `{}`", stage.stage_id)
        })?;
        let preferred_default_tool_id = default_tool_for_stage(stage_kind).to_string();
        let (anchor_tool_id, anchor_support_status) =
            select_anchor_tool(admissions, preferred_default_tool_id.as_str())?;
        let report_contract = bam_scientific_report_contract_for_stage(stage.stage_id.as_str());
        let canonical_stage_rank = canonical_rank_by_stage
            .get(stage.stage_id.as_str())
            .copied()
            .ok_or_else(|| anyhow!("BAM canonical stage order is missing `{}`", stage.stage_id))?;

        rows.insert(
            stage.stage_id.clone(),
            BamReportStageMetadata {
                canonical_stage_rank,
                readiness_kind: readiness_kind_label(stage.readiness_kind).to_string(),
                criticality: criticality_label(criticality).to_string(),
                anchor_tool_id: anchor_tool_id.to_string(),
                anchor_support_status: anchor_support_status.to_string(),
                report_section_id: placement.section_id.to_string(),
                report_section_title: placement.section_title.to_string(),
                summary_table_id: placement.summary_table_id.to_string(),
                summary_table_title: placement.summary_table_title.to_string(),
                workflow_branch_id: branch_by_stage.get(stage.stage_id.as_str()).cloned(),
                scientific_context_required: scientific_context_required(report_contract.as_ref()),
                report_focus: placement.report_focus.to_string(),
            },
        );
    }

    Ok(rows)
}

fn placement_for_stage(stage_id: &str) -> Option<BamReportPlacement> {
    match stage_id {
        "bam.align" | "bam.validate" | "bam.qc_pre" | "bam.mapping_summary" => {
            Some(BamReportPlacement {
                section_id: "alignment_intake",
                section_title: "Alignment Intake",
                summary_table_id: "alignment_baseline",
                summary_table_title: "Alignment Baseline",
                report_focus: "alignment provenance, validation status, and intake QC baselines",
            })
        }
        "bam.filter" | "bam.mapq_filter" | "bam.length_filter" | "bam.overlap_correction" => {
            Some(BamReportPlacement {
                section_id: "alignment_refinement",
                section_title: "Alignment Refinement",
                summary_table_id: "filter_retention",
                summary_table_title: "Filter and Retention",
                report_focus:
                    "alignment cleanup, overlap correction, and retained-read policy effects",
            })
        }
        "bam.markdup" | "bam.duplication_metrics" | "bam.complexity" => Some(BamReportPlacement {
            section_id: "library_complexity",
            section_title: "Library Complexity",
            summary_table_id: "duplicate_complexity",
            summary_table_title: "Duplicate and Complexity",
            report_focus: "duplicate handling, library saturation, and complexity evidence",
        }),
        "bam.coverage" | "bam.insert_size" | "bam.gc_bias" | "bam.endogenous_content" => {
            Some(BamReportPlacement {
                section_id: "coverage_quality",
                section_title: "Coverage and Quality",
                summary_table_id: "coverage_bias_qc",
                summary_table_title: "Coverage, Bias, and QC",
                report_focus:
                    "coverage regime, insert-size behavior, GC bias, and endogenous-signal reporting",
            })
        }
        "bam.damage" | "bam.authenticity" | "bam.contamination" => Some(BamReportPlacement {
            section_id: "ancient_signal",
            section_title: "Ancient Signal",
            summary_table_id: "damage_authenticity",
            summary_table_title: "Damage and Authenticity",
            report_focus: "damage evidence, authenticity advisories, and contamination guardrails",
        }),
        "bam.sex" | "bam.haplogroups" | "bam.kinship" => Some(BamReportPlacement {
            section_id: "sample_identity",
            section_title: "Sample Identity",
            summary_table_id: "identity_inference",
            summary_table_title: "Identity and Relatedness",
            report_focus: "sex inference, haplogroup context, and relatedness interpretation",
        }),
        "bam.bias_mitigation" | "bam.recalibration" | "bam.genotyping" => {
            Some(BamReportPlacement {
                section_id: "downstream_readiness",
                section_title: "Downstream Readiness",
                summary_table_id: "variant_readiness",
                summary_table_title: "Variant and Bias Readiness",
                report_focus:
                    "bias control, recalibration, and genotyping readiness before downstream inference",
            })
        }
        _ => None,
    }
}

fn branch_id_by_stage() -> BTreeMap<String, String> {
    let mut by_stage = BTreeMap::new();
    for (branch_id, stage_ids) in optional_branches() {
        for stage_id in *stage_ids {
            by_stage.insert((*stage_id).to_string(), (*branch_id).to_string());
        }
    }
    by_stage
}

fn scientific_context_required(
    report_contract: Option<&BamScientificReportContractV1>,
) -> Vec<String> {
    report_contract
        .map(|contract| contract.required_population_or_reference_context.clone())
        .unwrap_or_default()
}

fn ensure_bam_report_map_contract(
    rows: &[BamReportMapRow],
    expected_rows: &[ExpectedBenchmarkResultRow],
) -> Result<()> {
    let report_result_ids =
        rows.iter().map(|row| row.result_row_id.as_str()).collect::<BTreeSet<_>>();
    if report_result_ids.len() != rows.len() {
        return Err(anyhow!("BAM report map must keep one row per expected BAM result row"));
    }

    let expected_result_ids =
        expected_rows.iter().map(|row| row.result_row_id.as_str()).collect::<BTreeSet<_>>();
    if report_result_ids != expected_result_ids {
        return Err(anyhow!(
            "BAM report map must retain every BAM expected-result row; expected {} rows and found {} report rows",
            expected_rows.len(),
            rows.len()
        ));
    }

    let report_keys = rows.iter().map(report_result_key).collect::<BTreeSet<_>>();
    let expected_keys = expected_rows.iter().map(expected_result_key).collect::<BTreeSet<_>>();
    if report_keys != expected_keys {
        return Err(anyhow!(
            "BAM report map drifted from the governed BAM expected-result identity slice"
        ));
    }

    if rows.len() != 49 {
        return Err(anyhow!(
            "BAM report map must contain 49 benchmark-ready result rows, found {}",
            rows.len()
        ));
    }
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    if stage_count != 24 {
        return Err(anyhow!(
            "BAM report map must retain 24 benchmark-ready stages, found {stage_count}"
        ));
    }

    for row in rows {
        if row.result_row_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_family_id.trim().is_empty()
            || row.fixture_id.trim().is_empty()
            || row.sample_scope.trim().is_empty()
            || row.support_status.trim().is_empty()
            || row.anchor_tool_id.trim().is_empty()
            || row.anchor_support_status.trim().is_empty()
            || row.report_section_id.trim().is_empty()
            || row.summary_table_id.trim().is_empty()
        {
            return Err(anyhow!(
                "BAM report map row `{}` / `{}` is missing required report binding fields",
                row.stage_id,
                row.tool_id
            ));
        }
    }

    let section_count =
        rows.iter().map(|row| row.report_section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.as_str()).collect::<BTreeSet<_>>().len();
    if section_count != 7 || summary_table_count != 7 {
        return Err(anyhow!(
            "BAM report map must retain 7 sections and 7 summary tables, found {section_count} sections and {summary_table_count} tables"
        ));
    }

    require_row_mapping(
        rows,
        "bam.align",
        "bwa",
        "alignment_intake",
        "alignment_baseline",
        "optional",
        "supported",
        "bwa",
        "supported",
        None,
    )?;
    require_row_mapping(
        rows,
        "bam.damage",
        "mapdamage2",
        "ancient_signal",
        "damage_authenticity",
        "essential",
        "supported",
        "mapdamage2",
        "supported",
        Some("ancient_dna_authenticity"),
    )?;
    require_row_mapping(
        rows,
        "bam.contamination",
        "schmutzi",
        "ancient_signal",
        "damage_authenticity",
        "essential",
        "supported",
        "schmutzi",
        "supported",
        Some("ancient_dna_authenticity"),
    )?;
    require_row_mapping(
        rows,
        "bam.sex",
        "rxy",
        "sample_identity",
        "identity_inference",
        "essential",
        "supported",
        "rxy",
        "supported",
        Some("sample_identity"),
    )?;
    require_row_mapping(
        rows,
        "bam.recalibration",
        "gatk",
        "downstream_readiness",
        "variant_readiness",
        "essential",
        "supported",
        "gatk",
        "supported",
        Some("variant_readiness"),
    )?;
    require_row_mapping(
        rows,
        "bam.kinship",
        "king",
        "sample_identity",
        "identity_inference",
        "essential",
        "supported",
        "king",
        "supported",
        Some("sample_identity"),
    )?;

    Ok(())
}

fn report_result_key(row: &BamReportMapRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.result_row_id, row.stage_id, row.tool_id, row.corpus_family_id, row.fixture_id
    )
}

fn expected_result_key(row: &ExpectedBenchmarkResultRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.result_row_id, row.stage_id, row.tool_id, row.corpus_family_id, row.fixture_id
    )
}

#[allow(clippy::too_many_arguments)]
fn require_row_mapping(
    rows: &[BamReportMapRow],
    stage_id: &str,
    tool_id: &str,
    section_id: &str,
    summary_table_id: &str,
    criticality: &str,
    support_status: &str,
    anchor_tool_id: &str,
    anchor_support_status: &str,
    workflow_branch_id: Option<&str>,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| anyhow!("BAM report map is missing `{stage_id}` / `{tool_id}`"))?;
    if row.report_section_id != section_id
        || row.summary_table_id != summary_table_id
        || row.criticality != criticality
        || row.support_status != support_status
        || row.anchor_tool_id != anchor_tool_id
        || row.anchor_support_status != anchor_support_status
        || row.workflow_branch_id.as_deref() != workflow_branch_id
    {
        return Err(anyhow!(
            "BAM report map row `{stage_id}` / `{tool_id}` drifted from its governed reporting contract"
        ));
    }
    Ok(())
}

fn report_reason(
    row: &ExpectedBenchmarkResultRow,
    report_section_id: &str,
    summary_table_id: &str,
    report_focus: &str,
    support_status: &str,
    anchor_tool_id: &str,
    anchor_support_status: &str,
    workflow_branch_id: Option<&str>,
    scientific_context_required: &[String],
) -> String {
    let branch_note = workflow_branch_id
        .map(|branch_id| format!("; workflow branch `{branch_id}`"))
        .unwrap_or_default();
    let scientific_note = if scientific_context_required.is_empty() {
        String::new()
    } else {
        format!("; scientific context requires {}", scientific_context_required.join(", "))
    };
    format!(
        "benchmark result binding `{}` / `{}` on fixture `{}` keeps BAM tool support status `{support_status}` while stage anchor tool `{anchor_tool_id}` remains `{anchor_support_status}`; benchmark reporting places it in section `{report_section_id}` and summary table `{summary_table_id}` to track {report_focus}{branch_note}{scientific_note}",
        row.stage_id, row.tool_id, row.fixture_id
    )
}

fn render_bam_report_map_tsv(rows: &[BamReportMapRow]) -> String {
    let mut rendered = String::from(
        "result_row_id\tstage_id\ttool_id\tcorpus_family_id\tfixture_id\tsample_scope\tcanonical_stage_rank\treadiness_kind\tcriticality\tsupport_status\tanchor_tool_id\tanchor_support_status\treport_section_id\treport_section_title\tsummary_table_id\tsummary_table_title\tworkflow_branch_id\tscientific_context_required\treport_focus\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.result_row_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_scope),
            row.canonical_stage_rank,
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.criticality),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.anchor_tool_id),
            sanitize_tsv(&row.anchor_support_status),
            sanitize_tsv(&row.report_section_id),
            sanitize_tsv(&row.report_section_title),
            sanitize_tsv(&row.summary_table_id),
            sanitize_tsv(&row.summary_table_title),
            sanitize_tsv(row.workflow_branch_id.as_deref().unwrap_or("")),
            sanitize_tsv(&row.scientific_context_required.join(",")),
            sanitize_tsv(&row.report_focus),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn admission_support_status<'a>(
    admissions: &'a [super::catalog::ReadinessStageAdmission],
    tool_id: &str,
) -> Result<&'a str> {
    admissions
        .iter()
        .find(|admission| admission.tool_id == tool_id)
        .map(|admission| admission.support_status.as_str())
        .ok_or_else(|| anyhow!("BAM report map stage is missing support status for `{tool_id}`"))
}

fn select_anchor_tool<'a>(
    admissions: &'a [super::catalog::ReadinessStageAdmission],
    preferred_tool_id: &str,
) -> Result<(&'a str, &'a str)> {
    if let Some(admission) =
        admissions.iter().find(|admission| admission.tool_id == preferred_tool_id)
    {
        return Ok((admission.tool_id.as_str(), admission.support_status.as_str()));
    }

    let admission = admissions
        .iter()
        .min_by(|left, right| {
            admission_support_rank(&left.support_status)
                .cmp(&admission_support_rank(&right.support_status))
                .then_with(|| left.tool_id.cmp(&right.tool_id))
        })
        .ok_or_else(|| anyhow!("BAM report map stage has no admitted benchmark tools"))?;
    Ok((admission.tool_id.as_str(), admission.support_status.as_str()))
}

fn admission_support_rank(status: &str) -> usize {
    match status {
        "supported" => 0,
        "planned" => 1,
        _ => 2,
    }
}

fn readiness_kind_label(kind: LocalStageReadinessKind) -> &'static str {
    match kind {
        LocalStageReadinessKind::DryRun => "dry_run",
        LocalStageReadinessKind::Smoke => "smoke",
        LocalStageReadinessKind::DryOrSmoke => "dry_or_smoke",
    }
}

fn criticality_label(criticality: StageCriticality) -> &'static str {
    match criticality {
        StageCriticality::Essential => "essential",
        StageCriticality::Optional => "optional",
        StageCriticality::Experimental => "experimental",
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_report_map, BAM_REPORT_MAP_SCHEMA_VERSION, DEFAULT_BAM_REPORT_MAP_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn bam_report_map_tracks_governed_result_bindings() {
        let repo_root = repo_root();
        let report = render_bam_report_map(&repo_root, PathBuf::from(DEFAULT_BAM_REPORT_MAP_PATH))
            .expect("render BAM report map");

        assert_eq!(report.schema_version, BAM_REPORT_MAP_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_REPORT_MAP_PATH);
        assert_eq!(report.domain, "bam");
        assert_eq!(report.expected_result_row_count, 49);
        assert_eq!(report.row_count, 49);
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.tool_count, 25);
        assert_eq!(report.section_count, 7);
        assert_eq!(report.summary_table_count, 7);

        let align = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.align" && row.tool_id == "bwa")
            .expect("align row");
        assert_eq!(align.fixture_id, "corpus-01-mini");
        assert_eq!(align.support_status, "supported");
        assert_eq!(align.report_section_id, "alignment_intake");
        assert_eq!(align.summary_table_id, "alignment_baseline");
        assert_eq!(align.anchor_tool_id, "bwa");

        let contamination = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.contamination" && row.tool_id == "schmutzi")
            .expect("contamination row");
        assert_eq!(contamination.anchor_tool_id, "schmutzi");
        assert_eq!(contamination.support_status, "supported");
        assert_eq!(contamination.workflow_branch_id.as_deref(), Some("ancient_dna_authenticity"));
        assert_eq!(contamination.report_section_id, "ancient_signal");
        assert_eq!(contamination.summary_table_id, "damage_authenticity");

        let sex = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.sex" && row.tool_id == "rxy")
            .expect("sex row");
        assert_eq!(sex.anchor_tool_id, "rxy");
        assert_eq!(
            sex.scientific_context_required,
            vec!["chromosome_system".to_string(), "minimum_y_sites".to_string()]
        );
        assert_eq!(sex.report_section_id, "sample_identity");

        let recalibration = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.recalibration" && row.tool_id == "gatk")
            .expect("recalibration row");
        assert_eq!(recalibration.criticality, "essential");
        assert_eq!(recalibration.support_status, "supported");
        assert_eq!(recalibration.report_section_id, "downstream_readiness");
        assert_eq!(recalibration.summary_table_id, "variant_readiness");
    }
}
