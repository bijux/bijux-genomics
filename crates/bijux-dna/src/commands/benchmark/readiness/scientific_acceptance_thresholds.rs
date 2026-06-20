use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::{
    stage_comparable_metric_contracts_for_stage, BamScientificInsufficiencyPolicy,
    BamScientificPassDirection, BamScientificToleranceKind,
};
use serde::{Deserialize, Serialize};

use super::bam_comparable_metrics::{
    render_bam_comparable_metrics, BamComparableMetricsReport, DEFAULT_BAM_COMPARABLE_METRICS_PATH,
};
use super::fastq_comparable_metrics::{
    render_fastq_comparable_metrics, FastqComparableMetricsReport,
    DEFAULT_FASTQ_COMPARABLE_METRICS_PATH,
};
use super::vcf_comparable_metrics::{
    render_vcf_comparable_metrics, VcfComparableMetricsReport, VcfComparableMetricsRow,
    DEFAULT_VCF_COMPARABLE_METRICS_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH: &str =
    "configs/bench/local/scientific-acceptance-thresholds.toml";
pub(crate) const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_VERSION: &str =
    "bijux.bench.local_scientific_acceptance_thresholds.v1";
const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.scientific_acceptance_thresholds.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScientificAcceptanceDirection {
    ExactMatchPreferred,
    HigherIsBetter,
    LowerIsBetter,
    Minimum,
    Maximum,
    Range,
    StructuredMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScientificAcceptanceToleranceKind {
    RelativeFraction,
    AbsoluteDelta,
    ExactMatch,
    NormalizedSetOverlap,
    NormalizedRecordOverlap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScientificAcceptancePassRule {
    MustMatchReference,
    MustMeetOrExceedReference,
    MustNotExceedReference,
    MustRemainWithinReferenceRange,
    MustMatchReferenceStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScientificAcceptanceInsufficiencyBehavior {
    RefuseStageComparison,
    DropMetricFromStage,
    WarnAndExcludeStage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScientificAcceptanceThresholdsConfig {
    pub(crate) schema_version: String,
    pub(crate) rows: Vec<ScientificAcceptanceThresholdRow>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScientificAcceptanceThresholdRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) metric_id: String,
    pub(crate) metric_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) unit: Option<String>,
    pub(crate) direction: ScientificAcceptanceDirection,
    pub(crate) tolerance_kind: ScientificAcceptanceToleranceKind,
    pub(crate) tolerance_value: f64,
    pub(crate) pass_rule: ScientificAcceptancePassRule,
    pub(crate) insufficiency_behavior: ScientificAcceptanceInsufficiencyBehavior,
    pub(crate) required: bool,
    pub(crate) declaration_origin: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ScientificAcceptanceThresholdsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) comparable_metric_count: usize,
    pub(crate) row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) direction_counts: BTreeMap<String, usize>,
    pub(crate) insufficiency_behavior_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<ScientificAcceptanceThresholdRow>,
}

pub(crate) fn run_render_scientific_acceptance_thresholds(
    args: &parse::BenchReadinessRenderScientificAcceptanceThresholdsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_scientific_acceptance_thresholds(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn render_scientific_acceptance_thresholds(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ScientificAcceptanceThresholdsReport> {
    let scratch_root = std::env::temp_dir().join(format!(
        "bijux-scientific-acceptance-thresholds-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&scratch_root)
        .with_context(|| format!("create {}", scratch_root.display()))?;

    let fastq_report = render_fastq_comparable_metrics(
        repo_root,
        scratch_root.join(DEFAULT_FASTQ_COMPARABLE_METRICS_PATH),
    )?;
    let bam_report = render_bam_comparable_metrics(
        repo_root,
        scratch_root.join(DEFAULT_BAM_COMPARABLE_METRICS_PATH),
    )?;
    let vcf_report = render_vcf_comparable_metrics(
        repo_root,
        scratch_root.join(DEFAULT_VCF_COMPARABLE_METRICS_PATH),
    )?;
    let render_result = render_scientific_acceptance_thresholds_from_reports(
        repo_root,
        output_path,
        &fastq_report,
        &bam_report,
        &vcf_report,
    );
    let _ = fs::remove_dir_all(&scratch_root);
    render_result
}

pub(crate) fn render_scientific_acceptance_thresholds_from_reports(
    repo_root: &Path,
    output_path: PathBuf,
    fastq_report: &FastqComparableMetricsReport,
    bam_report: &BamComparableMetricsReport,
    vcf_report: &VcfComparableMetricsReport,
) -> Result<ScientificAcceptanceThresholdsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_scientific_acceptance_threshold_rows_from_reports(
        fastq_report,
        bam_report,
        vcf_report,
    )?;
    ensure_unique_rows(&rows)?;

    let config = ScientificAcceptanceThresholdsConfig {
        schema_version: SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_VERSION.to_string(),
        rows: rows.clone(),
    };
    let rendered = toml::to_string_pretty(&config)
        .context("serialize scientific acceptance thresholds config")?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let comparable_metric_count =
        fastq_report.rows.iter().map(|row| row.shared_metric_fields.len()).sum::<usize>()
            + bam_report.rows.iter().map(|row| row.shared_metric_fields.len()).sum::<usize>()
            + vcf_report.row_count;
    if rows.len() != comparable_metric_count {
        return Err(anyhow!(
            "scientific acceptance thresholds cover {} comparable metrics but expected {comparable_metric_count}",
            rows.len()
        ));
    }

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut direction_counts = BTreeMap::<String, usize>::new();
    let mut insufficiency_behavior_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *direction_counts
            .entry(scientific_acceptance_direction_label(row.direction).to_string())
            .or_default() += 1;
        *insufficiency_behavior_counts
            .entry(
                scientific_acceptance_insufficiency_behavior_label(row.insufficiency_behavior)
                    .to_string(),
            )
            .or_default() += 1;
    }

    Ok(ScientificAcceptanceThresholdsReport {
        schema_version: SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, &output_path),
        comparable_metric_count,
        row_count: rows.len(),
        domain_counts,
        direction_counts,
        insufficiency_behavior_counts,
        rows,
    })
}

pub(crate) fn collect_scientific_acceptance_threshold_rows_from_reports(
    fastq_report: &FastqComparableMetricsReport,
    bam_report: &BamComparableMetricsReport,
    vcf_report: &VcfComparableMetricsReport,
) -> Result<Vec<ScientificAcceptanceThresholdRow>> {
    let mut rows = Vec::new();

    for row in &fastq_report.rows {
        for metric_id in &row.shared_metric_fields {
            rows.push(fastq_acceptance_row(&row.stage_id, metric_id)?);
        }
    }

    for row in &bam_report.rows {
        let stage_id = StageId::new(row.stage_id.clone());
        for metric in stage_comparable_metric_contracts_for_stage(&stage_id) {
            let threshold = metric.scientific_threshold.ok_or_else(|| {
                anyhow!(
                    "BAM comparable metric `{}` / `{}` is missing a scientific threshold contract",
                    row.stage_id,
                    metric.name
                )
            })?;
            rows.push(ScientificAcceptanceThresholdRow {
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                metric_id: metric.name.clone(),
                metric_name: metric.name,
                unit: None,
                direction: scientific_acceptance_direction_from_bam(threshold.pass_direction),
                tolerance_kind: scientific_acceptance_tolerance_kind_from_bam(
                    threshold.tolerance_kind,
                ),
                tolerance_value: threshold.tolerance_value,
                pass_rule: scientific_acceptance_pass_rule_from_bam(threshold.pass_direction),
                insufficiency_behavior: scientific_acceptance_insufficiency_behavior_from_bam(
                    threshold.insufficiency_policy,
                ),
                required: true,
                declaration_origin: "bam_stage_metric_contract".to_string(),
            });
        }
    }

    for row in &vcf_report.rows {
        rows.push(vcf_acceptance_row(row)?);
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.metric_id.cmp(&right.metric_id))
    });
    Ok(rows)
}

pub(crate) fn scientific_acceptance_direction_label(
    direction: ScientificAcceptanceDirection,
) -> &'static str {
    match direction {
        ScientificAcceptanceDirection::ExactMatchPreferred => "exact_match_preferred",
        ScientificAcceptanceDirection::HigherIsBetter => "higher_is_better",
        ScientificAcceptanceDirection::LowerIsBetter => "lower_is_better",
        ScientificAcceptanceDirection::Minimum => "minimum",
        ScientificAcceptanceDirection::Maximum => "maximum",
        ScientificAcceptanceDirection::Range => "range",
        ScientificAcceptanceDirection::StructuredMatch => "structured_match",
    }
}

