use thiserror::Error;
use serde::{Deserialize, Serialize};

use super::EnaResultKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaQuery {
    pub projects: Vec<String>,
    pub samples: Vec<String>,
    pub extra_accessions: Vec<String>,
    pub result: EnaResultKind,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum QueryValidationError {
    #[error("provide at least one of --project, --sample, or --accession")]
    MissingSelectors,
}

impl EnaQuery {
    #[must_use]
    pub fn normalized_accessions(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(normalized_values(&self.projects));
        all.extend(normalized_values(&self.samples));
        all.extend(normalized_values(&self.extra_accessions));
        all.sort();
        all.dedup();
        all
    }

    /// # Errors
    /// Returns an error when the query does not contain any usable selector.
    pub fn validate(&self) -> Result<(), QueryValidationError> {
        if self.normalized_accessions().is_empty() {
            return Err(QueryValidationError::MissingSelectors);
        }
        Ok(())
    }

    #[must_use]
    pub fn has_sample_filter(&self) -> bool {
        !normalized_values(&self.samples).is_empty()
    }

    #[must_use]
    pub fn sample_allowed(&self, sample_accession: &str) -> bool {
        let normalized_samples = normalized_values(&self.samples);
        if normalized_samples.is_empty() {
            return true;
        }
        normalized_samples
            .iter()
            .any(|sample| sample == sample_accession)
    }
}

fn normalized_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_accessions_trim_empty_and_deduplicate() {
        let query = EnaQuery {
            projects: vec![" PRJEB1 ".to_string()],
            samples: vec![String::new(), " SAMEA1 ".to_string()],
            extra_accessions: vec!["PRJEB1".to_string(), "  ".to_string()],
            result: EnaResultKind::ReadRun,
        };

        assert_eq!(
            query.normalized_accessions(),
            vec!["PRJEB1".to_string(), "SAMEA1".to_string()]
        );
    }

    #[test]
    fn validate_rejects_queries_without_usable_selectors() {
        let query = EnaQuery {
            projects: vec![" ".to_string()],
            samples: Vec::new(),
            extra_accessions: vec![String::new()],
            result: EnaResultKind::ReadRun,
        };

        assert_eq!(
            query.validate(),
            Err(QueryValidationError::MissingSelectors)
        );
    }
}
