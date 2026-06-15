use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastqRecord {
    pub header: String,
    pub sequence: String,
    pub plus: String,
    pub quality: String,
}

impl FastqRecord {
    #[must_use]
    pub fn base_name_and_mate(&self) -> (String, Option<u8>) {
        parse_header_pairing(self.header.trim_start_matches('@'))
    }
}

/// Read FASTQ records from plain or gzipped input.
///
/// # Errors
/// Returns an error when input is malformed.
pub fn read_fastq_records(path: &Path) -> Result<Vec<FastqRecord>> {
    let reader: Box<dyn BufRead> = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let file = std::fs::File::open(path)?;
        let decoder = flate2::read::MultiGzDecoder::new(file);
        Box::new(BufReader::new(decoder))
    } else {
        Box::new(BufReader::new(std::fs::File::open(path)?))
    };

    let mut lines = reader.lines();
    let mut records = Vec::new();
    let mut line_no = 0_usize;

    loop {
        let Some(header) = lines.next() else {
            break;
        };
        line_no += 1;
        let header = header?;
        let Some(sequence) = lines.next() else {
            return Err(anyhow!("truncated FASTQ at {} line {}", path.display(), line_no));
        };
        line_no += 1;
        let sequence = sequence?;
        let Some(plus) = lines.next() else {
            return Err(anyhow!("truncated FASTQ at {} line {}", path.display(), line_no));
        };
        line_no += 1;
        let plus = plus?;
        let Some(quality) = lines.next() else {
            return Err(anyhow!("truncated FASTQ at {} line {}", path.display(), line_no));
        };
        line_no += 1;
        let quality = quality?;

        if !header.starts_with('@') {
            return Err(anyhow!("invalid FASTQ header at {} line {}", path.display(), line_no - 3));
        }
        if !plus.starts_with('+') {
            return Err(anyhow!(
                "invalid FASTQ plus line at {} line {}",
                path.display(),
                line_no - 1
            ));
        }
        if sequence.len() != quality.len() {
            return Err(anyhow!(
                "sequence/quality length mismatch at {} line {}",
                path.display(),
                line_no
            ));
        }

        records.push(FastqRecord { header, sequence, plus, quality });
    }

    Ok(records)
}

/// Write FASTQ records to plain or gzipped output, preserving record order.
///
/// # Errors
/// Returns an error when output cannot be written.
pub fn write_fastq_records(path: &Path, records: &[FastqRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let file = std::fs::File::create(path)?;
        let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        for record in records {
            writeln!(encoder, "{}", record.header)?;
            writeln!(encoder, "{}", record.sequence)?;
            writeln!(encoder, "{}", record.plus)?;
            writeln!(encoder, "{}", record.quality)?;
        }
        encoder.finish()?;
    } else {
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        for record in records {
            writeln!(writer, "{}", record.header)?;
            writeln!(writer, "{}", record.sequence)?;
            writeln!(writer, "{}", record.plus)?;
            writeln!(writer, "{}", record.quality)?;
        }
        writer.flush()?;
    }

    Ok(())
}

/// Compare FASTQ inputs and outputs record-by-record to count changed versus unchanged reads.
///
/// Sequence and quality determine whether a read changed. Header or plus-line drift alone does
/// not count as read correction because downstream benchmark rows are meant to reflect content
/// changes, not wrapper-level formatting differences.
///
/// # Errors
/// Returns an error when either FASTQ input is malformed.
pub fn count_changed_fastq_reads(path_before: &Path, path_after: &Path) -> Result<(u64, u64)> {
    let before = read_fastq_records(path_before)?;
    let after = read_fastq_records(path_after)?;

    let shared_len = before.len().min(after.len());
    let mut changed_reads = 0_u64;
    let mut unchanged_reads = 0_u64;

    for (before_record, after_record) in before.iter().zip(after.iter()) {
        if before_record.sequence == after_record.sequence
            && before_record.quality == after_record.quality
        {
            unchanged_reads += 1;
        } else {
            changed_reads += 1;
        }
    }

    let unmatched_reads = before.len().max(after.len()) - shared_len;
    changed_reads += unmatched_reads as u64;

    Ok((changed_reads, unchanged_reads))
}

#[must_use]
pub fn parse_header_pairing(header: &str) -> (String, Option<u8>) {
    let token = header.split_whitespace().next().unwrap_or(header);
    if let Some(base) = token.strip_suffix("/1") {
        return (base.to_string(), Some(1));
    }
    if let Some(base) = token.strip_suffix("/2") {
        return (base.to_string(), Some(2));
    }

    let mut fields = header.split_whitespace();
    let first = fields.next().unwrap_or(header);
    if let Some(descriptor) = fields.next() {
        if descriptor.starts_with("1:") {
            return (first.to_string(), Some(1));
        }
        if descriptor.starts_with("2:") {
            return (first.to_string(), Some(2));
        }
    }

    (first.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::{count_changed_fastq_reads, read_fastq_records, write_fastq_records, FastqRecord};

    #[test]
    fn fastq_io_round_trip_gz() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-io")?;
        let path = temp.path().join("reads.fastq.gz");
        let records = vec![
            FastqRecord {
                header: "@read1/1".to_string(),
                sequence: "ACGT".to_string(),
                plus: "+".to_string(),
                quality: "!!!!".to_string(),
            },
            FastqRecord {
                header: "@read1/2".to_string(),
                sequence: "TGCA".to_string(),
                plus: "+".to_string(),
                quality: "####".to_string(),
            },
        ];

        write_fastq_records(&path, &records)?;
        let decoded = read_fastq_records(&path)?;
        assert_eq!(decoded, records);
        Ok(())
    }

    #[test]
    fn count_changed_fastq_reads_tracks_sequence_and_quality_deltas() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-change-count")?;
        let before = temp.path().join("before.fastq");
        let after = temp.path().join("after.fastq");

        write_fastq_records(
            &before,
            &[
                FastqRecord {
                    header: "@read1".to_string(),
                    sequence: "ACGT".to_string(),
                    plus: "+".to_string(),
                    quality: "!!!!".to_string(),
                },
                FastqRecord {
                    header: "@read2".to_string(),
                    sequence: "TGCA".to_string(),
                    plus: "+".to_string(),
                    quality: "####".to_string(),
                },
                FastqRecord {
                    header: "@read3".to_string(),
                    sequence: "CCCC".to_string(),
                    plus: "+".to_string(),
                    quality: "$$$$".to_string(),
                },
            ],
        )?;
        write_fastq_records(
            &after,
            &[
                FastqRecord {
                    header: "@read1 renamed".to_string(),
                    sequence: "ACGT".to_string(),
                    plus: "+renamed".to_string(),
                    quality: "!!!!".to_string(),
                },
                FastqRecord {
                    header: "@read2".to_string(),
                    sequence: "TGCT".to_string(),
                    plus: "+".to_string(),
                    quality: "####".to_string(),
                },
            ],
        )?;

        let (changed_reads, unchanged_reads) = count_changed_fastq_reads(&before, &after)?;
        assert_eq!(changed_reads, 2);
        assert_eq!(unchanged_reads, 1);
        Ok(())
    }
}
