//! Owner: bijux-dna-bench-model
//! Validation for persisted FASTQ read-length expectation evidence.

use crate::diagnostics::BenchError;
use crate::model::{BenchmarkFastqLayout, FastqReadLengthExpectations};

use super::FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1;

/// Validate the complete persisted FASTQ read-length expectation contract.
///
/// # Errors
/// Returns an error for malformed identities, digests, sample ordering, closure counts, numeric
/// statistics, or artifact paths.
pub fn validate_fastq_read_length_expectations(
    expectations: &FastqReadLengthExpectations,
) -> Result<(), BenchError> {
    if expectations.schema_version != FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1 {
        return Err(invalid("FASTQ read-length expectation schema mismatch"));
    }
    validate_identifier(&expectations.corpus_id, "corpus")?;
    validate_digest(&expectations.data_index_sha256, "data index")?;
    validate_digest(&expectations.catalog_sha256, "catalog")?;
    let sample_count = u64::try_from(expectations.samples.len())
        .map_err(|_| invalid("FASTQ expectation sample count overflows"))?;
    if expectations.samples.is_empty() || expectations.sample_count != sample_count {
        return Err(invalid("FASTQ read-length expectation sample closure differs"));
    }

    let mut previous = None;
    let mut file_count = 0_u64;
    for sample in &expectations.samples {
        validate_identifier(&sample.sample_id, "sample")?;
        if previous.is_some_and(|value: &str| value >= sample.sample_id.as_str()) {
            return Err(invalid("FASTQ expectation samples are not strictly ordered"));
        }
        previous = Some(sample.sample_id.as_str());
        let expected_files = match sample.input_layout {
            BenchmarkFastqLayout::SingleEnd => 1,
            BenchmarkFastqLayout::PairedEnd => 2,
        };
        if sample.file_count != expected_files {
            return Err(invalid("FASTQ expectation layout and file closure differ"));
        }
        file_count = file_count
            .checked_add(sample.file_count)
            .ok_or_else(|| invalid("FASTQ expectation file count overflows"))?;
        validate_statistics(sample)?;
        let expected_summary = format!("summaries/{}.summary.tsv", sample.sample_id);
        let expected_histogram = format!("histograms/{}.histogram.tsv", sample.sample_id);
        if sample.summary_path != expected_summary || sample.histogram_path != expected_histogram {
            return Err(invalid("FASTQ expectation artifact path is not canonical"));
        }
        for (digest, label) in [
            (&sample.summary_sha256, "summary"),
            (&sample.histogram_sha256, "histogram"),
            (&sample.exact_rows_sha256, "exact rows"),
        ] {
            validate_digest(digest, label)?;
        }
    }
    if expectations.file_count != file_count {
        return Err(invalid("FASTQ read-length expectation file closure differs"));
    }
    Ok(())
}

fn validate_statistics(
    sample: &crate::model::FastqReadLengthExpectationSample,
) -> Result<(), BenchError> {
    let minimum = decimal_f64(sample.minimum_read_length)?;
    let maximum = decimal_f64(sample.maximum_read_length)?;
    if sample.read_count == 0
        || sample.minimum_read_length == 0
        || sample.minimum_read_length > sample.maximum_read_length
        || !sample.mean_read_length.is_finite()
        || !sample.median_read_length.is_finite()
        || sample.mean_read_length < minimum
        || sample.mean_read_length > maximum
        || sample.median_read_length < minimum
        || sample.median_read_length > maximum
        || sample.distinct_read_lengths == 0
        || sample.distinct_read_lengths > sample.read_count
    {
        return Err(invalid("FASTQ read-length expectation statistics are invalid"));
    }
    Ok(())
}

fn decimal_f64(value: u64) -> Result<f64, BenchError> {
    value.to_string().parse().map_err(|_| invalid("FASTQ expectation statistic is not decimal"))
}

fn validate_identifier(value: &str, label: &str) -> Result<(), BenchError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(invalid(&format!("FASTQ expectation {label} identity is malformed")));
    }
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid(&format!("FASTQ expectation {label} digest is malformed")));
    }
    Ok(())
}

fn invalid(message: &str) -> BenchError {
    BenchError::InvalidPolicy(message.to_string())
}
