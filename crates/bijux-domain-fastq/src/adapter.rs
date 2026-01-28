use anyhow::{anyhow, Result};
use bijux_domain_adapter::{DomainAdapter, DomainCapability};
use serde_json::json;

#[allow(dead_code)]
pub struct FastqAdapter;

impl DomainAdapter for FastqAdapter {
    type Artifact = bijux_engine::api::DataArtifact;
    type Metrics = bijux_analyze::MetricSet<bijux_analyze::FastqStatsMetrics>;

    fn validate_inputs(&self, artifacts: &[Self::Artifact]) -> Result<()> {
        if artifacts.is_empty() {
            return Err(anyhow!("no FASTQ artifacts provided"));
        }
        Ok(())
    }

    fn compute_deltas(
        &self,
        _before: &Self::Metrics,
        _after: &Self::Metrics,
    ) -> Result<serde_json::Value> {
        Ok(json!({}))
    }

    fn compatibility(&self, _from: &Self::Artifact, _to: &Self::Artifact) -> Result<()> {
        Ok(())
    }
}

impl DomainCapability for FastqAdapter {
    type Artifact = crate::contracts::FastqArtifact;
    type Metrics = serde_json::Value;

    fn domain_name() -> &'static str {
        "fastq"
    }

    fn canonical_pipeline() -> bijux_engine::api::PipelineSpec {
        let canonical = crate::pipeline::canonical_pipeline();
        bijux_engine::api::PipelineSpec {
            stages: canonical.required,
        }
    }
}
