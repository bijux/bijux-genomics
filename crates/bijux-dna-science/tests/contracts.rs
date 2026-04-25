use bijux_dna_science::domain::{BindingId, ClaimId, DecisionId, EvidenceId, ScienceReleaseId, SourceId};

#[test]
fn typed_ids_require_durable_prefixed_values() {
    assert!(SourceId::parse("source.fastq.execution-support").is_ok());
    assert!(EvidenceId::parse("evidence.fastq.environment-governance").is_ok());
    assert!(ClaimId::parse("claim.fastq.defaults-are-governed").is_ok());
    assert!(DecisionId::parse("decision.fastq.environment-surface").is_ok());
    assert!(BindingId::parse("binding.fastq.environment-tool-surface").is_ok());
    assert!(ScienceReleaseId::parse("release.fastq-environment-baseline").is_ok());

    assert!(SourceId::parse("Source.fastq.execution-support").is_err());
    assert!(ClaimId::parse("claim.").is_err());
    assert!(BindingId::parse("binding.fastq..broken").is_err());
}
