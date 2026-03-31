use std::collections::BTreeMap;

use bijux_dna_analyze::MetricDirection;

#[derive(Debug, Clone)]
pub struct GatePolicy {
    pub objective: String,
    pub required_metrics: Vec<String>,
    pub thresholds: BTreeMap<String, f64>,
    pub allowed_regressions: BTreeMap<String, f64>,
    pub must_not_regress: Vec<String>,
    pub semantics_overrides: BTreeMap<String, MetricDirection>,
    pub stage_overrides: BTreeMap<String, GatePolicyOverrides>,
}

#[derive(Debug, Clone)]
pub struct GatePolicyOverrides {
    pub required_metrics: Vec<String>,
    pub thresholds: BTreeMap<String, f64>,
    pub allowed_regressions: BTreeMap<String, f64>,
    pub must_not_regress: Vec<String>,
    pub semantics_overrides: BTreeMap<String, MetricDirection>,
}
