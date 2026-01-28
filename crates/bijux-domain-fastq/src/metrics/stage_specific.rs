use super::common::{
    metric_f64, metric_u64, ContaminationMetrics, IntegrityMetrics, QualityShiftMetrics,
    RetentionMetrics, SemanticMetrics,
};
use super::deltas::delta_from_counts;

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
