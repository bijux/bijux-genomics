use crate::model::{split_ena_field, split_ena_u64_field, EnaQuery, EnaRecord};
use std::collections::HashMap;

use super::{filereport, EnaClientError};

pub(super) fn parse_filereport_tsv(
    tsv: &str,
    query: &EnaQuery,
) -> Result<Vec<EnaRecord>, EnaClientError> {
    let mut lines = tsv.lines();
    let Some(header_line) = lines.next() else {
        return Err(EnaClientError::InvalidResponse(
            "filereport response is empty".to_string(),
        ));
    };
    let headers: Vec<&str> = header_line.split('\t').collect();
    validate_headers(&headers, query)?;

    Ok(lines
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
                library_layout: row
                    .get("library_layout")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                library_source: row
                    .get("library_source")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                library_strategy: row
                    .get("library_strategy")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                instrument_model: row
                    .get("instrument_model")
                    .and_then(|v| opt_field(v))
                    .map(ToString::to_string),
                base_count: row
                    .get("base_count")
                    .and_then(|v| opt_field(v))
                    .and_then(|v| v.parse::<u64>().ok()),
                read_count: row
                    .get("read_count")
                    .and_then(|v| opt_field(v))
                    .and_then(|v| v.parse::<u64>().ok()),
                fastq_bytes: row
                    .get("fastq_bytes")
                    .map_or_else(Vec::new, |v| split_ena_u64_field(v)),
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
        .collect())
}

fn opt_field(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn validate_headers(headers: &[&str], query: &EnaQuery) -> Result<(), EnaClientError> {
    let missing = filereport::filereport_fields(query.result)
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
