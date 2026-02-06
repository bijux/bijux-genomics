use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ids::ArtifactId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRole {
    Reads,
    TrimmedReads,
    Bam,
    DedupBam,
    ReportJson,
    Log,
    Index,
    MetricsJson,
    MetricsEnvelope,
    StageReport,
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
            ArtifactRole::ReportJson => "report_json",
            ArtifactRole::Log => "log",
            ArtifactRole::Index => "index",
            ArtifactRole::MetricsJson => "metrics_json",
            ArtifactRole::MetricsEnvelope => "metrics_envelope",
            ArtifactRole::StageReport => "stage_report",
            ArtifactRole::SummaryJson => "summary_json",
            ArtifactRole::SummaryTsv => "summary_tsv",
            ArtifactRole::ReportHtml => "report_html",
            ArtifactRole::Unknown => "unknown",
        }
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
        Self {
            name,
            path,
            role,
            optional: false,
        }
    }

    #[must_use]
    pub fn optional(name: ArtifactId, path: PathBuf, role: ArtifactRole) -> Self {
        Self {
            name,
            path,
            role,
            optional: true,
        }
    }
}

pub type ArtifactRef = ArtifactSpec;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageIO {
    pub inputs: Vec<ArtifactSpec>,
    pub outputs: Vec<ArtifactSpec>,
}
