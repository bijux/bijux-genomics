use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::stage_semantics::{stage_kind, stage_semantics, FastqStageKind};
use bijux_dna_domain_fastq::stages::semantics::STAGES;
use bijux_dna_domain_fastq::{
    default_execution_tool_for_stage, stage_criticality, stage_metric_classes, StageCriticality,
};
use serde::Serialize;

use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_REPORT_MAP_PATH: &str = "benchmarks/readiness/fastq-report-map.tsv";
const FASTQ_REPORT_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.fastq_report_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqReportMapRow {
    pub(crate) stage_id: String,
    pub(crate) canonical_stage_rank: usize,
    pub(crate) readiness_kind: String,
    pub(crate) stage_kind: String,
    pub(crate) criticality: String,
    pub(crate) anchor_tool_id: String,
    pub(crate) anchor_support_status: String,
    pub(crate) report_section_id: String,
    pub(crate) report_section_title: String,
    pub(crate) summary_table_id: String,
    pub(crate) summary_table_title: String,
    pub(crate) metric_classes: Vec<String>,
    pub(crate) mutates_fastq: bool,
    pub(crate) produces_reports_only: bool,
    pub(crate) report_focus: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqReportMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) section_count: usize,
    pub(crate) summary_table_count: usize,
    pub(crate) section_counts: BTreeMap<String, usize>,
    pub(crate) stage_kind_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FastqReportMapRow>,
}

#[derive(Debug, Clone, Copy)]
struct FastqReportPlacement {
    section_id: &'static str,
    section_title: &'static str,
    summary_table_id: &'static str,
    summary_table_title: &'static str,
    report_focus: &'static str,
}

pub(crate) fn run_render_fastq_report_map(
    args: &parse::BenchReadinessRenderFastqReportMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_report_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_REPORT_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_report_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqReportMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_fastq_report_map_rows(repo_root)?;
    let section_count =
        rows.iter().map(|row| row.report_section_id.clone()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.clone()).collect::<BTreeSet<_>>().len();
    let mut section_counts = BTreeMap::<String, usize>::new();
    let mut stage_kind_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *section_counts.entry(row.report_section_id.clone()).or_default() += 1;
        *stage_kind_counts.entry(row.stage_kind.clone()).or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_report_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(FastqReportMapReport {
        schema_version: FASTQ_REPORT_MAP_SCHEMA_VERSION,
        domain: "fastq",
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: rows.len(),
        section_count,
        summary_table_count,
        section_counts,
        stage_kind_counts,
        rows,
    })
}

