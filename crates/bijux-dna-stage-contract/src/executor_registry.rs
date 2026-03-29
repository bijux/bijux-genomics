//! Code-backed stage executor/readiness registry.

/// CI/governance readiness badge for a stage executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessBadge {
    /// Implemented but not admitted to stable profiles.
    Experimental,
    /// Implemented and admitted to stable profiles.
    Supported,
    /// Implemented, validated, and locked for high-assurance profiles.
    Certified,
}

/// Domain label for stage ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageDomain {
    /// FASTQ domain.
    Fastq,
    /// BAM domain.
    Bam,
    /// VCF domain.
    Vcf,
}

/// Code-backed executor entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageExecutorEntry {
    /// Stage identifier (`domain.verb`).
    pub stage_id: &'static str,
    /// Canonical executor binding label.
    pub executor: &'static str,
    /// Owning domain.
    pub domain: StageDomain,
    /// Readiness badge.
    pub readiness: ReadinessBadge,
}