pub(crate) fn scientific_acceptance_tolerance_kind_label(
    kind: ScientificAcceptanceToleranceKind,
) -> &'static str {
    match kind {
        ScientificAcceptanceToleranceKind::RelativeFraction => "relative_fraction",
        ScientificAcceptanceToleranceKind::AbsoluteDelta => "absolute_delta",
        ScientificAcceptanceToleranceKind::ExactMatch => "exact_match",
        ScientificAcceptanceToleranceKind::NormalizedSetOverlap => "normalized_set_overlap",
        ScientificAcceptanceToleranceKind::NormalizedRecordOverlap => "normalized_record_overlap",
    }
}

pub(crate) fn scientific_acceptance_pass_rule_label(
    pass_rule: ScientificAcceptancePassRule,
) -> &'static str {
    match pass_rule {
        ScientificAcceptancePassRule::MustMatchReference => "must_match_reference",
        ScientificAcceptancePassRule::MustMeetOrExceedReference => "must_meet_or_exceed_reference",
        ScientificAcceptancePassRule::MustNotExceedReference => "must_not_exceed_reference",
        ScientificAcceptancePassRule::MustRemainWithinReferenceRange => {
            "must_remain_within_reference_range"
        }
        ScientificAcceptancePassRule::MustMatchReferenceStructure => {
            "must_match_reference_structure"
        }
    }
}

