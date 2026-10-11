use crate::model::{EnaOfflineFixture, EnaQuery, EnaRecord, EnaResultKind};
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
        query.validate()?;
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

/// # Errors
/// Returns an error when fixture schema is unsupported or JSON decoding fails.
pub fn fetch_records_from_offline_fixture(
    fixture_raw: &str,
    query: &EnaQuery,
) -> Result<Vec<EnaRecord>, EnaClientError> {
    let fixture: EnaOfflineFixture = serde_json::from_str(fixture_raw)
        .map_err(|error| EnaClientError::InvalidResponse(error.to_string()))?;
    if fixture.schema_version != "bijux.ena.offline_fixture.v1" {
        return Err(EnaClientError::InvalidResponse(format!(
            "unsupported offline fixture schema {}",
            fixture.schema_version
        )));
    }
    query.validate()?;
    Ok(fixture
        .runs
        .into_iter()
        .filter(|record| offline_record_matches_query(record, query))
        .collect())
}

fn offline_record_matches_query(record: &EnaRecord, query: &EnaQuery) -> bool {
    let samples = normalized_selector_set(&query.samples);
    if !samples.is_empty() {
        return record.sample_accession.as_deref().is_some_and(|value| samples.contains(value));
    }
    let projects = normalized_selector_set(&query.projects);
    let accessions = normalized_selector_set(&query.extra_accessions);
    record.study_accession.as_deref().is_some_and(|value| projects.contains(value))
        || [
            record.run_accession.as_deref(),
            record.analysis_accession.as_deref(),
            record.experiment_accession.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| accessions.contains(value))
}

