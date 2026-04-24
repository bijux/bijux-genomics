use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeSuitability {
    pub suitable: bool,
    pub reason: String,
    pub r1_mean_len: Option<usize>,
    pub r2_mean_len: Option<usize>,
    pub predicted_merge_rate: Option<f64>,
    pub probe_pairs: Option<usize>,
}

/// Assess whether paired-end reads are suitable for merging.
///
/// # Errors
/// Returns an error if inputs cannot be read.
pub fn assess_merge_suitability(r1: &Path, r2: &Path) -> Result<MergeSuitability> {
    let (r1_mean, r2_mean) = read_sequence_length_means(r1, r2, 64)?;
    let (predicted_merge_rate, probe_pairs) = estimate_overlap_rate(r1, r2, 64)?;
    let (Some(r1_len), Some(r2_len)) = (r1_mean, r2_mean) else {
        return Ok(MergeSuitability {
            suitable: false,
            reason: "missing read length samples".to_string(),
            r1_mean_len: r1_mean,
            r2_mean_len: r2_mean,
            predicted_merge_rate,
            probe_pairs,
        });
    };
    if r1_len == 0 || r2_len == 0 {
        return Ok(MergeSuitability {
            suitable: false,
            reason: "zero-length reads detected".to_string(),
            r1_mean_len: r1_mean,
            r2_mean_len: r2_mean,
            predicted_merge_rate,
            probe_pairs,
        });
    }
    if r1_len != r2_len {
        return Ok(MergeSuitability {
            suitable: false,
            reason: "read lengths differ between R1 and R2".to_string(),
            r1_mean_len: r1_mean,
            r2_mean_len: r2_mean,
            predicted_merge_rate,
            probe_pairs,
        });
    }
    let overlap_threshold = 0.05;
    let suitable = if let Some(rate) = predicted_merge_rate {
        r1_len <= 150 && rate >= overlap_threshold
    } else {
        r1_len <= 150
    };
    let reason = if let Some(rate) = predicted_merge_rate {
        if rate < overlap_threshold {
            format!("overlap probe predicts merge rate {rate:.2} < {overlap_threshold:.2}")
        } else if r1_len <= 150 {
            "read length suggests overlap is likely".to_string()
        } else {
            "read length suggests overlap is unlikely".to_string()
        }
    } else if r1_len <= 150 {
        "read length suggests overlap is likely".to_string()
    } else {
        "read length suggests overlap is unlikely".to_string()
    };
    Ok(MergeSuitability {
        suitable,
        reason,
        r1_mean_len: r1_mean,
        r2_mean_len: r2_mean,
        predicted_merge_rate,
        probe_pairs,
    })
}

fn read_sequence_length_means(
    r1: &Path,
    r2: &Path,
    max_records: usize,
) -> Result<(Option<usize>, Option<usize>)> {
    let r1_lengths =
        read_sequences(r1, max_records)?.into_iter().map(|seq| seq.len()).collect::<Vec<_>>();
    let r2_lengths =
        read_sequences(r2, max_records)?.into_iter().map(|seq| seq.len()).collect::<Vec<_>>();
    Ok((mean_length(&r1_lengths), mean_length(&r2_lengths)))
}

pub(super) fn read_sequences(path: &Path, max_records: usize) -> Result<Vec<String>> {
    let data = super::read_fastq_text(path)?;
    let mut seqs = Vec::new();
    for (idx, line) in data.lines().enumerate() {
        if idx % 4 == 1 {
            seqs.push(line.trim().to_string());
            if seqs.len() >= max_records {
                break;
            }
        }
    }
    Ok(seqs)
}

fn estimate_overlap_rate(
    r1: &Path,
    r2: &Path,
    max_pairs: usize,
) -> Result<(Option<f64>, Option<usize>)> {
    let r1_seqs = read_sequences(r1, max_pairs)?;
    let r2_seqs = read_sequences(r2, max_pairs)?;
    let pairs = r1_seqs.len().min(r2_seqs.len());
    if pairs == 0 {
        return Ok((None, None));
    }
    let mut overlaps = 0_usize;
    for idx in 0..pairs {
        let left = &r1_seqs[idx];
        let right = reverse_complement(&r2_seqs[idx]);
        if has_overlap(left, &right, 10) {
            overlaps += 1;
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let rate = overlaps as f64 / pairs as f64;
    Ok((Some(rate), Some(pairs)))
}

fn reverse_complement(seq: &str) -> String {
    seq.chars()
        .rev()
        .map(|base| match base {
            'A' | 'a' => 'T',
            'C' | 'c' => 'G',
            'G' | 'g' => 'C',
            'T' | 't' => 'A',
            'N' | 'n' => 'N',
            other => other,
        })
        .collect()
}

fn has_overlap(left: &str, right: &str, min_len: usize) -> bool {
    let max_len = left.len().min(right.len());
    for len in (min_len..=max_len).rev() {
        if left.ends_with(&right[..len]) {
            return true;
        }
    }
    false
}

fn mean_length(lengths: &[usize]) -> Option<usize> {
    if lengths.is_empty() {
        return None;
    }
    let sum: usize = lengths.iter().sum();
    Some(sum / lengths.len())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::read_sequences;
    use crate::stages::contract::runtime::inspect_headers;

    #[test]
    fn inspect_headers_reads_gzipped_fastq_inputs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-contract")?;
        let path = temp.path().join("reads.fastq.gz");
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"@read/1\nACGT\n+\n!!!!\n")?;
        let payload = encoder.finish()?;
        bijux_dna_infra::write_bytes(&path, &payload)?;

        let inspection = inspect_headers(&path, None, true)?;

        assert!(inspection.warnings.is_empty());
        assert_eq!(read_sequences(&path, 1)?, vec!["ACGT".to_string()]);
        Ok(())
    }
}
