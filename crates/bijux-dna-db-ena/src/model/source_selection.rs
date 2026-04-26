use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnaResultKind {
    ReadRun,
    Analysis,
}

impl EnaResultKind {
    #[must_use]
    pub fn as_api_value(self) -> &'static str {
        match self {
            Self::ReadRun => "read_run",
            Self::Analysis => "analysis",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnaSourcePreference {
    Ftp,
    Https,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnaFileSource {
    FastqFtp,
    SubmittedFtp,
    SraFtp,
    BamFtp,
}

impl EnaFileSource {
    #[must_use]
    pub fn field_name(self) -> &'static str {
        match self {
            Self::FastqFtp => "fastq_ftp",
            Self::SubmittedFtp => "submitted_ftp",
            Self::SraFtp => "sra_ftp",
            Self::BamFtp => "bam_ftp",
        }
    }
}

#[must_use]
pub fn normalize_url(raw: &str, preference: EnaSourcePreference) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("ftp://")
    {
        return trimmed.to_string();
    }
    match preference {
        EnaSourcePreference::Ftp => format!("ftp://{trimmed}"),
        EnaSourcePreference::Https => format!("https://{trimmed}"),
    }
}
