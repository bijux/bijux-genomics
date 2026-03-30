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
