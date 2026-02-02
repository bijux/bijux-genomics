//! BAM invariants and thresholds.

use bijux_core::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};

use crate::authenticity::contamination_cross_check;
use crate::metrics::BamMetricsV1;

#[derive(Debug, Clone)]
pub struct BamInvariantThresholds {
    pub contamination_warn: f64,
    pub contamination_fail: f64,
    pub coverage_warn: f64,
    pub coverage_fail: f64,
    pub duplication_warn: f64,
    pub complexity_low: u64,
}

impl Default for BamInvariantThresholds {
    fn default() -> Self {
        Self {
            contamination_warn: 0.05,
            contamination_fail: 0.10,
            coverage_warn: 0.5,
            coverage_fail: 0.2,
            duplication_warn: 0.5,
            complexity_low: 1_000_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BamInvariantEvaluation {
    pub results: Vec<InvariantResultV1>,
    pub verdict: StageVerdictV1,
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

fn dup_fraction(metrics: &BamMetricsV1) -> f64 {
    if metrics.alignment.total == 0 {
        0.0
    } else {
        u64_to_f64(metrics.alignment.duplicates) / u64_to_f64(metrics.alignment.total)
    }
}

#[must_use]
pub fn evaluate_bam_invariants(
    stage_id: &str,
    metrics: &BamMetricsV1,
    thresholds: &BamInvariantThresholds,
) -> BamInvariantEvaluation {
    #[allow(clippy::too_many_lines)]
    evaluate_bam_invariants_inner(stage_id, metrics, thresholds)
}

#[allow(clippy::too_many_lines)]
fn evaluate_bam_invariants_inner(
    stage_id: &str,
    metrics: &BamMetricsV1,
    thresholds: &BamInvariantThresholds,
) -> BamInvariantEvaluation {
    let mut results = Vec::new();
    let mut status = InvariantStatusV1::Pass;

    let contamination = metrics.contamination.estimate;
    let contamination_status = if contamination >= thresholds.contamination_fail {
        InvariantStatusV1::Fail
    } else if contamination >= thresholds.contamination_warn {
        InvariantStatusV1::Warn
    } else {
        InvariantStatusV1::Pass
    };
    status = std::cmp::max(status.clone(), contamination_status.clone());
    results.push(InvariantResultV1 {
        id: "contamination_rate".to_string(),
        status: contamination_status,
        message: format!("contamination estimate {contamination:.3}"),
        remediation: Some("review contamination model or filter aggressively".to_string()),
    });

    let coverage = metrics.coverage.mean;
    let coverage_status = if coverage <= thresholds.coverage_fail {
        InvariantStatusV1::Fail
    } else if coverage <= thresholds.coverage_warn {
        InvariantStatusV1::Warn
    } else {
        InvariantStatusV1::Pass
    };
    status = std::cmp::max(status.clone(), coverage_status.clone());
    results.push(InvariantResultV1 {
        id: "coverage_mean".to_string(),
        status: coverage_status,
        message: format!("mean coverage {coverage:.3}x"),
        remediation: Some("insufficient coverage for downstream inference".to_string()),
    });

    let dup_fraction = dup_fraction(metrics);
    let duplication_status = if dup_fraction >= thresholds.duplication_warn {
        InvariantStatusV1::Warn
    } else {
        InvariantStatusV1::Pass
    };
    status = std::cmp::max(status.clone(), duplication_status.clone());
    results.push(InvariantResultV1 {
        id: "duplicate_fraction".to_string(),
        status: duplication_status,
        message: format!("duplicate fraction {dup_fraction:.3}"),
        remediation: Some("consider markdup removal or library complexity checks".to_string()),
    });

    let complexity = metrics.complexity.observed_reads;
    if complexity < thresholds.complexity_low {
        let message = if dup_fraction >= thresholds.duplication_warn {
            "low complexity with high duplicates indicates library saturation"
        } else {
            "preseq complexity is low; library likely saturated"
        };
        results.push(InvariantResultV1 {
            id: "complexity_vs_duplicates".to_string(),
            status: InvariantStatusV1::Warn,
            message: message.to_string(),
            remediation: Some("consider deeper library prep or avoid over-filtering".to_string()),
        });
    } else if dup_fraction >= thresholds.duplication_warn {
        results.push(InvariantResultV1 {
            id: "complexity_vs_duplicates".to_string(),
            status: InvariantStatusV1::Warn,
            message: "duplicates high but preseq complexity suggests high diversity".to_string(),
            remediation: Some("verify markdup configuration or library prep".to_string()),
        });
    }

    let damage = metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p);
    let assessment = contamination_cross_check(damage, contamination);
    let contam_status = if contamination >= thresholds.contamination_fail && damage < 0.05 {
        InvariantStatusV1::Fail
    } else if contamination >= thresholds.contamination_warn && damage < 0.05 {
        InvariantStatusV1::Warn
    } else {
        InvariantStatusV1::Pass
    };
    status = std::cmp::max(status.clone(), contam_status.clone());
    results.push(InvariantResultV1 {
        id: "contamination_damage_check".to_string(),
        status: contam_status,
        message: assessment,
        remediation: Some("review contamination model vs damage profile".to_string()),
    });

    if let Some(inference) = metrics.authenticity.library_type_inference.as_ref() {
        if let Some(declared) = inference.declared {
            if declared != inference.inferred {
                results.push(InvariantResultV1 {
                    id: "declared_vs_inferred_library".to_string(),
                    status: InvariantStatusV1::Warn,
                    message: format!(
                        "declared library type {:?} conflicts with inferred {:?}",
                        declared, inference.inferred
                    ),
                    remediation: Some(
                        "verify library metadata or rerun damage-based inference".to_string(),
                    ),
                });
            }
        }
    }

    if damage < 0.05 && metrics.mapq.mean >= 40.0 {
        results.push(InvariantResultV1 {
            id: "damage_mapq_correlation".to_string(),
            status: InvariantStatusV1::Warn,
            message: "high MAPQ with low damage signal suggests modern contamination".to_string(),
            remediation: Some("inspect damage profile and contamination estimates".to_string()),
        });
    }

    if let Some(comparison) = metrics.damage_comparison.as_ref() {
        if comparison.exceeds_threshold {
            results.push(InvariantResultV1 {
                id: "damage_tool_disagreement".to_string(),
                status: InvariantStatusV1::Warn,
                message: format!(
                    "damage tools {} vs {} disagree (C→T Δ{:.3}, G→A Δ{:.3})",
                    comparison.tool_a,
                    comparison.tool_b,
                    comparison.c_to_t_diff,
                    comparison.g_to_a_diff
                ),
                remediation: Some(
                    "verify damage tool inputs or rerun with consistent parameters".to_string(),
                ),
            });
        }
    }

    let mut reasons = Vec::new();
    for result in &results {
        if matches!(
            result.status,
            InvariantStatusV1::Warn | InvariantStatusV1::Fail
        ) {
            reasons.push(result.id.clone());
        }
    }
    let verdict = StageVerdictV1 {
        stage_id: stage_id.to_string(),
        verdict: status,
        reasons,
        key_metrics: serde_json::json!({
            "contamination": metrics.contamination.estimate,
            "coverage_mean": metrics.coverage.mean,
            "dup_fraction": dup_fraction,
        }),
    };

    BamInvariantEvaluation { results, verdict }
}
