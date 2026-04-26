use crate::client::EnaClientError;
use crate::model::EnaQuery;

use super::filereport_fields;
use std::collections::BTreeSet;

pub(super) fn validate_headers(headers: &[&str], query: &EnaQuery) -> Result<(), EnaClientError> {
    let mut seen = BTreeSet::new();
    let duplicate_headers =
        headers.iter().copied().filter(|header| !seen.insert(*header)).collect::<BTreeSet<_>>();

    if !duplicate_headers.is_empty() {
        return Err(EnaClientError::InvalidResponse(format!(
            "filereport response contains duplicate columns: {}",
            duplicate_headers.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }

    let missing = filereport_fields(query.result)
        .iter()
        .copied()
        .filter(|field| !headers.iter().any(|header| header == field))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return Ok(());
    }

    Err(EnaClientError::InvalidResponse(format!(
        "filereport response is missing required columns: {}",
        missing.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EnaQuery, EnaResultKind};
    use anyhow::bail;

    #[test]
    fn validate_headers_rejects_duplicate_columns() -> anyhow::Result<()> {
        let query = EnaQuery {
            projects: vec!["PRJEB1".to_string()],
            samples: Vec::new(),
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let headers = [
            "study_accession",
            "sample_accession",
            "experiment_accession",
            "run_accession",
            "run_accession",
            "fastq_ftp",
            "submitted_ftp",
            "sra_ftp",
        ];

        let Err(error) = validate_headers(&headers, &query) else {
            bail!("duplicate headers must fail");
        };

        assert!(error.to_string().contains("duplicate columns: run_accession"));
        Ok(())
    }
}
