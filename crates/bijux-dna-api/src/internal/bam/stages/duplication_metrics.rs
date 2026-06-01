use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

const DEFAULT_LIBRARY_SIZE_REASON: &str = "tool_report_did_not_provide_library_size_estimate";

/// Write durable typed duplication-metrics artifacts beside BAM stage outputs.
///
/// # Errors
/// Returns an error if the raw duplication report cannot be parsed or derived artifacts cannot be
/// written.
pub(crate) fn write_stage_duplication_metrics_artifacts(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<()> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let summary = summarize_stage_duplication_metrics(stage_dir, plan, &input_bam)?;
    let summary_path = stage_dir.join("duplication.summary.json");

    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    let _policy_path = write_stage_duplication_policy(stage_dir, plan)?;
    Ok(())
}

pub(crate) fn write_stage_duplication_policy(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let path = stage_dir.join("duplication.policy.json");
    bijux_dna_infra::atomic_write_json(&path, &duplicate_policy_payload(plan))
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn summarize_stage_duplication_metrics(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    input_bam: &Path,
) -> Result<bijux_dna_domain_bam::BamDuplicationMetricsSummaryV1> {
    let report_path = stage_dir.join("duplication.metrics.json");
    let report: Value = serde_json::from_str(&std::fs::read_to_string(&report_path)?)
        .with_context(|| format!("parse {}", report_path.display()))?;
    let observed = observed_duplication_metrics(&report)?;
    let method = report.get("method").and_then(Value::as_str).unwrap_or(plan.tool_id.as_str());
    let optical_duplicates = json_string(plan.params.get("optical_duplicates"));
    let umi_policy = json_string(plan.params.get("umi_policy"));
    let duplicate_action = json_string(plan.params.get("duplicate_action"));
    let mut summary = bijux_dna_domain_bam::summarize_bam_duplication_metrics(
        "bam.duplication_metrics",
        method,
        input_bam,
        observed.examined_reads,
        observed.duplicate_reads,
        observed.estimated_library_size,
        observed.insufficient_library_size_reason.as_deref(),
        optical_duplicates.as_deref(),
        umi_policy.as_deref(),
        duplicate_action.as_deref(),
    );
    summary.duplicate_fraction = observed.duplicate_fraction;
    Ok(summary)
}

fn duplicate_policy_payload(
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> bijux_dna_domain_bam::BamDuplicatePolicyV1 {
    bijux_dna_domain_bam::BamDuplicatePolicyV1 {
        schema_version: bijux_dna_domain_bam::BAM_DUPLICATE_POLICY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.duplication_metrics".to_string(),
        library_type: None,
        optical_duplicates: json_string(plan.params.get("optical_duplicates")),
        umi_policy: json_string(plan.params.get("umi_policy")),
        duplicate_action: json_string(plan.params.get("duplicate_action")),
        policy_scope: "observation_only".to_string(),
        library_semantics: vec!["reports duplicate burden without mutating BAM outputs".to_string()],
        comparison_ready_with: vec!["picard".to_string(), "samtools".to_string()],
    }
}

struct ObservedDuplicationMetrics {
    examined_reads: u64,
    duplicate_reads: u64,
    duplicate_fraction: f64,
    estimated_library_size: Option<u64>,
    insufficient_library_size_reason: Option<String>,
}

fn observed_duplication_metrics(report: &Value) -> Result<ObservedDuplicationMetrics> {
    let duplicate_reads = json_u64(report.get("duplicate_reads"))
        .or_else(|| json_u64(report.get("duplicate_pairs")))
        .or_else(|| json_u64(report.get("read_pair_duplicates")))
        .ok_or_else(|| anyhow!("duplication.metrics.json is missing duplicate counts"))?;
    let explicit_fraction = json_f64(report.get("duplicate_fraction"))
        .or_else(|| json_f64(report.get("pct_duplication")));
    let examined_reads = match json_u64(report.get("examined_reads"))
        .or_else(|| json_u64(report.get("examined_pairs")))
    {
        Some(examined_reads) => examined_reads,
        None => infer_examined_reads(duplicate_reads, explicit_fraction)?,
    };
    let duplicate_fraction =
        explicit_fraction.unwrap_or_else(|| fraction_from_counts(examined_reads, duplicate_reads));
    let estimated_library_size = json_u64(report.get("estimated_library_size"));
    let insufficient_library_size_reason = if estimated_library_size.is_some() {
        None
    } else {
        Some(
            json_string(report.get("insufficient_library_size_reason"))
                .unwrap_or_else(|| DEFAULT_LIBRARY_SIZE_REASON.to_string()),
        )
    };
    Ok(ObservedDuplicationMetrics {
        examined_reads,
        duplicate_reads,
        duplicate_fraction,
        estimated_library_size,
        insufficient_library_size_reason,
    })
}

fn infer_examined_reads(duplicate_reads: u64, duplicate_fraction: Option<f64>) -> Result<u64> {
    match duplicate_fraction {
        Some(fraction) if fraction > 0.0 => {
            let inferred = (duplicate_reads as f64 / fraction).round();
            if !inferred.is_finite() || inferred < duplicate_reads as f64 {
                return Err(anyhow!(
                    "duplication.metrics.json duplicate fraction is inconsistent with duplicate counts"
                ));
            }
            Ok(inferred as u64)
        }
        Some(0.0) if duplicate_reads == 0 => Ok(0),
        Some(0.0) => Err(anyhow!(
            "duplication.metrics.json cannot declare zero duplicate fraction with nonzero duplicate counts"
        )),
        Some(_) => Err(anyhow!(
            "duplication.metrics.json duplicate fraction must be non-negative"
        )),
        None => Err(anyhow!(
            "duplication.metrics.json is missing examined counts and duplicate fraction"
        )),
    }
}

fn fraction_from_counts(examined_reads: u64, duplicate_reads: u64) -> f64 {
    if examined_reads == 0 {
        0.0
    } else {
        duplicate_reads as f64 / examined_reads as f64
    }
}

fn json_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(ToOwned::to_owned)
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    match value {
        Some(Value::Number(number)) => number.as_u64(),
        Some(Value::String(number)) => number.parse::<u64>().ok().or_else(|| {
            number
                .parse::<f64>()
                .ok()
                .filter(|parsed| parsed.is_finite() && *parsed >= 0.0)
                .map(|parsed| parsed.round() as u64)
        }),
        _ => None,
    }
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    match value {
        Some(Value::Number(number)) => number.as_f64(),
        Some(Value::String(number)) => number.parse::<f64>().ok(),
        _ => None,
    }
}
