//! Request-scoped acquisition and reuse of admitted scientific readiness evidence.
mod acquisition;
mod session;
pub use acquisition::{collect_vcf_readiness_evidence_with_observations, ActiveEvidenceProvider};
pub use session::{AdapterInputProvider, VcfReadinessEvidenceSession};
