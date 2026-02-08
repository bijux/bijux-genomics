use bijux_dna_core::prelude::invariants::{InvariantResultV1, StageVerdictV1};

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
