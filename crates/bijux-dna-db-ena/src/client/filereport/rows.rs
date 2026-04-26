use crate::client::EnaClientError;
use crate::model::{split_ena_field, split_ena_u64_field, EnaQuery, EnaRecord};

use super::headers;

pub(crate) fn parse_filereport_tsv(
    tsv: &str,
    query: &EnaQuery,
) -> Result<Vec<EnaRecord>, EnaClientError> {
    let mut lines = tsv.lines();
    let Some(header_line) = lines.next() else {
        return Err(EnaClientError::InvalidResponse("filereport response is empty".to_string()));
    };
    let headers: Vec<&str> = header_line.split('\t').collect();
    headers::validate_headers(&headers, query)?;

    let mut records = Vec::new();
    for (line_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let values: Vec<&str> = line.split('\t').collect();
        validate_row_width(line_index + 2, values.len(), headers.len())?;
        let field = |name: &str| -> &str {
            headers
                .iter()
                .position(|header| *header == name)
                .and_then(|idx| values.get(idx).copied())
                .unwrap_or_default()
        };

        let sample_accession = opt_field(field("sample_accession")).map(ToString::to_string);

        if let Some(sample) = &sample_accession {
            if !query.sample_allowed(sample) {
                continue;
            }
        }

        records.push(EnaRecord {
            study_accession: opt_field(field("study_accession")).map(ToString::to_string),
            sample_accession,
            experiment_accession: opt_field(field("experiment_accession")).map(ToString::to_string),
            run_accession: opt_field(field("run_accession")).map(ToString::to_string),
            analysis_accession: opt_field(field("analysis_accession")).map(ToString::to_string),
            tax_id: opt_field(field("tax_id")).map(ToString::to_string),
            scientific_name: opt_field(field("scientific_name")).map(ToString::to_string),
            library_layout: opt_field(field("library_layout")).map(ToString::to_string),
            library_source: opt_field(field("library_source")).map(ToString::to_string),
            library_strategy: opt_field(field("library_strategy")).map(ToString::to_string),
            instrument_model: opt_field(field("instrument_model")).map(ToString::to_string),
            base_count: opt_field(field("base_count")).and_then(|v| v.parse::<u64>().ok()),
            read_count: opt_field(field("read_count")).and_then(|v| v.parse::<u64>().ok()),
            fastq_bytes: split_ena_u64_field(field("fastq_bytes")),
            fastq_ftp: split_ena_field(field("fastq_ftp")),
            submitted_ftp: split_ena_field(field("submitted_ftp")),
            sra_ftp: split_ena_field(field("sra_ftp")),
            bam_ftp: split_ena_field(field("bam_ftp")),
        });
    }

    Ok(records)
}

fn validate_row_width(
    line_number: usize,
    value_count: usize,
    header_count: usize,
) -> Result<(), EnaClientError> {
    if value_count == header_count {
        return Ok(());
    }

    Err(EnaClientError::InvalidResponse(format!(
        "filereport row {line_number} has {value_count} columns, expected {header_count}"
    )))
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
    use crate::model::{EnaQuery, EnaResultKind};
    use anyhow::bail;

    #[test]
    fn parse_filereport_tsv_rejects_short_rows() -> anyhow::Result<()> {
        let query = EnaQuery {
            projects: vec!["PRJEB1".to_string()],
            samples: Vec::new(),
            extra_accessions: Vec::new(),
            result: EnaResultKind::ReadRun,
        };
        let tsv = concat!(
            "study_accession\tsample_accession\texperiment_accession\trun_accession\t",
            "tax_id\tscientific_name\tlibrary_layout\tlibrary_source\tlibrary_strategy\t",
            "instrument_model\tbase_count\tread_count\tfastq_bytes\tfastq_ftp\t",
            "submitted_ftp\tsra_ftp\n",
            "PRJEB1\tSAMEA1\tERX1\tERR1\n",
        );

        let Err(error) = parse_filereport_tsv(tsv, &query) else {
            bail!("short rows must fail");
        };

        assert!(error.to_string().contains("row 2 has 4 columns"));
        Ok(())
    }
}
