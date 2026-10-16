use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    derive_fastq_scientific_acceptance_threshold, derive_vcf_scientific_acceptance_threshold,
    validate_scientific_acceptance_thresholds_config,
    validate_scientific_acceptance_thresholds_report,
    SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1, SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    ScientificAcceptanceDirection, ScientificAcceptanceInsufficiencyBehavior,
    ScientificAcceptancePassRule, ScientificAcceptanceThresholdRow,
    ScientificAcceptanceThresholdsConfig, ScientificAcceptanceThresholdsReport,
    ScientificAcceptanceToleranceKind,
};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::{
    stage_comparable_metric_contracts_for_stage, BamScientificInsufficiencyPolicy,
    BamScientificPassDirection, BamScientificToleranceKind,
};

use super::bam_comparable_metrics::{
    render_bam_comparable_metrics, BamComparableMetricsReport, DEFAULT_BAM_COMPARABLE_METRICS_PATH,
};
use super::fastq_comparable_metrics::{
    render_fastq_comparable_metrics, FastqComparableMetricsReport,
    DEFAULT_FASTQ_COMPARABLE_METRICS_PATH,
};
use super::vcf_comparable_metrics::{
    render_vcf_comparable_metrics, VcfComparableMetricsReport, DEFAULT_VCF_COMPARABLE_METRICS_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH: &str =
    "configs/bench/local/scientific-acceptance-thresholds.toml";
pub(crate) const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_VERSION: &str =
    SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1;
const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_HEADER: &str = "# schema_version = 1\n\
# owner = bijux-dna-bench\n\
# purpose = Governed scientific acceptance thresholds for local benchmark comparisons.\n\
# authority = bijux-dna-bench\n\
# stability = evolving\n\
# last_updated = 2026-06-25\n\n";

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
    let config = ScientificAcceptanceThresholdsConfig {
        schema_version: SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_VERSION.to_string(),
        rows: rows.clone(),
    };
    validate_scientific_acceptance_thresholds_config(&config)?;
    let body = toml::to_string_pretty(&config)
        .context("serialize scientific acceptance thresholds config")?;
    let rendered = format!("{SCIENTIFIC_ACCEPTANCE_THRESHOLDS_HEADER}{body}");

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

    let report = ScientificAcceptanceThresholdsReport {
        schema_version: SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1.to_string(),
        config_path: path_relative_to_repo(repo_root, &output_path),
        comparable_metric_count,
        row_count: rows.len(),
        domain_counts,
        direction_counts,
        insufficiency_behavior_counts,
        rows,
    };
    validate_scientific_acceptance_thresholds_report(&report)?;
    Ok(report)
}

pub(crate) fn collect_scientific_acceptance_threshold_rows_from_reports(
    fastq_report: &FastqComparableMetricsReport,
    bam_report: &BamComparableMetricsReport,
    vcf_report: &VcfComparableMetricsReport,
) -> Result<Vec<ScientificAcceptanceThresholdRow>> {
    let mut rows = Vec::new();

    for row in &fastq_report.rows {
        for metric_id in &row.shared_metric_fields {
            rows.push(derive_fastq_scientific_acceptance_threshold(&row.stage_id, metric_id)?);
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
        rows.push(derive_vcf_scientific_acceptance_threshold(
            &row.stage_id,
            &row.metric_id,
            &row.metric_name,
            &row.unit,
            scientific_acceptance_direction_from_vcf(row.direction),
            row.required,
        )?);
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
    direction.as_str()
}

pub(crate) fn scientific_acceptance_tolerance_kind_label(
    kind: ScientificAcceptanceToleranceKind,
) -> &'static str {
    kind.as_str()
}

pub(crate) fn scientific_acceptance_pass_rule_label(
    pass_rule: ScientificAcceptancePassRule,
) -> &'static str {
    pass_rule.as_str()
}

pub(crate) fn scientific_acceptance_insufficiency_behavior_label(
    behavior: ScientificAcceptanceInsufficiencyBehavior,
) -> &'static str {
    behavior.as_str()
}

fn scientific_acceptance_direction_from_vcf(
    direction: super::vcf_comparable_metrics::ComparableMetricDirection,
) -> ScientificAcceptanceDirection {
    match direction {
        super::vcf_comparable_metrics::ComparableMetricDirection::ExactMatchPreferred => {
            ScientificAcceptanceDirection::ExactMatchPreferred
        }
        super::vcf_comparable_metrics::ComparableMetricDirection::HigherIsBetter => {
            ScientificAcceptanceDirection::HigherIsBetter
        }
        super::vcf_comparable_metrics::ComparableMetricDirection::LowerIsBetter => {
            ScientificAcceptanceDirection::LowerIsBetter
        }
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
        BamScientificPassDirection::Minimum => ScientificAcceptancePassRule::MeetOrExceedReference,
        BamScientificPassDirection::Maximum => ScientificAcceptancePassRule::NotExceedReference,
        BamScientificPassDirection::Range => {
            ScientificAcceptancePassRule::RemainWithinReferenceRange
        }
        BamScientificPassDirection::ExactMatch => ScientificAcceptancePassRule::MatchReference,
        BamScientificPassDirection::StructuredMatch => {
            ScientificAcceptancePassRule::MatchReferenceStructure
        }
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
    if let Ok(relative) = path.strip_prefix(repo_root) {
        return relative.to_string_lossy().replace('\\', "/");
    }
    let components = path.components().collect::<Vec<_>>();
    if let Some(start) = components.iter().position(
        |component| matches!(component, std::path::Component::Normal(name) if *name == "configs"),
    ) {
        return components[start..]
            .iter()
            .collect::<PathBuf>()
            .to_string_lossy()
            .replace('\\', "/");
    }
    path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned()
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
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
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
        assert_eq!(report.domain_counts.get("vcf"), Some(&29));
        assert!(report.config_path.ends_with(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH));

        let raw = std::fs::read_to_string(output_path).expect("read config");
        assert!(raw.starts_with("# schema_version = 1\n"));
        assert!(raw.contains("# last_updated = 2026-06-25\n"));
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
            "match_reference_structure"
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
