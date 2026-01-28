use anyhow::{anyhow, Result};

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricDescriptor {
    pub value: serde_json::Value,
    pub meaning: &'static str,
    pub range: &'static str,
    pub notes: &'static str,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrityMetrics {
    pub reads_in: MetricDescriptor,
    pub reads_out: MetricDescriptor,
    pub bases_in: MetricDescriptor,
    pub bases_out: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RetentionMetrics {
    pub read_retention: MetricDescriptor,
    pub base_retention: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct QualityShiftMetrics {
    pub mean_q_before: MetricDescriptor,
    pub mean_q_after: MetricDescriptor,
    pub delta_mean_q: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContaminationMetrics {
    pub gc_before: MetricDescriptor,
    pub gc_after: MetricDescriptor,
    pub delta_gc: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SemanticMetrics {
    pub integrity: IntegrityMetrics,
    pub retention: RetentionMetrics,
    pub quality_shift: Option<QualityShiftMetrics>,
    pub contamination: Option<ContaminationMetrics>,
    pub interpretation: &'static str,
}

fn metric_u64(
    value: u64,
    meaning: &'static str,
    range: &'static str,
    notes: &'static str,
) -> MetricDescriptor {
    MetricDescriptor {
        value: serde_json::json!(value),
        meaning,
        range,
        notes,
    }
}

fn metric_f64(
    value: f64,
    meaning: &'static str,
    range: &'static str,
    notes: &'static str,
) -> MetricDescriptor {
    MetricDescriptor {
        value: serde_json::json!(value),
        meaning,
        range,
        notes,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FastqDelta {
    pub delta_mean_q: f64,
    pub delta_gc: f64,
    pub read_retention: f64,
    pub base_retention: f64,
}

impl FastqDelta {
    /// Validate delta metrics are sane.
    ///
    /// # Errors
    /// Returns an error if any delta values are invalid.
    pub fn validate(&self) -> Result<()> {
        if !self.delta_mean_q.is_finite() {
            return Err(anyhow!("delta_mean_q must be finite"));
        }
        if !self.delta_gc.is_finite() {
            return Err(anyhow!("delta_gc must be finite"));
        }
        if !(0.0..=1.0).contains(&self.read_retention) {
            return Err(anyhow!("read_retention must be within [0, 1]"));
        }
        if !(0.0..=1.0).contains(&self.base_retention) {
            return Err(anyhow!("base_retention must be within [0, 1]"));
        }
        Ok(())
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn ratio_u64(num: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        (num as f64) / (denom as f64)
    }
}

#[must_use]
pub fn compute_delta(
    before: bijux_engine::api::SeqkitMetrics,
    after: bijux_engine::api::SeqkitMetrics,
) -> FastqDelta {
    let read_retention = if before.reads > 0 {
        ratio_u64(after.reads, before.reads)
    } else {
        0.0
    };
    let base_retention = if before.bases > 0 {
        ratio_u64(after.bases, before.bases)
    } else {
        0.0
    };
    FastqDelta {
        delta_mean_q: after.mean_q - before.mean_q,
        delta_gc: after.gc_percent - before.gc_percent,
        read_retention,
        base_retention,
    }
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn delta_from_counts(
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    mean_q_before: f64,
    mean_q_after: f64,
    gc_before: f64,
    gc_after: f64,
) -> FastqDelta {
    let read_retention = if reads_in > 0 {
        ratio_u64(reads_out, reads_in)
    } else {
        0.0
    };
    let base_retention = if bases_in > 0 {
        ratio_u64(bases_out, bases_in)
    } else {
        0.0
    };
    FastqDelta {
        delta_mean_q: mean_q_after - mean_q_before,
        delta_gc: gc_after - gc_before,
        read_retention,
        base_retention,
    }
}

#[must_use]
pub fn semantic_trim(metrics: &bijux_analyze::FastqTrimMetrics) -> SemanticMetrics {
    let delta = delta_from_counts(
        metrics.reads_in,
        metrics.reads_out,
        metrics.bases_in,
        metrics.bases_out,
        metrics.mean_q_before,
        metrics.mean_q_after,
        0.0,
        0.0,
    );
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(
                metrics.reads_in,
                "Input reads",
                ">= 0",
                "Raw input read count.",
            ),
            reads_out: metric_u64(
                metrics.reads_out,
                "Output reads",
                ">= 0",
                "Reads after trimming.",
            ),
            bases_in: metric_u64(metrics.bases_in, "Input bases", ">= 0", "Raw input bases."),
            bases_out: metric_u64(
                metrics.bases_out,
                "Output bases",
                ">= 0",
                "Bases after trimming.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Bases retained",
                "[0,1]",
                "Fraction of bases retained.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(
                metrics.mean_q_before,
                "Mean Q before",
                "[0,45]",
                "Mean quality before trimming.",
            ),
            mean_q_after: metric_f64(
                metrics.mean_q_after,
                "Mean Q after",
                "[0,45]",
                "Mean quality after trimming.",
            ),
            delta_mean_q: metric_f64(
                delta.delta_mean_q,
                "Mean Q shift",
                "(-45,45)",
                "Mean quality change.",
            ),
        }),
        contamination: None,
        interpretation: "Trim should increase mean quality and may reduce read/base counts.",
    }
}

#[must_use]
pub fn semantic_filter(metrics: &bijux_analyze::FastqFilterMetrics) -> SemanticMetrics {
    let delta = delta_from_counts(
        metrics.reads_in,
        metrics.reads_out,
        metrics.reads_in,
        metrics.reads_out,
        metrics.mean_q_before,
        metrics.mean_q_after,
        0.0,
        0.0,
    );
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(
                metrics.reads_in,
                "Input reads",
                ">= 0",
                "Raw input read count.",
            ),
            reads_out: metric_u64(
                metrics.reads_out,
                "Output reads",
                ">= 0",
                "Reads after filtering.",
            ),
            bases_in: metric_u64(
                metrics.reads_in,
                "Input bases",
                ">= 0",
                "Approx bases inferred from reads.",
            ),
            bases_out: metric_u64(
                metrics.reads_out,
                "Output bases",
                ">= 0",
                "Approx bases after filtering.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Bases retained",
                "[0,1]",
                "Fraction of bases retained.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(
                metrics.mean_q_before,
                "Mean Q before",
                "[0,45]",
                "Mean quality before filtering.",
            ),
            mean_q_after: metric_f64(
                metrics.mean_q_after,
                "Mean Q after",
                "[0,45]",
                "Mean quality after filtering.",
            ),
            delta_mean_q: metric_f64(
                delta.delta_mean_q,
                "Mean Q shift",
                "(-45,45)",
                "Mean quality change.",
            ),
        }),
        contamination: None,
        interpretation: "Filter should drop low-quality reads and improve mean quality.",
    }
}

#[must_use]
pub fn semantic_validate(metrics: &bijux_analyze::FastqValidateMetrics) -> SemanticMetrics {
    let reads_in = metrics.reads_total;
    let reads_out = metrics.reads_valid;
    let delta = delta_from_counts(reads_in, reads_out, reads_in, reads_out, 0.0, 0.0, 0.0, 0.0);
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(reads_in, "Total reads", ">= 0", "Total reads inspected."),
            reads_out: metric_u64(
                reads_out,
                "Valid reads",
                ">= 0",
                "Reads passing validation.",
            ),
            bases_in: metric_u64(0, "Input bases", ">= 0", "Not computed in validation."),
            bases_out: metric_u64(0, "Output bases", ">= 0", "Not computed in validation."),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Valid read ratio",
                "[0,1]",
                "Fraction of valid reads.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Base retention",
                "[0,1]",
                "Not computed in validation.",
            ),
        },
        quality_shift: None,
        contamination: None,
        interpretation: "Validation should report invalid reads; no data is modified.",
    }
}

