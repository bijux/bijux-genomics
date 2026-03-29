use bijux_dna_core::metrics::{parse_derived_metric_id, parse_metric_id};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzeMetricId {
    Metric(bijux_dna_core::metrics::MetricId),
    Derived(bijux_dna_core::metrics::DerivedMetricId),
}

impl AnalyzeMetricId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Metric(id) => id.as_str(),
            Self::Derived(id) => id.as_str(),
        }
    }
}

impl std::str::FromStr for AnalyzeMetricId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some(id) = parse_metric_id(s) {
            return Ok(Self::Metric(id));
        }
        if let Some(id) = parse_derived_metric_id(s) {
            return Ok(Self::Derived(id));
        }
        Err(anyhow::anyhow!(
            "unknown metric id `{s}`; register in core metric registry first"
        ))
    }
}
