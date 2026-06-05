use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::pipeline_contract::{optional_branches, stage_criticality, StageCriticality};
use bijux_dna_domain_bam::{
    bam_scientific_report_contract_for_stage, BamScientificReportContractV1, BamStage,
};
use bijux_dna_planner_bam::stage_api::default_tool_for_stage;
use serde::Serialize;

use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_REPORT_MAP_PATH: &str = "target/bench-readiness/bam-report-map.tsv";
const BAM_REPORT_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_report_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamReportMapRow {
    pub(crate) stage_id: String,
    pub(crate) canonical_stage_rank: usize,
    pub(crate) readiness_kind: String,
    pub(crate) criticality: String,
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
    pub(crate) stage_count: usize,
    pub(crate) section_count: usize,
    pub(crate) summary_table_count: usize,
    pub(crate) section_counts: BTreeMap<String, usize>,
    pub(crate) branch_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamReportMapRow>,
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
    let rows = collect_bam_report_map_rows(repo_root)?;
    let section_count =
        rows.iter().map(|row| row.report_section_id.clone()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.clone()).collect::<BTreeSet<_>>().len();
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
        stage_count: rows.len(),
        section_count,
        summary_table_count,
        section_counts,
        branch_counts,
        rows,
    })
}

pub(crate) fn collect_bam_report_map_rows(repo_root: &Path) -> Result<Vec<BamReportMapRow>> {
    let inventory = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?;
    let stage_admissions =
        super::catalog::load_stage_admissions(repo_root, super::catalog::ReadinessDomain::Bam)?;
    let canonical_rank_by_stage = BamStage::all()
        .iter()
        .enumerate()
        .map(|(idx, stage)| (stage.as_str().to_string(), idx + 1))
        .collect::<BTreeMap<_, _>>();
    let branch_by_stage = branch_id_by_stage();

    let mut rows = Vec::with_capacity(inventory.stages.len());
    for stage in inventory.stages {
        let placement = placement_for_stage(stage.stage_id.as_str())
            .ok_or_else(|| anyhow!("BAM report map is missing stage placement for `{}`", stage.stage_id))?;
        let stage_kind = BamStage::try_from(stage.stage_id.as_str())
            .with_context(|| format!("resolve BAM stage `{}`", stage.stage_id))?;
        let criticality = stage_criticality(stage.stage_id.as_str())
            .ok_or_else(|| anyhow!("BAM report map is missing stage criticality for `{}`", stage.stage_id))?;
        let admissions = stage_admissions.get(stage.stage_id.as_str()).ok_or_else(|| {
            anyhow!("BAM report map is missing admitted benchmark tools for `{}`", stage.stage_id)
        })?;
        let preferred_default_tool_id = default_tool_for_stage(stage_kind).to_string();
        let (anchor_tool_id, anchor_support_status) =
            select_anchor_tool(admissions, preferred_default_tool_id.as_str())?;
        let report_contract = bam_scientific_report_contract_for_stage(stage.stage_id.as_str());
        let scientific_context_required = scientific_context_required(report_contract.as_ref());
        let workflow_branch_id = branch_by_stage.get(stage.stage_id.as_str()).cloned();
        let canonical_stage_rank =
            canonical_rank_by_stage.get(stage.stage_id.as_str()).copied().ok_or_else(|| {
                anyhow!("BAM canonical stage order is missing `{}`", stage.stage_id)
            })?;

        rows.push(BamReportMapRow {
            stage_id: stage.stage_id.clone(),
            canonical_stage_rank,
            readiness_kind: readiness_kind_label(stage.readiness_kind).to_string(),
            criticality: criticality_label(criticality).to_string(),
            anchor_tool_id: anchor_tool_id.to_string(),
            anchor_support_status: anchor_support_status.to_string(),
            report_section_id: placement.section_id.to_string(),
            report_section_title: placement.section_title.to_string(),
            summary_table_id: placement.summary_table_id.to_string(),
            summary_table_title: placement.summary_table_title.to_string(),
            workflow_branch_id,
            scientific_context_required,
            report_focus: placement.report_focus.to_string(),
            reason: report_reason(
                stage.stage_id.as_str(),
                placement,
                criticality,
                anchor_tool_id,
                anchor_support_status,
                report_contract.as_ref(),
            ),
        });
    }

    rows.sort_by(|left, right| left.canonical_stage_rank.cmp(&right.canonical_stage_rank));
    ensure_bam_report_map_contract(&rows)?;
    Ok(rows)
}

