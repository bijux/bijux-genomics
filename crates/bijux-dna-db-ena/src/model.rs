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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaQuery {
    pub projects: Vec<String>,
    pub samples: Vec<String>,
    pub extra_accessions: Vec<String>,
    pub result: EnaResultKind,
}

impl EnaQuery {
    #[must_use]
    pub fn normalized_accessions(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(self.projects.iter().cloned());
        all.extend(self.samples.iter().cloned());
        all.extend(self.extra_accessions.iter().cloned());
        all.sort();
        all.dedup();
        all
    }

    #[must_use]
    pub fn has_sample_filter(&self) -> bool {
        !self.samples.is_empty()
    }

    #[must_use]
    pub fn sample_allowed(&self, sample_accession: &str) -> bool {
        if self.samples.is_empty() {
            return true;
        }
        self.samples.iter().any(|s| s == sample_accession)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaRunManifest {
    pub query: EnaQuery,
    pub source: EnaFileSource,
    pub preference: EnaSourcePreference,
    pub records: Vec<EnaRecord>,
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
pub fn normalize_url(raw: &str, preference: EnaSourcePreference) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("ftp://") {
        return raw.to_string();
    }
    match preference {
        EnaSourcePreference::Ftp => format!("ftp://{raw}"),
        EnaSourcePreference::Https => format!("https://{raw}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ena_field_handles_empty_and_multi_values() {
        assert_eq!(split_ena_field("a;b; ;c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn normalize_url_adds_expected_scheme() {
        assert_eq!(
            normalize_url(
                "ftp.sra.ebi.ac.uk/vol1/x.fastq.gz",
                EnaSourcePreference::Ftp
            ),
            "ftp://ftp.sra.ebi.ac.uk/vol1/x.fastq.gz"
        );
        assert_eq!(
            normalize_url(
                "ftp.sra.ebi.ac.uk/vol1/x.fastq.gz",
                EnaSourcePreference::Https
            ),
            "https://ftp.sra.ebi.ac.uk/vol1/x.fastq.gz"
        );
    }
}
