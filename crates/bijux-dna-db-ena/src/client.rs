use crate::model::{split_ena_field, EnaQuery, EnaRecord, EnaResultKind};
use reqwest::blocking::Client;
use std::collections::HashMap;
use thiserror::Error;

const ENA_API_BASE: &str = "https://www.ebi.ac.uk/ena/portal/api/filereport";

#[derive(Debug, Error)]
pub enum EnaClientError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid ENA response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone)]
pub struct EnaClient {
    http: Client,
}

impl EnaClient {
    pub fn new(user_agent: &str) -> Result<Self, EnaClientError> {
        let http = Client::builder().user_agent(user_agent).build()?;
        Ok(Self { http })
    }

    pub fn fetch_records(&self, query: &EnaQuery) -> Result<Vec<EnaRecord>, EnaClientError> {
        let accessions = query.normalized_accessions();
        let mut out = Vec::new();
        for accession in accessions {
            let url = build_filereport_url(&accession, query.result);
            let body = self.http.get(url).send()?.error_for_status()?.text()?;
            out.extend(parse_filereport_tsv(&body, query));
        }
        Ok(out)
    }
}

#[must_use]
pub fn build_filereport_url(accession: &str, result: EnaResultKind) -> String {
    // Fields include both read_run and analysis keys so one parser supports both results.
    let fields = [
        "study_accession",
        "sample_accession",
        "experiment_accession",
        "run_accession",
        "analysis_accession",
        "analysis_type",
        "tax_id",
        "scientific_name",
        "fastq_ftp",
        "submitted_ftp",
        "sra_ftp",
        "bam_ftp",
    ]
    .join(",");

    format!(
        "{ENA_API_BASE}?accession={accession}&result={}&fields={fields}&format=tsv&download=true&limit=0",
        result.as_api_value()
    )
}

#[must_use]
pub fn parse_filereport_tsv(tsv: &str, query: &EnaQuery) -> Vec<EnaRecord> {
    let mut lines = tsv.lines();
    let Some(header_line) = lines.next() else {
        return Vec::new();
    };
    let headers: Vec<&str> = header_line.split('\t').collect();

    lines
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            let values: Vec<&str> = line.split('\t').collect();
            let mut row: HashMap<&str, &str> = HashMap::new();
            for (idx, header) in headers.iter().enumerate() {
                let value = values.get(idx).copied().unwrap_or_default();
                row.insert(header, value);
            }

            let sample_accession = row
                .get("sample_accession")
                .and_then(|v| opt_field(v))
                .map(ToString::to_string);

            if let Some(sample) = &sample_accession {
                if !query.sample_allowed(sample) {
                    return None;
                }
            }

            Some(EnaRecord {
                study_accession: row
                    .get("study_accession")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                sample_accession,
                experiment_accession: row
                    .get("experiment_accession")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                run_accession: row
                    .get("run_accession")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                analysis_accession: row
                    .get("analysis_accession")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                tax_id: row
                    .get("tax_id")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                scientific_name: row
                    .get("scientific_name")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                fastq_ftp: row
                    .get("fastq_ftp")
                    .map_or_else(Vec::new, |v| split_ena_field(v)),
                submitted_ftp: row
                    .get("submitted_ftp")
                    .map_or_else(Vec::new, |v| split_ena_field(v)),
                sra_ftp: row
                    .get("sra_ftp")
                    .map_or_else(Vec::new, |v| split_ena_field(v)),
                bam_ftp: row
                    .get("bam_ftp")
                    .map_or_else(Vec::new, |v| split_ena_field(v)),
            })
        })
        .collect()
}

fn opt_field(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
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
    }

    #[test]
    fn parse_filereport_tsv_filters_by_sample() {
        let query = EnaQuery {
            projects: vec!["PRJEBX".to_string()],
            samples: vec!["SAMEA1".to_string()],
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let tsv = "study_accession\tsample_accession\trun_accession\tfastq_ftp\nPRJEBX\tSAMEA1\tERR1\tftp.sra.ebi.ac.uk/a.fastq.gz\nPRJEBX\tSAMEA2\tERR2\tftp.sra.ebi.ac.uk/b.fastq.gz\n";
        let rows = parse_filereport_tsv(tsv, &query);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].run_accession.as_deref(), Some("ERR1"));
    }
}
