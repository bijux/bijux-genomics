use anyhow::{anyhow, Result};
use bijux_domain_adapter::DomainAdapter;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct DummyArtifact {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DummyMetrics {
    pub value: i64,
}

pub struct DummyAdapter;

impl DomainAdapter for DummyAdapter {
    type Artifact = DummyArtifact;
    type Metrics = DummyMetrics;

    fn validate_inputs(&self, artifacts: &[Self::Artifact]) -> Result<()> {
        if artifacts.is_empty() {
            return Err(anyhow!("no dummy artifacts"));
        }
        Ok(())
    }

    fn compute_deltas(
        &self,
        before: &Self::Metrics,
        after: &Self::Metrics,
    ) -> Result<serde_json::Value> {
        Ok(json!({ "delta": after.value - before.value }))
    }

    fn compatibility(&self, _from: &Self::Artifact, _to: &Self::Artifact) -> Result<()> {
        Ok(())
    }
}