pub(crate) fn scientific_acceptance_insufficiency_behavior_label(
    behavior: ScientificAcceptanceInsufficiencyBehavior,
) -> &'static str {
    match behavior {
        ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison => {
            "refuse_stage_comparison"
        }
        ScientificAcceptanceInsufficiencyBehavior::DropMetricFromStage => "drop_metric_from_stage",
        ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage => "warn_and_exclude_stage",
    }
}

fn fastq_acceptance_row(
    stage_id: &str,
    metric_id: &str,
) -> Result<ScientificAcceptanceThresholdRow> {
    let (unit, direction, tolerance_kind, tolerance_value, pass_rule, insufficiency_behavior) =
        match (stage_id, metric_id) {
            ("fastq.index_reference", "index_build_exit_code") => (
                Some("exit_code"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::ExactMatch,
                0.0,
                ScientificAcceptancePassRule::MustMatchReference,
                ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison,
            ),
            ("fastq.profile_overrepresented_sequences", "sequence_count") => (
                Some("reads"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::ExactMatch,
                0.0,
                ScientificAcceptancePassRule::MustMatchReference,
                ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage,
            ),
            ("fastq.profile_overrepresented_sequences", "flagged_sequences") => (
                Some("sequences"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::AbsoluteDelta,
                1.0,
                ScientificAcceptancePassRule::MustMatchReference,
                ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage,
            ),
            ("fastq.profile_overrepresented_sequences", "top_fraction") => (
                Some("fraction"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::AbsoluteDelta,
                0.05,
                ScientificAcceptancePassRule::MustMatchReference,
                ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage,
            ),
            ("fastq.validate_reads", "format_validation_pass_rate") => (
                Some("fraction"),
                ScientificAcceptanceDirection::HigherIsBetter,
                ScientificAcceptanceToleranceKind::AbsoluteDelta,
                0.01,
                ScientificAcceptancePassRule::MustMeetOrExceedReference,
                ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison,
            ),
            _ => {
                return Err(anyhow!(
                    "FASTQ comparable metric `{stage_id}` / `{metric_id}` is missing scientific acceptance policy"
                ))
            }
        };

    Ok(ScientificAcceptanceThresholdRow {
        domain: "fastq".to_string(),
        stage_id: stage_id.to_string(),
        metric_id: metric_id.to_string(),
        metric_name: metric_id.to_string(),
        unit: unit.map(str::to_string),
        direction,
        tolerance_kind,
        tolerance_value,
        pass_rule,
        insufficiency_behavior,
        required: true,
        declaration_origin: "fastq_comparable_metric_policy".to_string(),
    })
}

fn vcf_acceptance_row(row: &VcfComparableMetricsRow) -> Result<ScientificAcceptanceThresholdRow> {
    let direction = scientific_acceptance_direction_from_vcf_label(&row.direction)?;
    let tolerance_kind = match direction {
        ScientificAcceptanceDirection::ExactMatchPreferred => {
            ScientificAcceptanceToleranceKind::ExactMatch
        }
        ScientificAcceptanceDirection::HigherIsBetter
        | ScientificAcceptanceDirection::LowerIsBetter => vcf_tolerance_kind(&row.unit)?,
        ScientificAcceptanceDirection::Minimum
        | ScientificAcceptanceDirection::Maximum
        | ScientificAcceptanceDirection::Range
        | ScientificAcceptanceDirection::StructuredMatch => {
            return Err(anyhow!(
                "VCF comparable metric `{}` / `{}` uses unsupported direction `{}`",
                row.stage_id,
                row.metric_id,
                row.direction
            ))
        }
    };
    let tolerance_value = vcf_tolerance_value(row, tolerance_kind)?;

    Ok(ScientificAcceptanceThresholdRow {
        domain: "vcf".to_string(),
        stage_id: row.stage_id.clone(),
        metric_id: row.metric_id.clone(),
        metric_name: row.metric_name.clone(),
        unit: Some(row.unit.clone()),
        direction,
        tolerance_kind,
        tolerance_value,
        pass_rule: scientific_acceptance_pass_rule_from_direction(direction),
        insufficiency_behavior: if row.required {
            ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison
        } else {
            ScientificAcceptanceInsufficiencyBehavior::DropMetricFromStage
        },
        required: row.required,
        declaration_origin: "vcf_comparable_metric_policy".to_string(),
    })
}

fn vcf_tolerance_kind(unit: &str) -> Result<ScientificAcceptanceToleranceKind> {
    match unit {
        "fraction" | "score" => Ok(ScientificAcceptanceToleranceKind::AbsoluteDelta),
        "sites" | "samples" | "variants" | "pairs" | "clusters" | "populations" | "genotypes"
        | "bases" => Ok(ScientificAcceptanceToleranceKind::RelativeFraction),
        _ => Err(anyhow!("VCF comparable metric unit `{unit}` is missing a tolerance kind policy")),
    }
}

fn vcf_tolerance_value(
    row: &VcfComparableMetricsRow,
    tolerance_kind: ScientificAcceptanceToleranceKind,
) -> Result<f64> {
    match tolerance_kind {
        ScientificAcceptanceToleranceKind::ExactMatch => Ok(0.0),
        ScientificAcceptanceToleranceKind::AbsoluteDelta => match row.metric_id.as_str() {
            "concordance" | "switch_error_proxy" | "missingness_post" => Ok(0.02),
            _ => Ok(0.05),
        },
        ScientificAcceptanceToleranceKind::RelativeFraction => {
            if row.stage_id == "vcf.phasing" && row.metric_id == "phase_block_n50" {
                Ok(0.15)
            } else {
                Ok(0.05)
            }
        }
        ScientificAcceptanceToleranceKind::NormalizedSetOverlap
        | ScientificAcceptanceToleranceKind::NormalizedRecordOverlap => Err(anyhow!(
            "VCF comparable metric `{}` / `{}` uses unsupported tolerance kind policy",
            row.stage_id,
            row.metric_id
        )),
    }
}

fn scientific_acceptance_direction_from_vcf_label(
    direction: &str,
) -> Result<ScientificAcceptanceDirection> {
    match direction {
        "exact_match_preferred" => Ok(ScientificAcceptanceDirection::ExactMatchPreferred),
        "higher_is_better" => Ok(ScientificAcceptanceDirection::HigherIsBetter),
        "lower_is_better" => Ok(ScientificAcceptanceDirection::LowerIsBetter),
        _ => Err(anyhow!("unknown VCF comparable metric direction `{direction}`")),
    }
}

fn scientific_acceptance_direction_from_bam(
    direction: BamScientificPassDirection,
) -> ScientificAcceptanceDirection {
    match direction {
        BamScientificPassDirection::Minimum => ScientificAcceptanceDirection::Minimum,
        BamScientificPassDirection::Maximum => ScientificAcceptanceDirection::Maximum,
        BamScientificPassDirection::Range => ScientificAcceptanceDirection::Range,
        BamScientificPassDirection::ExactMatch => {
            ScientificAcceptanceDirection::ExactMatchPreferred
        }
        BamScientificPassDirection::StructuredMatch => {
            ScientificAcceptanceDirection::StructuredMatch
        }
    }
}

fn scientific_acceptance_tolerance_kind_from_bam(
    kind: BamScientificToleranceKind,
) -> ScientificAcceptanceToleranceKind {
    match kind {
        BamScientificToleranceKind::RelativeFraction => {
            ScientificAcceptanceToleranceKind::RelativeFraction
        }
        BamScientificToleranceKind::AbsoluteDelta => {
            ScientificAcceptanceToleranceKind::AbsoluteDelta
        }
        BamScientificToleranceKind::ExactMatch => ScientificAcceptanceToleranceKind::ExactMatch,
        BamScientificToleranceKind::NormalizedSetOverlap => {
            ScientificAcceptanceToleranceKind::NormalizedSetOverlap
        }
        BamScientificToleranceKind::NormalizedRecordOverlap => {
            ScientificAcceptanceToleranceKind::NormalizedRecordOverlap
        }
    }
}

fn scientific_acceptance_insufficiency_behavior_from_bam(
    behavior: BamScientificInsufficiencyPolicy,
) -> ScientificAcceptanceInsufficiencyBehavior {
    match behavior {
        BamScientificInsufficiencyPolicy::RefuseStageComparison => {
            ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison
        }
        BamScientificInsufficiencyPolicy::DropMetricFromStage => {
            ScientificAcceptanceInsufficiencyBehavior::DropMetricFromStage
        }
        BamScientificInsufficiencyPolicy::WarnAndExcludeStage => {
            ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage
        }
    }
}

fn scientific_acceptance_pass_rule_from_bam(
    direction: BamScientificPassDirection,
) -> ScientificAcceptancePassRule {
    match direction {
        BamScientificPassDirection::Minimum => {
            ScientificAcceptancePassRule::MustMeetOrExceedReference
        }
        BamScientificPassDirection::Maximum => ScientificAcceptancePassRule::MustNotExceedReference,
        BamScientificPassDirection::Range => {
            ScientificAcceptancePassRule::MustRemainWithinReferenceRange
        }
        BamScientificPassDirection::ExactMatch => ScientificAcceptancePassRule::MustMatchReference,
        BamScientificPassDirection::StructuredMatch => {
            ScientificAcceptancePassRule::MustMatchReferenceStructure
        }
    }
}

fn scientific_acceptance_pass_rule_from_direction(
    direction: ScientificAcceptanceDirection,
) -> ScientificAcceptancePassRule {
    match direction {
        ScientificAcceptanceDirection::ExactMatchPreferred => {
            ScientificAcceptancePassRule::MustMatchReference
        }
        ScientificAcceptanceDirection::HigherIsBetter | ScientificAcceptanceDirection::Minimum => {
            ScientificAcceptancePassRule::MustMeetOrExceedReference
        }
        ScientificAcceptanceDirection::LowerIsBetter | ScientificAcceptanceDirection::Maximum => {
            ScientificAcceptancePassRule::MustNotExceedReference
        }
        ScientificAcceptanceDirection::Range => {
            ScientificAcceptancePassRule::MustRemainWithinReferenceRange
        }
        ScientificAcceptanceDirection::StructuredMatch => {
            ScientificAcceptancePassRule::MustMatchReferenceStructure
        }
    }
}

fn ensure_unique_rows(rows: &[ScientificAcceptanceThresholdRow]) -> Result<()> {
    let mut seen = BTreeMap::<(String, String, String), String>::new();
    for row in rows {
        let key = (row.domain.clone(), row.stage_id.clone(), row.metric_id.clone());
        if let Some(origin) = seen.insert(key.clone(), row.declaration_origin.clone()) {
            return Err(anyhow!(
                "duplicate scientific acceptance threshold row for `{}:{}/{}`
first origin: `{origin}`
second origin: `{}`",
                key.0,
                key.1,
                key.2,
                row.declaration_origin
            ));
        }
    }
    Ok(())
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
        render_scientific_acceptance_thresholds, scientific_acceptance_direction_label,
        scientific_acceptance_insufficiency_behavior_label, scientific_acceptance_pass_rule_label,
        scientific_acceptance_tolerance_kind_label, ScientificAcceptanceThresholdRow,
        ScientificAcceptanceThresholdsConfig, DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crates directory")
            .parent()
            .expect("repository root")
            .to_path_buf()
    }

    fn find_row<'a>(
        rows: &'a [ScientificAcceptanceThresholdRow],
        domain: &str,
        stage_id: &str,
        metric_id: &str,
    ) -> &'a ScientificAcceptanceThresholdRow {
        rows.iter()
            .find(|row| {
                row.domain == domain && row.stage_id == stage_id && row.metric_id == metric_id
            })
            .unwrap_or_else(|| {
                panic!("missing scientific acceptance threshold row `{domain}` / `{stage_id}` / `{metric_id}`")
            })
    }

    #[test]
    fn render_scientific_acceptance_thresholds_covers_all_comparable_metrics() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let output_path = tempdir.path().join(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH);

        let report = render_scientific_acceptance_thresholds(&repo_root, output_path.clone())
            .expect("render scientific acceptance thresholds");

        assert_eq!(report.row_count, report.comparable_metric_count);
        assert_eq!(report.domain_counts.get("fastq"), Some(&5));
        assert_eq!(report.domain_counts.get("bam"), Some(&51));
        assert_eq!(report.domain_counts.get("vcf"), Some(&35));
        assert!(report.config_path.ends_with(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH));

        let raw = std::fs::read_to_string(output_path).expect("read config");
        let parsed: ScientificAcceptanceThresholdsConfig =
            toml::from_str(&raw).expect("parse rendered config");
        assert_eq!(parsed.rows.len(), report.row_count);
    }

    #[test]
    fn render_scientific_acceptance_thresholds_preserves_domain_specific_semantics() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let report = render_scientific_acceptance_thresholds(
            &repo_root,
            tempdir.path().join(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
        )
        .expect("render scientific acceptance thresholds");

        let validate_errors = find_row(&report.rows, "bam", "bam.validate", "validation_errors");
        assert_eq!(
            scientific_acceptance_direction_label(validate_errors.direction),
            "structured_match"
        );
        assert_eq!(
            scientific_acceptance_tolerance_kind_label(validate_errors.tolerance_kind),
            "normalized_set_overlap"
        );
        assert_eq!(
            scientific_acceptance_pass_rule_label(validate_errors.pass_rule),
            "must_match_reference_structure"
        );
        assert_eq!(
            scientific_acceptance_insufficiency_behavior_label(
                validate_errors.insufficiency_behavior
            ),
            "refuse_stage_comparison"
        );

        let top_fraction = find_row(
            &report.rows,
            "fastq",
            "fastq.profile_overrepresented_sequences",
            "top_fraction",
        );
        assert_eq!(
            scientific_acceptance_direction_label(top_fraction.direction),
            "exact_match_preferred"
        );
        assert_eq!(
            scientific_acceptance_tolerance_kind_label(top_fraction.tolerance_kind),
            "absolute_delta"
        );
        assert_eq!(top_fraction.tolerance_value, 0.05);

        let dosage_r2 = find_row(&report.rows, "vcf", "vcf.imputation_metrics", "dosage_r2");
        assert!(!dosage_r2.required);
        assert_eq!(
            scientific_acceptance_insufficiency_behavior_label(dosage_r2.insufficiency_behavior),
            "drop_metric_from_stage"
        );
    }
}
