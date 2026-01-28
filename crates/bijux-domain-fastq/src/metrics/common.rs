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

#[must_use]
pub fn metric_u64(
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

#[must_use]
pub fn metric_f64(
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
