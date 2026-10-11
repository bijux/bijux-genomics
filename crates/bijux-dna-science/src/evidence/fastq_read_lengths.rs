//! Independent FASTQ read-length truth derivation for benchmark acceptance.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use anyhow::{bail, Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_FASTQ_LINE_BYTES: usize = 16 * 1024 * 1024;
const MAX_SAMPLE_FILES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqInputLayout {
    SingleEnd,
    PairedEnd,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FastqReadLengthTruth {
    pub sample_id: String,
    pub input_layout: FastqInputLayout,
    pub file_count: u64,
    pub read_count: u64,
    pub minimum_read_length: u64,
    pub mean_read_length: f64,
    pub median_read_length: f64,
    pub maximum_read_length: u64,
    pub distinct_read_lengths: u64,
    pub histogram: BTreeMap<u64, u64>,
    pub exact_rows_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FastqReadLengthSummary {
    pub read_count: u64,
    pub minimum_read_length: u64,
    pub mean_read_length: f64,
    pub median_read_length: f64,
    pub maximum_read_length: u64,
    pub distinct_read_lengths: u64,
}

/// Derive exact aggregate read-length statistics from a governed histogram.
///
/// # Errors
/// Returns an error when the histogram is empty, contains zero-valued lengths or counts, or its
/// read/base counts exceed the supported integer range.
pub fn summarize_fastq_read_length_histogram(
    histogram: &BTreeMap<u64, u64>,
) -> Result<FastqReadLengthSummary> {
    if histogram.is_empty() || histogram.iter().any(|(length, count)| *length == 0 || *count == 0) {
        bail!("FASTQ read-length histogram is empty or contains zero-valued observations");
    }
    let read_count = histogram.values().try_fold(0_u64, |total, count| {
        total.checked_add(*count).context("FASTQ read count overflow")
    })?;
    let total_bases = histogram.iter().try_fold(0_u64, |total, (length, count)| {
        let bases = length.checked_mul(*count).context("FASTQ base count overflow")?;
        total.checked_add(bases).context("FASTQ base count overflow")
    })?;
    Ok(FastqReadLengthSummary {
        read_count,
        minimum_read_length: *histogram.first_key_value().context("FASTQ histogram is empty")?.0,
        mean_read_length: decimal_f64(total_bases)? / decimal_f64(read_count)?,
        median_read_length: histogram_median(histogram, read_count)?,
        maximum_read_length: *histogram.last_key_value().context("FASTQ histogram is empty")?.0,
        distinct_read_lengths: u64::try_from(histogram.len())
            .context("FASTQ histogram size overflow")?,
    })
}

/// Independently decode a sample's FASTQ files and derive exact read-length truth.
///
/// Input order is significant: exact row identity follows the declared file order and record
/// order. Plain FASTQ and multi-member gzip FASTQ are supported.
///
/// # Errors
/// Returns an error for an invalid sample/file closure, symlink or non-file input, unreadable or
/// oversized lines, malformed/truncated FASTQ framing, empty record IDs or sequences,
/// sequence/quality length disagreement, arithmetic overflow, or an empty observation set.
pub fn derive_fastq_read_length_truth(
    sample_id: &str,
    input_layout: FastqInputLayout,
    paths: &[impl AsRef<Path>],
) -> Result<FastqReadLengthTruth> {
    validate_sample(sample_id, input_layout, paths.len())?;
    let mut histogram = BTreeMap::<u64, u64>::new();
    let mut read_count = 0_u64;
    let mut exact_rows = Sha256::new();

    for path in paths {
        let path = path.as_ref();
        let mut reader = open_fastq(path)?;
        let mut record_number = 0_u64;
        while let Some(header) = read_fastq_line(&mut reader, path, "header")? {
            record_number = record_number.checked_add(1).context("FASTQ record count overflow")?;
            let sequence = required_line(&mut reader, path, record_number, "sequence")?;
            let plus = required_line(&mut reader, path, record_number, "separator")?;
            let quality = required_line(&mut reader, path, record_number, "quality")?;
            let record_id = record_id(&header, path, record_number)?;
            if plus.first() != Some(&b'+') {
                bail!("FASTQ separator is invalid at {} record {record_number}", path.display());
            }
            if sequence.is_empty() {
                bail!("FASTQ sequence is empty at {} record {record_number}", path.display());
            }
            if sequence.len() != quality.len() {
                bail!(
                    "FASTQ sequence and quality lengths differ at {} record {record_number}",
                    path.display()
                );
            }
            let length = u64::try_from(sequence.len()).context("FASTQ sequence length overflow")?;
            read_count = read_count.checked_add(1).context("FASTQ read count overflow")?;
            let count = histogram.entry(length).or_default();
            *count = count.checked_add(1).context("FASTQ histogram count overflow")?;
            exact_rows.update(record_id.as_bytes());
            exact_rows.update(b"\t");
            exact_rows.update(length.to_string().as_bytes());
            exact_rows.update(b"\n");
        }
    }
    if read_count == 0 {
        bail!("independent FASTQ read-length evidence is empty for {sample_id}");
    }
    let summary = summarize_fastq_read_length_histogram(&histogram)?;
    if summary.read_count != read_count {
        bail!("FASTQ histogram count differs from decoded records");
    }
    Ok(FastqReadLengthTruth {
        sample_id: sample_id.to_string(),
        input_layout,
        file_count: u64::try_from(paths.len()).context("FASTQ file count overflow")?,
        read_count: summary.read_count,
        minimum_read_length: summary.minimum_read_length,
        mean_read_length: summary.mean_read_length,
        median_read_length: summary.median_read_length,
        maximum_read_length: summary.maximum_read_length,
        distinct_read_lengths: summary.distinct_read_lengths,
        histogram,
        exact_rows_sha256: sha256_hex(exact_rows.finalize()),
    })
}

fn validate_sample(sample_id: &str, layout: FastqInputLayout, file_count: usize) -> Result<()> {
    if sample_id.is_empty()
        || sample_id != sample_id.trim()
        || sample_id.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        bail!("FASTQ sample ID is empty or malformed");
    }
    let expected = match layout {
        FastqInputLayout::SingleEnd => 1,
        FastqInputLayout::PairedEnd => 2,
    };
    if file_count != expected || file_count > MAX_SAMPLE_FILES {
        bail!("FASTQ file closure differs from declared input layout");
    }
    Ok(())
}

fn open_fastq(path: &Path) -> Result<Box<dyn BufRead>> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect FASTQ input {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("FASTQ input is not a regular non-symlink file: {}", path.display());
    }
    let file = File::open(path).with_context(|| format!("open FASTQ input {}", path.display()))?;
    if path.extension().and_then(|value| value.to_str()) == Some("gz") {
        Ok(Box::new(BufReader::new(MultiGzDecoder::new(file))))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

fn required_line(
    reader: &mut dyn BufRead,
    path: &Path,
    record_number: u64,
    field: &str,
) -> Result<Vec<u8>> {
    read_fastq_line(reader, path, field)?.with_context(|| {
        format!(
            "FASTQ record is truncated at {} record {record_number}: missing {field}",
            path.display()
        )
    })
}

fn read_fastq_line(reader: &mut dyn BufRead, path: &Path, field: &str) -> Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let read = reader
        .take((MAX_FASTQ_LINE_BYTES + 2) as u64)
        .read_until(b'\n', &mut line)
        .with_context(|| format!("read FASTQ {field} from {}", path.display()))?;
    if read == 0 {
        return Ok(None);
    }
    if line.len() > MAX_FASTQ_LINE_BYTES + 1
        || (line.len() > MAX_FASTQ_LINE_BYTES && line.last() != Some(&b'\n'))
    {
        bail!("FASTQ {field} exceeds the bounded line limit at {}", path.display());
    }
    if line.last() == Some(&b'\n') {
        line.pop();
    }
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    Ok(Some(line))
}

fn record_id<'a>(header: &'a [u8], path: &Path, record_number: u64) -> Result<&'a str> {
    if header.first() != Some(&b'@') {
        bail!("FASTQ header is invalid at {} record {record_number}", path.display());
    }
    let identifier = header[1..]
        .split(u8::is_ascii_whitespace)
        .next()
        .filter(|value| !value.is_empty())
        .context("FASTQ record identifier is empty")?;
    std::str::from_utf8(identifier).with_context(|| {
        format!("FASTQ record identifier is not UTF-8 at {} record {record_number}", path.display())
    })
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = digest.as_ref();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for &byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn histogram_median(histogram: &BTreeMap<u64, u64>, observations: u64) -> Result<f64> {
    let lower_rank = observations.checked_add(1).context("FASTQ median rank overflow")? / 2;
    let upper_rank = observations.checked_add(2).context("FASTQ median rank overflow")? / 2;
    let mut cumulative = 0_u64;
    let mut lower = None;
    for (&length, &count) in histogram {
        cumulative = cumulative.checked_add(count).context("FASTQ median count overflow")?;
        if lower.is_none() && cumulative >= lower_rank {
            lower = Some(length);
        }
        if cumulative >= upper_rank {
            return Ok(f64::midpoint(
                decimal_f64(lower.context("FASTQ lower median is absent")?)?,
                decimal_f64(length)?,
            ));
        }
    }
    bail!("FASTQ histogram does not contain the declared observation count")
}

fn decimal_f64(value: u64) -> Result<f64> {
    value.to_string().parse().context("convert bounded FASTQ statistic to decimal")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::summarize_fastq_read_length_histogram;

    #[test]
    fn histogram_summary_preserves_even_median_and_weighted_mean() -> anyhow::Result<()> {
        let histogram = BTreeMap::from([(10, 1), (20, 2), (40, 1)]);
        let observed = summarize_fastq_read_length_histogram(&histogram)?;
        assert_eq!(observed.read_count, 4);
        assert_eq!(observed.minimum_read_length, 10);
        assert_eq!(observed.maximum_read_length, 40);
        assert_eq!(observed.distinct_read_lengths, 3);
        assert!((observed.median_read_length - 20.0).abs() < f64::EPSILON);
        assert!((observed.mean_read_length - 22.5).abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn histogram_summary_rejects_zero_and_overflow() {
        assert!(summarize_fastq_read_length_histogram(&BTreeMap::from([(0, 1)])).is_err());
        assert!(summarize_fastq_read_length_histogram(&BTreeMap::from([(u64::MAX, 2)])).is_err());
    }
}