#[must_use]
pub fn semantic_stats(metrics: &bijux_analyze::FastqStatsMetrics) -> SemanticMetrics {
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(metrics.reads_total, "Input reads", ">= 0", "Total reads."),
            reads_out: metric_u64(
                metrics.reads_total,
                "Output reads",
                ">= 0",
                "Stats stage does not modify reads.",
            ),
            bases_in: metric_u64(metrics.bases_total, "Input bases", ">= 0", "Total bases."),
            bases_out: metric_u64(
                metrics.bases_total,
                "Output bases",
                ">= 0",
                "Stats stage does not modify bases.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                1.0,
                "Reads retained",
                "[0,1]",
                "Stats is non-transforming.",
            ),
            base_retention: metric_f64(
                1.0,
                "Bases retained",
                "[0,1]",
                "Stats is non-transforming.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(metrics.mean_q, "Mean Q", "[0,45]", "Mean quality."),
            mean_q_after: metric_f64(metrics.mean_q, "Mean Q", "[0,45]", "Mean quality."),
            delta_mean_q: metric_f64(
                0.0,
                "Mean Q shift",
                "0",
                "Stats stage does not alter reads.",
            ),
        }),
        contamination: Some(ContaminationMetrics {
            gc_before: metric_f64(metrics.gc_percent, "GC%", "[0,100]", "GC percentage."),
            gc_after: metric_f64(metrics.gc_percent, "GC%", "[0,100]", "GC percentage."),
            delta_gc: metric_f64(0.0, "GC shift", "0", "Stats stage does not alter reads."),
        }),
        interpretation: "Stats is observational; values describe the input only.",
    }
}
