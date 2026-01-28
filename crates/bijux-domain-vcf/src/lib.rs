//! Dummy VCF domain stub.

use anyhow::{anyhow, Result};
use bijux_domain_adapter::DomainAdapter;

#[derive(Debug, Clone)]
pub struct VcfArtifact;

#[derive(Debug, Clone)]
pub struct VcfMetrics;

pub struct VcfAdapter;

impl DomainAdapter for VcfAdapter {
    type Artifact = VcfArtifact;
    type Metrics = VcfMetrics;

    fn validate_inputs(&self, _artifacts: &[Self::Artifact]) -> Result<()> {
        Err(anyhow!("vcf domain stub"))
    }

    fn compute_deltas(
        &self,
        _before: &Self::Metrics,
        _after: &Self::Metrics,
    ) -> Result<serde_json::Value> {
        Err(anyhow!("vcf domain stub"))
    }

    fn compatibility(&self, _from: &Self::Artifact, _to: &Self::Artifact) -> Result<()> {
        Err(anyhow!("vcf domain stub"))
    }
}
