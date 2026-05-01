use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Domain {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StabilityTier {
    Stable,
    Beta,
    Experimental,
}

impl StabilityTier {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Experimental => "experimental",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArtifactType {
    FastqReads,
    ReferenceFasta,
    Bam,
    Variant,
    SampleSheet,
    ReportJson,
    RunManifestJson,
    StageSummariesJson,
    MetricsBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MetricsBundle {
    FastqCore,
    BamCore,
    BamAdna,
    VcfCore,
    CrossHandoff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReportSection {
    Fastq,
    Bam,
    Vcf,
    Cross,
    Handoff,
    PipelineDefaults,
}
