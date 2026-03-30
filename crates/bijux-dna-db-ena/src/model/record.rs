use serde::{Deserialize, Serialize};

use super::EnaSourcePreference;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaRecord {
    pub study_accession: Option<String>,
    pub sample_accession: Option<String>,
    pub experiment_accession: Option<String>,
    pub run_accession: Option<String>,
    pub analysis_accession: Option<String>,
    pub tax_id: Option<String>,
    pub scientific_name: Option<String>,
    pub library_layout: Option<String>,
    pub library_source: Option<String>,
    pub library_strategy: Option<String>,
    pub instrument_model: Option<String>,
    pub base_count: Option<u64>,
    pub read_count: Option<u64>,
    pub fastq_bytes: Vec<u64>,
    pub fastq_ftp: Vec<String>,
    pub submitted_ftp: Vec<String>,
    pub sra_ftp: Vec<String>,
    pub bam_ftp: Vec<String>,
}

impl EnaRecord {
    #[must_use]
    pub fn accession_label(&self) -> String {
        self.run_accession
            .clone()
            .or_else(|| self.analysis_accession.clone())
            .or_else(|| self.experiment_accession.clone())
            .unwrap_or_else(|| "unknown_accession".to_string())
    }

    #[must_use]
    pub fn preferred_urls(
        &self,
        source: EnaFileSource,
        preference: EnaSourcePreference,
    ) -> Vec<String> {
        let urls = match source {
            EnaFileSource::FastqFtp => &self.fastq_ftp,
            EnaFileSource::SubmittedFtp => &self.submitted_ftp,
            EnaFileSource::SraFtp => &self.sra_ftp,
            EnaFileSource::BamFtp => &self.bam_ftp,
        };
        urls.iter()
            .map(|u| normalize_url(u, preference))
            .collect::<Vec<_>>()
    }
}

#[must_use]
pub fn split_ena_field(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[must_use]
pub fn split_ena_u64_field(value: &str) -> Vec<u64> {
    value
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u64>().ok())
        .collect()
}

#[must_use]
pub fn normalize_url(raw: &str, preference: EnaSourcePreference) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("ftp://") {
        return raw.to_string();
    }
    match preference {
        EnaSourcePreference::Ftp => format!("ftp://{raw}"),
        EnaSourcePreference::Https => format!("https://{raw}"),
    }
}
