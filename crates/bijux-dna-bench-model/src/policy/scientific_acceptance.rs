//! Owner: deterministic FASTQ and VCF scientific acceptance declarations.

use anyhow::{anyhow, Result};
use bijux_dna_core::id_catalog::{
    FASTQ_INDEX_REFERENCE, FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES, FASTQ_VALIDATE_READS,
    VCF_PHASING,
};

use crate::{
    ScientificAcceptanceDirection, ScientificAcceptanceInsufficiencyBehavior,
    ScientificAcceptancePassRule, ScientificAcceptanceThresholdRow,
    ScientificAcceptanceToleranceKind,
};

/// Derive the governed FASTQ threshold for a declared stage and metric.
/// # Errors
/// Rejects stage/metric pairs without a governed scientific acceptance policy.
pub fn derive_fastq_scientific_acceptance_threshold(
    stage_id: &str,
    metric_id: &str,
) -> Result<ScientificAcceptanceThresholdRow> {
    let (unit, direction, tolerance_kind, tolerance_value, pass_rule, insufficiency_behavior) =
        match (stage_id, metric_id) {
            (FASTQ_INDEX_REFERENCE, "index_build_exit_code") => (
                Some("exit_code"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::ExactMatch,
                0.0,
                ScientificAcceptancePassRule::MatchReference,
                ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison,
            ),
            (FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES, "sequence_count") => (
                Some("reads"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::ExactMatch,
                0.0,
                ScientificAcceptancePassRule::MatchReference,
                ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage,
            ),
            (FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES, "flagged_sequences") => (
                Some("sequences"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::AbsoluteDelta,
                1.0,
                ScientificAcceptancePassRule::MatchReference,
                ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage,
            ),
            (FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES, "top_fraction") => (
                Some("fraction"),
                ScientificAcceptanceDirection::ExactMatchPreferred,
                ScientificAcceptanceToleranceKind::AbsoluteDelta,
                0.05,
                ScientificAcceptancePassRule::MatchReference,
                ScientificAcceptanceInsufficiencyBehavior::WarnAndExcludeStage,
            ),
            (FASTQ_VALIDATE_READS, "format_validation_pass_rate") => (
                Some("fraction"),
                ScientificAcceptanceDirection::HigherIsBetter,
                ScientificAcceptanceToleranceKind::AbsoluteDelta,
                0.01,
                ScientificAcceptancePassRule::MeetOrExceedReference,
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

/// Derive the governed VCF threshold from an acquired comparable metric.
///
/// Exact matching takes precedence over unit classification. Requiredness controls
/// insufficiency handling; declaration derivation does not evaluate measured results.
/// # Errors
/// Rejects unsupported directions or units in their established priority order.
pub fn derive_vcf_scientific_acceptance_threshold(
    stage_id: &str,
    metric_id: &str,
    metric_name: &str,
    unit: &str,
    direction: ScientificAcceptanceDirection,
    required: bool,
) -> Result<ScientificAcceptanceThresholdRow> {
    let tolerance_kind = match direction {
        ScientificAcceptanceDirection::ExactMatchPreferred => {
            ScientificAcceptanceToleranceKind::ExactMatch
        }
        ScientificAcceptanceDirection::HigherIsBetter
        | ScientificAcceptanceDirection::LowerIsBetter => vcf_tolerance_kind(unit)?,
        ScientificAcceptanceDirection::Minimum
        | ScientificAcceptanceDirection::Maximum
        | ScientificAcceptanceDirection::Range
        | ScientificAcceptanceDirection::StructuredMatch => {
            return Err(anyhow!(
                "VCF comparable metric `{}` / `{}` uses unsupported direction `{}`",
                stage_id,
                metric_id,
                direction.as_str()
            ))
        }
    };
    let tolerance_value = vcf_tolerance_value(stage_id, metric_id, tolerance_kind)?;

    Ok(ScientificAcceptanceThresholdRow {
        domain: "vcf".to_string(),
        stage_id: stage_id.to_string(),
        metric_id: metric_id.to_string(),
        metric_name: metric_name.to_string(),
        unit: Some(unit.to_string()),
        direction,
        tolerance_kind,
        tolerance_value,
        pass_rule: scientific_acceptance_pass_rule_from_direction(direction),
        insufficiency_behavior: if required {
            ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison
        } else {
            ScientificAcceptanceInsufficiencyBehavior::DropMetricFromStage
        },
        required,
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
    stage_id: &str,
    metric_id: &str,
    tolerance_kind: ScientificAcceptanceToleranceKind,
) -> Result<f64> {
    match tolerance_kind {
        ScientificAcceptanceToleranceKind::ExactMatch => Ok(0.0),
        ScientificAcceptanceToleranceKind::AbsoluteDelta => match metric_id {
            "concordance" | "switch_error_proxy" | "missingness_post" => Ok(0.02),
            _ => Ok(0.05),
        },
        ScientificAcceptanceToleranceKind::RelativeFraction => {
            if stage_id == VCF_PHASING && metric_id == "phase_block_n50" {
                Ok(0.15)
            } else {
                Ok(0.05)
            }
        }
        ScientificAcceptanceToleranceKind::NormalizedSetOverlap
        | ScientificAcceptanceToleranceKind::NormalizedRecordOverlap => Err(anyhow!(
            "VCF comparable metric `{stage_id}` / `{metric_id}` uses unsupported tolerance kind policy"
        )),
    }
}

fn scientific_acceptance_pass_rule_from_direction(
    direction: ScientificAcceptanceDirection,
) -> ScientificAcceptancePassRule {
    match direction {
        ScientificAcceptanceDirection::ExactMatchPreferred => {
            ScientificAcceptancePassRule::MatchReference
        }
        ScientificAcceptanceDirection::HigherIsBetter | ScientificAcceptanceDirection::Minimum => {
            ScientificAcceptancePassRule::MeetOrExceedReference
        }
        ScientificAcceptanceDirection::LowerIsBetter | ScientificAcceptanceDirection::Maximum => {
            ScientificAcceptancePassRule::NotExceedReference
        }
        ScientificAcceptanceDirection::Range => {
            ScientificAcceptancePassRule::RemainWithinReferenceRange
        }
        ScientificAcceptanceDirection::StructuredMatch => {
            ScientificAcceptancePassRule::MatchReferenceStructure
        }
    }
}
