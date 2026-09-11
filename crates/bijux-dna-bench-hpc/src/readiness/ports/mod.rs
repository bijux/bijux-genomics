//! Explicit typed providers for local readiness orchestration.
mod inventory;
pub mod observations;
pub use inventory::StageEvidence;
mod harness;
pub use harness::HarnessEvidence;
mod corpora;
pub use corpora::CorpusEvidence;
mod pipelines;
pub use pipelines::PipelineEvidence;
mod slurm;
pub use slurm::SlurmArtifactEvidence;
mod campaign;
pub use campaign::LocalCampaignEvidence;
/// Complete local readiness provider composition; there is no remote submission method.
pub trait LocalReadinessBackend:
    StageEvidence
    + HarnessEvidence
    + CorpusEvidence
    + PipelineEvidence
    + SlurmArtifactEvidence
    + LocalCampaignEvidence
{
}
impl<
        T: StageEvidence
            + HarnessEvidence
            + CorpusEvidence
            + PipelineEvidence
            + SlurmArtifactEvidence
            + LocalCampaignEvidence,
    > LocalReadinessBackend for T
{
}

pub use observations::*;
/// Domain supported by the local FASTQ/BAM readiness contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessDomain {
    Fastq,
    Bam,
}