pub(crate) fn collect_fastq_report_map_rows(repo_root: &Path) -> Result<Vec<FastqReportMapRow>> {
    let inventory = load_local_stage_inventory(repo_root, BenchLocalDomain::Fastq)?;
    let stage_admissions =
        super::catalog::load_stage_admissions(repo_root, super::catalog::ReadinessDomain::Fastq)?;
    let canonical_rank_by_stage = STAGES
        .iter()
        .enumerate()
        .map(|(idx, stage)| (stage.stage_id.as_str().to_string(), idx + 1))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(inventory.stages.len());
    for stage in inventory.stages {
        let stage_id = StageId::new(stage.stage_id.clone());
        let placement = placement_for_stage(stage_id.as_str()).ok_or_else(|| {
            anyhow!("FASTQ report map is missing stage placement for `{}`", stage_id)
        })?;
        let semantics = stage_semantics(&stage_id).ok_or_else(|| {
            anyhow!("FASTQ report map is missing stage semantics for `{}`", stage_id)
        })?;
        let stage_kind = stage_kind(&stage_id)
            .ok_or_else(|| anyhow!("FASTQ report map is missing stage kind for `{}`", stage_id))?;
        let criticality = stage_criticality(&stage_id)
            .ok_or_else(|| anyhow!("FASTQ report map is missing criticality for `{}`", stage_id))?;
        let admissions = stage_admissions.get(stage_id.as_str()).ok_or_else(|| {
            anyhow!("FASTQ report map is missing admitted benchmark tools for `{}`", stage_id)
        })?;
        let preferred_default_tool_id =
            default_execution_tool_for_stage(&stage_id).map(|tool_id| tool_id.to_string());
        let (anchor_tool_id, anchor_support_status) =
            select_anchor_tool(admissions, preferred_default_tool_id.as_deref())?;
        let metric_classes = stage_metric_classes(&stage_id)
            .unwrap_or(&[])
            .iter()
            .map(metric_class_label)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let canonical_stage_rank = canonical_rank_by_stage
            .get(stage_id.as_str())
            .copied()
            .ok_or_else(|| anyhow!("FASTQ canonical stage order is missing `{}`", stage_id))?;

        rows.push(FastqReportMapRow {
            stage_id: stage_id.to_string(),
            canonical_stage_rank,
            readiness_kind: readiness_kind_label(stage.readiness_kind).to_string(),
            stage_kind: stage_kind_label(stage_kind).to_string(),
            criticality: criticality_label(criticality).to_string(),
            anchor_tool_id: anchor_tool_id.to_string(),
            anchor_support_status: anchor_support_status.to_string(),
            report_section_id: placement.section_id.to_string(),
            report_section_title: placement.section_title.to_string(),
            summary_table_id: placement.summary_table_id.to_string(),
            summary_table_title: placement.summary_table_title.to_string(),
            metric_classes,
            mutates_fastq: semantics.mutates_fastq,
            produces_reports_only: semantics.produces_reports_only,
            report_focus: placement.report_focus.to_string(),
            reason: report_reason(
                stage_id.as_str(),
                placement,
                stage_kind,
                criticality,
                anchor_tool_id,
                anchor_support_status,
                semantics.produces_reports_only,
            ),
        });
    }

    rows.sort_by(|left, right| left.canonical_stage_rank.cmp(&right.canonical_stage_rank));
    ensure_fastq_report_map_contract(&rows)?;
    Ok(rows)
}

fn placement_for_stage(stage_id: &str) -> Option<FastqReportPlacement> {
    match stage_id {
        "fastq.index_reference" => Some(FastqReportPlacement {
            section_id: "reference_preparation",
            section_title: "Reference Preparation",
            summary_table_id: "reference_index_assets",
            summary_table_title: "Reference Index Assets",
            report_focus: "reference provenance and benchmark index availability",
        }),
        "fastq.validate_reads" => Some(FastqReportPlacement {
            section_id: "input_readiness",
            section_title: "Input Readiness",
            summary_table_id: "validation_intake",
            summary_table_title: "Validation and Intake",
            report_focus: "input validation, manifest integrity, and governed read admission",
        }),
        "fastq.profile_read_lengths"
        | "fastq.detect_adapters"
        | "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.report_qc" => Some(FastqReportPlacement {
            section_id: "quality_profiling",
            section_title: "Quality Profiling",
            summary_table_id: "qc_signal_profiles",
            summary_table_title: "QC Signal Profiles",
            report_focus: "baseline read quality, composition, and aggregated QC evidence",
        }),
        "fastq.detect_duplicates_premerge" | "fastq.estimate_library_complexity_prealign" => {
            Some(FastqReportPlacement {
                section_id: "quality_profiling",
                section_title: "Quality Profiling",
                summary_table_id: "premerge_complexity",
                summary_table_title: "Pre-merge Complexity",
                report_focus: "duplicate burden and pre-alignment library complexity evidence",
            })
        }
        "fastq.trim_terminal_damage"
        | "fastq.trim_polyg_tails"
        | "fastq.trim_reads"
        | "fastq.filter_reads"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.filter_low_complexity"
        | "fastq.correct_errors"
        | "fastq.extract_umis" => Some(FastqReportPlacement {
            section_id: "read_cleanup",
            section_title: "Read Cleanup",
            summary_table_id: "cleanup_retention",
            summary_table_title: "Cleanup and Retention",
            report_focus: "sequence-retention deltas, cleanup transforms, and mutation provenance",
        }),
        "fastq.deplete_rrna"
        | "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.screen_taxonomy" => Some(FastqReportPlacement {
            section_id: "contamination_screening",
            section_title: "Contamination Screening",
            summary_table_id: "screening_contamination",
            summary_table_title: "Screening and Contamination",
            report_focus:
                "reference-guided depletion, contamination screening, and taxonomy evidence",
        }),
        "fastq.normalize_primers"
        | "fastq.remove_chimeras"
        | "fastq.infer_asvs"
        | "fastq.cluster_otus"
        | "fastq.normalize_abundance" => Some(FastqReportPlacement {
            section_id: "amplicon_interpretation",
            section_title: "Amplicon Interpretation",
            summary_table_id: "amplicon_features",
            summary_table_title: "Amplicon Feature Tables",
            report_focus:
                "amplicon cleanup, feature inference, and abundance normalization outputs",
        }),
        _ => None,
    }
}