fn placement_for_stage(stage_id: &str) -> Option<BamReportPlacement> {
    match stage_id {
        "bam.align" | "bam.validate" | "bam.qc_pre" | "bam.mapping_summary" => Some(BamReportPlacement {
            section_id: "alignment_intake",
            section_title: "Alignment Intake",
            summary_table_id: "alignment_baseline",
            summary_table_title: "Alignment Baseline",
            report_focus: "alignment provenance, validation status, and intake QC baselines",
        }),
        "bam.filter" | "bam.mapq_filter" | "bam.length_filter" | "bam.overlap_correction" => {
            Some(BamReportPlacement {
                section_id: "alignment_refinement",
                section_title: "Alignment Refinement",
                summary_table_id: "filter_retention",
                summary_table_title: "Filter and Retention",
                report_focus: "alignment cleanup, overlap correction, and retained-read policy effects",
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
                report_focus: "coverage regime, insert-size behavior, GC bias, and endogenous-signal reporting",
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
        "bam.bias_mitigation" | "bam.recalibration" | "bam.genotyping" => Some(BamReportPlacement {
            section_id: "downstream_readiness",
            section_title: "Downstream Readiness",
            summary_table_id: "variant_readiness",
            summary_table_title: "Variant and Bias Readiness",
            report_focus: "bias control, recalibration, and genotyping readiness before downstream inference",
        }),
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

fn ensure_bam_report_map_contract(rows: &[BamReportMapRow]) -> Result<()> {
    if rows.len() != 24 {
        return Err(anyhow!(
            "BAM report map must contain 24 benchmark-ready stages, found {}",
            rows.len()
        ));
    }

    let expected_rows = [
        (
            "bam.align",
            "alignment_intake",
            "alignment_baseline",
            "optional",
            "bwa",
            "supported",
            None,
        ),
        (
            "bam.damage",
            "ancient_signal",
            "damage_authenticity",
            "essential",
            "mapdamage2",
            "supported",
            Some("ancient_dna_authenticity"),
        ),
        (
            "bam.contamination",
            "ancient_signal",
            "damage_authenticity",
            "essential",
            "schmutzi",
            "supported",
            Some("ancient_dna_authenticity"),
        ),
        (
            "bam.sex",
            "sample_identity",
            "identity_inference",
            "essential",
            "rxy",
            "supported",
            Some("sample_identity"),
        ),
        (
            "bam.recalibration",
            "downstream_readiness",
            "variant_readiness",
            "essential",
            "gatk",
            "supported",
            Some("variant_readiness"),
        ),
        (
            "bam.kinship",
            "sample_identity",
            "identity_inference",
            "essential",
            "king",
            "supported",
            Some("sample_identity"),
        ),
    ];

    for (
        stage_id,
        section_id,
        summary_table_id,
        criticality,
        anchor_tool_id,
        anchor_support_status,
        workflow_branch_id,
    ) in expected_rows
    {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id)
            .ok_or_else(|| anyhow!("BAM report map is missing stage `{stage_id}`"))?;
        if row.report_section_id != section_id
            || row.summary_table_id != summary_table_id
            || row.criticality != criticality
            || row.anchor_tool_id != anchor_tool_id
            || row.anchor_support_status != anchor_support_status
            || row.workflow_branch_id.as_deref() != workflow_branch_id
        {
            return Err(anyhow!(
                "BAM report map stage `{stage_id}` must retain its governed report placement and anchor tool contract; found section=`{}` table=`{}` criticality=`{}` anchor_tool=`{}` anchor_status=`{}` branch=`{}`",
                row.report_section_id,
                row.summary_table_id,
                row.criticality,
                row.anchor_tool_id,
                row.anchor_support_status,
                row.workflow_branch_id.as_deref().unwrap_or("")
            ));
        }
    }

    let section_count = rows
        .iter()
        .map(|row| row.report_section_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let summary_table_count = rows
        .iter()
        .map(|row| row.summary_table_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    if section_count != 7 || summary_table_count != 7 {
        return Err(anyhow!(
            "BAM report map must retain 7 sections and 7 summary tables, found {section_count} sections and {summary_table_count} tables"
        ));
    }

    Ok(())
}

fn report_reason(
    stage_id: &str,
    placement: BamReportPlacement,
    criticality: StageCriticality,
    anchor_tool_id: &str,
    anchor_support_status: &str,
    report_contract: Option<&BamScientificReportContractV1>,
) -> String {
    let contract_note = report_contract
        .map(|contract| {
            format!(
                "; scientific context requires {}",
                contract.required_population_or_reference_context.join(", ")
            )
        })
        .unwrap_or_default();
    format!(
        "stage `{stage_id}` is a {} BAM benchmark stage with anchor tool `{anchor_tool_id}` ({anchor_support_status}); benchmark reporting places it in section `{}` and summary table `{}` to track {}{}",
        criticality_label(criticality),
        placement.section_id,
        placement.summary_table_id,
        placement.report_focus,
        contract_note,
    )
}

fn render_bam_report_map_tsv(rows: &[BamReportMapRow]) -> String {
    let mut rendered = String::from(
        "stage_id\tcanonical_stage_rank\treadiness_kind\tcriticality\tanchor_tool_id\tanchor_support_status\treport_section_id\treport_section_title\tsummary_table_id\tsummary_table_title\tworkflow_branch_id\tscientific_context_required\treport_focus\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            row.canonical_stage_rank,
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.criticality),
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

fn select_anchor_tool<'a>(
    admissions: &'a [super::catalog::ReadinessStageAdmission],
    preferred_tool_id: &str,
) -> Result<(&'a str, &'a str)> {
    if let Some(admission) = admissions.iter().find(|admission| admission.tool_id == preferred_tool_id) {
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

    use super::{render_bam_report_map, BAM_REPORT_MAP_SCHEMA_VERSION, DEFAULT_BAM_REPORT_MAP_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn bam_report_map_tracks_governed_stage_sections() {
        let repo_root = repo_root();
        let report =
            render_bam_report_map(&repo_root, PathBuf::from(DEFAULT_BAM_REPORT_MAP_PATH))
                .expect("render BAM report map");

        assert_eq!(report.schema_version, BAM_REPORT_MAP_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_REPORT_MAP_PATH);
        assert_eq!(report.domain, "bam");
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.section_count, 7);
        assert_eq!(report.summary_table_count, 7);
        assert_eq!(report.section_counts.get("alignment_intake"), Some(&4));
        assert_eq!(report.section_counts.get("alignment_refinement"), Some(&4));
        assert_eq!(report.section_counts.get("sample_identity"), Some(&3));

        let contamination = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.contamination")
            .expect("contamination row");
        assert_eq!(contamination.anchor_tool_id, "schmutzi");
        assert_eq!(contamination.workflow_branch_id.as_deref(), Some("ancient_dna_authenticity"));
        assert_eq!(contamination.report_section_id, "ancient_signal");
        assert_eq!(contamination.summary_table_id, "damage_authenticity");

        let sex = report.rows.iter().find(|row| row.stage_id == "bam.sex").expect("sex row");
        assert_eq!(sex.anchor_tool_id, "rxy");
        assert_eq!(
            sex.scientific_context_required,
            vec!["chromosome_system".to_string(), "minimum_y_sites".to_string()]
        );
        assert_eq!(sex.report_section_id, "sample_identity");

        let recalibration = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.recalibration")
            .expect("recalibration row");
        assert_eq!(recalibration.criticality, "essential");
        assert_eq!(recalibration.report_section_id, "downstream_readiness");
        assert_eq!(recalibration.summary_table_id, "variant_readiness");
    }
}
