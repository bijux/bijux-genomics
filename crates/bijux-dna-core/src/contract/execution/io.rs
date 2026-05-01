use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ids::ArtifactId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRoleFamily {
    Reads,
    Alignment,
    Variant,
    Reference,
    Index,
    Report,
    Metrics,
    Provenance,
    Evidence,
    Log,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRole {
    Reads,
    TrimmedReads,
    Bam,
    DedupBam,
    Alignment,
    Variant,
    ReportJson,
    Report,
    Log,
    Reference,
    Index,
    MetricsJson,
    MetricsEnvelope,
    Metrics,
    StageReport,
    Provenance,
    Evidence,
    SummaryJson,
    SummaryTsv,
    ReportHtml,
    #[serde(other)]
    Unknown,
}

impl ArtifactRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ArtifactRole::Reads => "reads",
            ArtifactRole::TrimmedReads => "trimmed_reads",
            ArtifactRole::Bam => "bam",
            ArtifactRole::DedupBam => "dedup_bam",
            ArtifactRole::Alignment => "alignment",
            ArtifactRole::Variant => "variant",
            ArtifactRole::ReportJson => "report_json",
            ArtifactRole::Report => "report",
            ArtifactRole::Log => "log",
            ArtifactRole::Reference => "reference",
            ArtifactRole::Index => "index",
            ArtifactRole::MetricsJson => "metrics_json",
            ArtifactRole::MetricsEnvelope => "metrics_envelope",
            ArtifactRole::Metrics => "metrics",
            ArtifactRole::StageReport => "stage_report",
            ArtifactRole::Provenance => "provenance",
            ArtifactRole::Evidence => "evidence",
            ArtifactRole::SummaryJson => "summary_json",
            ArtifactRole::SummaryTsv => "summary_tsv",
            ArtifactRole::ReportHtml => "report_html",
            ArtifactRole::Unknown => "unknown",
        }
    }

    #[must_use]
    pub const fn family(self) -> ArtifactRoleFamily {
        match self {
            ArtifactRole::Reads | ArtifactRole::TrimmedReads => ArtifactRoleFamily::Reads,
            ArtifactRole::Bam | ArtifactRole::DedupBam | ArtifactRole::Alignment => {
                ArtifactRoleFamily::Alignment
            }
            ArtifactRole::Variant => ArtifactRoleFamily::Variant,
            ArtifactRole::Reference => ArtifactRoleFamily::Reference,
            ArtifactRole::Index => ArtifactRoleFamily::Index,
            ArtifactRole::ReportJson
            | ArtifactRole::Report
            | ArtifactRole::StageReport
            | ArtifactRole::SummaryJson
            | ArtifactRole::SummaryTsv
            | ArtifactRole::ReportHtml => ArtifactRoleFamily::Report,
            ArtifactRole::MetricsJson | ArtifactRole::MetricsEnvelope | ArtifactRole::Metrics => {
                ArtifactRoleFamily::Metrics
            }
            ArtifactRole::Provenance => ArtifactRoleFamily::Provenance,
            ArtifactRole::Evidence => ArtifactRoleFamily::Evidence,
            ArtifactRole::Log => ArtifactRoleFamily::Log,
            ArtifactRole::Unknown => ArtifactRoleFamily::Other,
        }
    }

    #[must_use]
    pub const fn is_typed(self) -> bool {
        !matches!(self, ArtifactRole::Unknown)
    }

    #[must_use]
    pub fn from_port_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "reads" | "reads_in" | "reads_out" | "reads_r1" | "reads_r2" | "validated"
            | "validated_reads" | "filtered_reads" | "corrected_reads" | "merged_reads"
            | "unmerged_reads" | "sample_reads" => Some(Self::Reads),
            "trimmed_reads" | "trimmed_reads_r1" | "trimmed_reads_r2" => Some(Self::TrimmedReads),
            "bam" | "aligned_bam" | "filtered_bam" => Some(Self::Bam),
            "dedup_bam" | "markeddup_bam" => Some(Self::DedupBam),
            "alignment" => Some(Self::Alignment),
            "vcf" | "variants" | "gl_vcf" => Some(Self::Variant),
            "reference" | "reference_fasta" | "host_reference_bundle" => Some(Self::Reference),
            "index" | "bam_index" | "reference_index" => Some(Self::Index),
            "report" => Some(Self::Report),
            "report_json"
            | "filter_report_json"
            | "validation_report"
            | "coverage_report"
            | "damage_report"
            | "classification_report_json" => Some(Self::ReportJson),
            "metrics" => Some(Self::Metrics),
            "metrics_json" => Some(Self::MetricsJson),
            "metrics_envelope" => Some(Self::MetricsEnvelope),
            "stage_report" => Some(Self::StageReport),
            "provenance" | "gl_provenance" | "lineage_manifest" => Some(Self::Provenance),
            "evidence" | "evidence_bundle" => Some(Self::Evidence),
            "summary_json" => Some(Self::SummaryJson),
            "summary_tsv" | "screen_report_tsv" | "rrna_report_tsv" => Some(Self::SummaryTsv),
            "report_html" | "multiqc_report" => Some(Self::ReportHtml),
            "log" | "raw_backend_report_txt" => Some(Self::Log),
            _ => None,
        }
    }
}

impl Default for ArtifactRole {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactSpec {
    pub name: ArtifactId,
    pub path: PathBuf,
    pub role: ArtifactRole,
    #[serde(default)]
    pub optional: bool,
}

impl ArtifactSpec {
    #[must_use]
    pub fn required(name: ArtifactId, path: PathBuf, role: ArtifactRole) -> Self {
        Self { name, path, role, optional: false }
    }

    #[must_use]
    pub fn optional(name: ArtifactId, path: PathBuf, role: ArtifactRole) -> Self {
        Self { name, path, role, optional: true }
    }
}

pub type ArtifactRef = ArtifactSpec;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageIO {
    pub inputs: Vec<ArtifactSpec>,
    pub outputs: Vec<ArtifactSpec>,
}
