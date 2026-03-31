//! Owner: bijux-dna-bench
//! Gate policy engine for benchmark decisions.
//! Owns typed gating decisions with rationale trace.
//! Must not panic on missing metrics.
//! Invariants: decisions are deterministic.

mod config;
mod evaluation;

use bijux_dna_analyze::metric_semantics;

use crate::error::BenchError;
pub use config::{GatePolicy, GatePolicyOverrides};

impl GatePolicy {
    /// # Errors
    /// Returns an error if the policy references unknown metrics.
    pub fn validate(&self) -> Result<(), BenchError> {
        let mut unknown = Vec::new();
        for metric_id in self
            .required_metrics
            .iter()
            .chain(self.thresholds.keys())
            .chain(self.allowed_regressions.keys())
            .chain(self.must_not_regress.iter())
        {
            if metric_semantics(metric_id).is_none()
                && !self.semantics_overrides.contains_key(metric_id)
            {
                unknown.push(metric_id.clone());
            }
        }
        for override_policy in self.stage_overrides.values() {
            for metric_id in override_policy
                .required_metrics
                .iter()
                .chain(override_policy.thresholds.keys())
                .chain(override_policy.allowed_regressions.keys())
                .chain(override_policy.must_not_regress.iter())
            {
                if metric_semantics(metric_id).is_none()
                    && !override_policy.semantics_overrides.contains_key(metric_id)
                {
                    unknown.push(metric_id.clone());
                }
            }
        }
        if !unknown.is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "unknown metrics: {}",
                unknown.join(",")
            )));
        }
        Ok(())
    }
}
