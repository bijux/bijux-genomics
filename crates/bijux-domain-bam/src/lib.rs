//! Dummy BAM domain stub.

use anyhow::{anyhow, Result};
use bijux_domain_adapter::DomainAdapter;

#[derive(Debug, Clone)]
pub struct BamArtifact;

#[derive(Debug, Clone)]
pub struct BamMetrics;

pub struct BamAdapter;

impl DomainAdapter for BamAdapter {
    type Artifact = BamArtifact;
    type Metrics = BamMetrics;

    fn validate_inputs(&self, _artifacts: &[Self::Artifact]) -> Result<()> {
        Err(anyhow!("bam domain stub"))
    }

    fn compute_deltas(
        &self,
        _before: &Self::Metrics,
        _after: &Self::Metrics,
    ) -> Result<serde_json::Value> {
        Err(anyhow!("bam domain stub"))
    }

    fn compatibility(&self, _from: &Self::Artifact, _to: &Self::Artifact) -> Result<()> {
        Err(anyhow!("bam domain stub"))
    }
}