fn ensure_fastq_report_map_contract(rows: &[FastqReportMapRow]) -> Result<()> {
    if rows.len() != 27 {
        return Err(anyhow!(
            "FASTQ report map must contain 27 benchmark-ready stages, found {}",
            rows.len()
        ));
    }

    let expected_rows = [
        (
            "fastq.index_reference",
            "reference_preparation",
            "reference_index_assets",
            "meta",
            "optional",
            "bowtie2_build",
            "supported",
        ),
        (
            "fastq.validate_reads",
            "input_readiness",
            "validation_intake",
            "core",
            "essential",
            "fastqvalidator",
            "supported",
        ),
        (
            "fastq.report_qc",
            "quality_profiling",
            "qc_signal_profiles",
            "optional",
            "essential",
            "multiqc",
            "supported",
        ),
        (
            "fastq.estimate_library_complexity_prealign",
            "quality_profiling",
            "premerge_complexity",
            "optional",
            "optional",
            "bijux_dna",
            "planned",
        ),
        (
            "fastq.trim_reads",
            "read_cleanup",
            "cleanup_retention",
            "core",
            "essential",
            "fastp",
            "supported",
        ),
        (
            "fastq.screen_taxonomy",
            "contamination_screening",
            "screening_contamination",
            "optional",
            "optional",
            "kraken2",
            "supported",
        ),
        (
            "fastq.infer_asvs",
            "amplicon_interpretation",
            "amplicon_features",
            "amplicon",
            "experimental",
            "dada2",
            "supported",
        ),
    ];

    for (
        stage_id,
        section_id,
        summary_table_id,
        stage_kind,
        criticality,
        anchor_tool_id,
        anchor_support_status,
    ) in expected_rows
    {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id)
            .ok_or_else(|| anyhow!("FASTQ report map is missing stage `{stage_id}`"))?;
        if row.report_section_id != section_id
            || row.summary_table_id != summary_table_id
            || row.stage_kind != stage_kind
            || row.criticality != criticality
            || row.anchor_tool_id != anchor_tool_id
            || row.anchor_support_status != anchor_support_status
        {
            return Err(anyhow!(
                "FASTQ report map stage `{stage_id}` must retain its governed report placement and anchor tool contract"
            ));
        }
    }

    let section_count =
        rows.iter().map(|row| row.report_section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.as_str()).collect::<BTreeSet<_>>().len();
    if section_count != 6 || summary_table_count != 7 {
        return Err(anyhow!(
            "FASTQ report map must retain 6 sections and 7 summary tables, found {section_count} sections and {summary_table_count} tables"
        ));
    }

    Ok(())
}

fn report_reason(
    stage_id: &str,
    placement: FastqReportPlacement,
    stage_kind: FastqStageKind,
    criticality: StageCriticality,
    anchor_tool_id: &str,
    anchor_support_status: &str,
    produces_reports_only: bool,
) -> String {
    let stage_mode = if produces_reports_only { "reports-only" } else { "transforming" };
    format!(
        "stage `{stage_id}` is a {} {} FASTQ stage with benchmark anchor tool `{anchor_tool_id}` ({anchor_support_status}); benchmark reporting places it in section `{}` and summary table `{}` to track {}",
        criticality_label(criticality),
        stage_kind_label(stage_kind),
        placement.section_id,
        placement.summary_table_id,
        placement.report_focus,
    )
    .replace(" FASTQ stage", &format!(" FASTQ {stage_mode} stage"))
}