fn normalized_selector_set(values: &[String]) -> std::collections::BTreeSet<&str> {
    values.iter().map(String::as_str).map(str::trim).filter(|value| !value.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EnaQuery;
    use anyhow::bail;

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
    fn parse_filereport_tsv_filters_by_sample() -> anyhow::Result<()> {
        let query = EnaQuery {
            projects: vec!["PRJEBX".to_string()],
            samples: vec!["SAMEA1".to_string()],
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let tsv = concat!(
            "study_accession\tsample_accession\texperiment_accession\trun_accession\t",
            "tax_id\tscientific_name\tlibrary_layout\tlibrary_source\tlibrary_strategy\t",
            "library_selection\tinstrument_platform\tinstrument_model\tbase_count\tread_count\t",
            "fastq_bytes\tfastq_ftp\tfastq_md5\tfirst_public\tlast_updated\tsubmitted_ftp\tsra_ftp\n",
            "PRJEBX\tSAMEA1\tERX1\tERR1\t9606\tHomo sapiens\tPAIRED\tGENOMIC\tWGS\tRANDOM\tILLUMINA\t",
            "Illumina NovaSeq 6000\t100\t10\t42\tftp.sra.ebi.ac.uk/a.fastq.gz\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\t2020-01-01\t2020-01-02\t",
            "ftp.sra.ebi.ac.uk/a.submitted.fastq.gz\tftp.sra.ebi.ac.uk/a.sra\n",
            "PRJEBX\tSAMEA2\tERX2\tERR2\t9606\tHomo sapiens\tPAIRED\tGENOMIC\tWGS\tRANDOM\tILLUMINA\t",
            "Illumina NovaSeq 6000\t200\t20\t84\tftp.sra.ebi.ac.uk/b.fastq.gz\tbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\t2020-01-01\t2020-01-02\t",
            "ftp.sra.ebi.ac.uk/b.submitted.fastq.gz\tftp.sra.ebi.ac.uk/b.sra\n",
        );
        let rows = parse_filereport_tsv(tsv, &query)?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].run_accession.as_deref(), Some("ERR1"));
        assert_eq!(rows[0].library_selection.as_deref(), Some("RANDOM"));
        assert_eq!(rows[0].instrument_platform.as_deref(), Some("ILLUMINA"));
        assert_eq!(rows[0].fastq_md5, ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]);
        assert_eq!(rows[0].first_public.as_deref(), Some("2020-01-01"));
        assert_eq!(rows[0].last_updated.as_deref(), Some("2020-01-02"));
        Ok(())
    }

    #[test]
    fn parse_filereport_tsv_rejects_missing_required_columns() -> anyhow::Result<()> {
        let query = EnaQuery {
            projects: vec!["PRJEBX".to_string()],
            samples: Vec::new(),
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let tsv = "study_accession\trun_accession\nPRJEBX\tERR1\n";

        let Err(error) = parse_filereport_tsv(tsv, &query) else {
            bail!("missing columns must fail");
        };
        assert!(error.to_string().contains("missing required columns: sample_accession"));
        Ok(())
    }

    #[test]
    fn fetch_records_rejects_invalid_queries_before_request() -> anyhow::Result<()> {
        let client = EnaClient::new(CRATE_USER_AGENT)?;
        let query = EnaQuery {
            projects: vec![" ".to_string()],
            samples: Vec::new(),
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };

        let Err(error) = client.fetch_records(&query) else {
            bail!("invalid queries must fail before requesting ENA");
        };

        assert!(error.to_string().contains("invalid ENA query"));
        Ok(())
    }

    #[test]
    fn offline_fixture_fetch_supports_single_paired_and_inconsistent_rows() -> anyhow::Result<()> {
        let query = EnaQuery {
            projects: vec!["PRJX".to_string()],
            samples: vec!["SAMEA1".to_string(), "SAMEA2".to_string()],
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let fixture = serde_json::json!({
            "schema_version": "bijux.ena.offline_fixture.v1",
            "runs": [
                {
                    "study_accession": "PRJX",
                    "sample_accession": "SAMEA1",
                    "experiment_accession": "ERX1",
                    "run_accession": "ERR_SINGLE",
                    "analysis_accession": null,
                    "tax_id": "9606",
                    "scientific_name": "Homo sapiens",
                    "library_layout": "SINGLE",
                    "library_source": "GENOMIC",
                    "library_strategy": "WGS",
                    "instrument_model": "NovaSeq",
                    "base_count": 100,
                    "read_count": 10,
                    "fastq_bytes": [42],
                    "fastq_ftp": ["ftp.sra.ebi.ac.uk/vol1/single.fastq.gz"],
                    "submitted_ftp": [],
                    "sra_ftp": [],
                    "bam_ftp": []
                },
                {
                    "study_accession": "PRJX",
                    "sample_accession": "SAMEA2",
                    "experiment_accession": "ERX2",
                    "run_accession": "ERR_PAIRED",
                    "analysis_accession": null,
                    "tax_id": "9606",
                    "scientific_name": "Homo sapiens",
                    "library_layout": "PAIRED",
                    "library_source": "GENOMIC",
                    "library_strategy": "WGS",
                    "instrument_model": "NovaSeq",
                    "base_count": 100,
                    "read_count": 10,
                    "fastq_bytes": [42, 43],
                    "fastq_ftp": ["ftp.sra.ebi.ac.uk/vol1/p1.fastq.gz", "ftp.sra.ebi.ac.uk/vol1/p2.fastq.gz"],
                    "submitted_ftp": [],
                    "sra_ftp": [],
                    "bam_ftp": []
                },
                {
                    "study_accession": "PRJX",
                    "sample_accession": null,
                    "experiment_accession": "ERX3",
                    "run_accession": "ERR_INCONSISTENT",
                    "analysis_accession": null,
                    "tax_id": "9606",
                    "scientific_name": "Homo sapiens",
                    "library_layout": "PAIRED",
                    "library_source": "GENOMIC",
                    "library_strategy": "WGS",
                    "instrument_model": "NovaSeq",
                    "base_count": 100,
                    "read_count": 10,
                    "fastq_bytes": [42],
                    "fastq_ftp": ["ftp.sra.ebi.ac.uk/vol1/inconsistent.fastq.gz"],
                    "submitted_ftp": [],
                    "sra_ftp": [],
                    "bam_ftp": []
                }
            ]
        });
        let rows = fetch_records_from_offline_fixture(&fixture.to_string(), &query)?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].run_accession.as_deref(), Some("ERR_SINGLE"));
        assert_eq!(rows[1].run_accession.as_deref(), Some("ERR_PAIRED"));
        Ok(())
    }

    #[test]
    fn offline_fixture_fetch_filters_exact_run_accessions() -> anyhow::Result<()> {
        let query = EnaQuery {
            projects: Vec::new(),
            samples: Vec::new(),
            extra_accessions: vec!["ERR_SELECTED".to_string()],
            result: EnaResultKind::ReadRun,
        };
        let record = |accession: &str| EnaRecord {
            study_accession: Some("PRJEB1".to_string()),
            sample_accession: Some(format!("SAMEA-{accession}")),
            experiment_accession: Some(format!("ERX-{accession}")),
            run_accession: Some(accession.to_string()),
            analysis_accession: None,
            tax_id: None,
            scientific_name: None,
            library_layout: None,
            library_source: None,
            library_strategy: None,
            library_selection: None,
            instrument_platform: None,
            instrument_model: None,
            base_count: None,
            read_count: None,
            fastq_bytes: Vec::new(),
            fastq_ftp: Vec::new(),
            fastq_md5: Vec::new(),
            first_public: None,
            last_updated: None,
            submitted_ftp: Vec::new(),
            sra_ftp: Vec::new(),
            bam_ftp: Vec::new(),
        };
        let fixture = serde_json::json!({
            "schema_version": "bijux.ena.offline_fixture.v1",
            "runs": [record("ERR_SELECTED"), record("ERR_UNRELATED")],
        });

        let rows = fetch_records_from_offline_fixture(&fixture.to_string(), &query)?;

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].run_accession.as_deref(), Some("ERR_SELECTED"));
        Ok(())
    }
}
