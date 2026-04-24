use crate::model::{EnaQuery, EnaRecord, EnaResultKind};
use reqwest::blocking::Client;

mod error;
mod filereport;

pub use error::EnaClientError;

pub const CRATE_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct EnaClient {
    http: Client,
}

impl EnaClient {
    /// # Errors
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn from_crate_identity() -> Result<Self, EnaClientError> {
        Self::new(CRATE_USER_AGENT)
    }

    /// # Errors
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn new(user_agent: &str) -> Result<Self, EnaClientError> {
        let http = Client::builder().user_agent(user_agent).build()?;
        Ok(Self { http })
    }

    /// # Errors
    /// Returns an error if any ENA request fails or response decoding fails.
    pub fn fetch_records(&self, query: &EnaQuery) -> Result<Vec<EnaRecord>, EnaClientError> {
        let accessions = query.normalized_accessions();
        let mut out = Vec::new();
        for accession in accessions {
            let url = filereport::build_filereport_url(&accession, query.result);
            let body = self.http.get(url).send()?.error_for_status()?.text()?;
            out.extend(filereport::parse_filereport_tsv(&body, query)?);
        }
        Ok(out)
    }
}

#[must_use]
pub fn build_filereport_url(accession: &str, result: EnaResultKind) -> String {
    filereport::build_filereport_url(accession, result)
}

/// # Errors
/// Returns an error when the filereport payload is empty or missing required columns.
pub fn parse_filereport_tsv(tsv: &str, query: &EnaQuery) -> Result<Vec<EnaRecord>, EnaClientError> {
    filereport::parse_filereport_tsv(tsv, query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EnaQuery;

    #[test]
    fn build_filereport_url_contains_expected_query() {
        let url = build_filereport_url("PRJEB22390", EnaResultKind::ReadRun);
        assert!(url.contains("accession=PRJEB22390"));
        assert!(url.contains("result=read_run"));
        assert!(url.contains("fields=study_accession"));
        assert!(url.contains("run_accession"));
        assert!(!url.contains("analysis_accession"));
    }

    #[test]
    fn build_analysis_filereport_url_uses_analysis_fields() {
        let url = build_filereport_url("ERZ123456", EnaResultKind::Analysis);
        assert!(url.contains("result=analysis"));
        assert!(url.contains("analysis_accession"));
        assert!(!url.contains("run_accession"));
    }

    #[test]
    fn parse_filereport_tsv_filters_by_sample() {
        let query = EnaQuery {
            projects: vec!["PRJEBX".to_string()],
            samples: vec!["SAMEA1".to_string()],
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let tsv = concat!(
            "study_accession\tsample_accession\texperiment_accession\trun_accession\t",
            "tax_id\tscientific_name\tlibrary_layout\tlibrary_source\tlibrary_strategy\t",
            "instrument_model\tbase_count\tread_count\tfastq_bytes\tfastq_ftp\t",
            "submitted_ftp\tsra_ftp\n",
            "PRJEBX\tSAMEA1\tERX1\tERR1\t9606\tHomo sapiens\tPAIRED\tGENOMIC\tWGS\t",
            "Illumina NovaSeq 6000\t100\t10\t42\tftp.sra.ebi.ac.uk/a.fastq.gz\t",
            "ftp.sra.ebi.ac.uk/a.submitted.fastq.gz\tftp.sra.ebi.ac.uk/a.sra\n",
            "PRJEBX\tSAMEA2\tERX2\tERR2\t9606\tHomo sapiens\tPAIRED\tGENOMIC\tWGS\t",
            "Illumina NovaSeq 6000\t200\t20\t84\tftp.sra.ebi.ac.uk/b.fastq.gz\t",
            "ftp.sra.ebi.ac.uk/b.submitted.fastq.gz\tftp.sra.ebi.ac.uk/b.sra\n",
        );
        let rows = parse_filereport_tsv(tsv, &query).expect("parse filtered rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].run_accession.as_deref(), Some("ERR1"));
    }

    #[test]
    fn parse_filereport_tsv_rejects_missing_required_columns() {
        let query = EnaQuery {
            projects: vec!["PRJEBX".to_string()],
            samples: Vec::new(),
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let tsv = "study_accession\trun_accession\nPRJEBX\tERR1\n";

        let error = parse_filereport_tsv(tsv, &query).expect_err("missing columns must fail");
        assert!(error.to_string().contains("missing required columns: sample_accession"));
    }
}