fn render_fastq_report_map_tsv(rows: &[FastqReportMapRow]) -> String {
    let mut rendered = String::from(
        "stage_id\tcanonical_stage_rank\treadiness_kind\tstage_kind\tcriticality\tanchor_tool_id\tanchor_support_status\treport_section_id\treport_section_title\tsummary_table_id\tsummary_table_title\tmetric_classes\tmutates_fastq\tproduces_reports_only\treport_focus\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            row.canonical_stage_rank,
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.stage_kind),
            sanitize_tsv(&row.criticality),
            sanitize_tsv(&row.anchor_tool_id),
            sanitize_tsv(&row.anchor_support_status),
            sanitize_tsv(&row.report_section_id),
            sanitize_tsv(&row.report_section_title),
            sanitize_tsv(&row.summary_table_id),
            sanitize_tsv(&row.summary_table_title),
            sanitize_tsv(&row.metric_classes.join(",")),
            row.mutates_fastq,
            row.produces_reports_only,
            sanitize_tsv(&row.report_focus),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn select_anchor_tool<'a>(
    admissions: &'a [super::catalog::ReadinessStageAdmission],
    preferred_tool_id: Option<&str>,
) -> Result<(&'a str, &'a str)> {
    if let Some(preferred_tool_id) = preferred_tool_id {
        if let Some(admission) =
            admissions.iter().find(|admission| admission.tool_id == preferred_tool_id)
        {
            return Ok((admission.tool_id.as_str(), admission.support_status.as_str()));
        }
    }

    let admission = admissions
        .iter()
        .min_by(|left, right| {
            admission_support_rank(&left.support_status)
                .cmp(&admission_support_rank(&right.support_status))
                .then_with(|| left.tool_id.cmp(&right.tool_id))
        })
        .ok_or_else(|| anyhow!("FASTQ report map stage has no admitted benchmark tools"))?;
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

fn stage_kind_label(kind: FastqStageKind) -> &'static str {
    match kind {
        FastqStageKind::Core => "core",
        FastqStageKind::Optional => "optional",
        FastqStageKind::Meta => "meta",
        FastqStageKind::Amplicon => "amplicon",
    }
}

fn criticality_label(criticality: StageCriticality) -> &'static str {
    match criticality {
        StageCriticality::Essential => "essential",
        StageCriticality::Optional => "optional",
        StageCriticality::Experimental => "experimental",
    }
}

fn metric_class_label(
    metric_class: &bijux_dna_domain_fastq::metrics::spec::MetricClass,
) -> &'static str {
    match metric_class {
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Integrity => "integrity",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Retention => "retention",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::QualityShift => "quality_shift",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Contamination => "contamination",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Composition => "composition",
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
        render_fastq_report_map, DEFAULT_FASTQ_REPORT_MAP_PATH, FASTQ_REPORT_MAP_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn fastq_report_map_tracks_governed_stage_sections() {
        let repo_root = repo_root();
        let report =
            render_fastq_report_map(&repo_root, PathBuf::from(DEFAULT_FASTQ_REPORT_MAP_PATH))
                .expect("render FASTQ report map");

        assert_eq!(report.schema_version, FASTQ_REPORT_MAP_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_REPORT_MAP_PATH);
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.section_count, 6);
        assert_eq!(report.summary_table_count, 7);
        assert_eq!(report.section_counts.get("quality_profiling"), Some(&7));
        assert_eq!(report.section_counts.get("read_cleanup"), Some(&9));
        assert_eq!(report.section_counts.get("contamination_screening"), Some(&4));
        assert_eq!(report.section_counts.get("amplicon_interpretation"), Some(&5));

        let screen_taxonomy = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.screen_taxonomy")
            .expect("screen taxonomy row");
        assert_eq!(screen_taxonomy.anchor_tool_id, "kraken2");
        assert_eq!(screen_taxonomy.anchor_support_status, "supported");
        assert_eq!(screen_taxonomy.report_section_id, "contamination_screening");
        assert_eq!(screen_taxonomy.summary_table_id, "screening_contamination");
        assert_eq!(screen_taxonomy.readiness_kind, "dry_or_smoke");

        let report_qc = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.report_qc")
            .expect("report qc row");
        assert_eq!(report_qc.anchor_tool_id, "multiqc");
        assert_eq!(report_qc.anchor_support_status, "supported");
        assert!(report_qc.produces_reports_only);
        assert_eq!(report_qc.report_section_id, "quality_profiling");
        assert_eq!(report_qc.summary_table_id, "qc_signal_profiles");
    }
}
