//! Owner: bijux-dna-domain-fastq
//! Deterministic primer-prefix normalization of admitted FASTQ record values.
//!
//! This is the native local-smoke computation, not a cutadapt implementation.
//! Primer metadata and orientation/mismatch/IUPAC controls retain their existing
//! report meaning; only `min_overlap_bp` changes prefix trimming here.

use crate::params::edna::PrimerNormalizationEffectiveParams;
use crate::types::FastqRecord;

#[cfg(test)]
#[path = "primer_normalization/contracts.rs"]
mod contracts;

const PRIMER_MOTIFS: &[&str] = &["CCTACGGG", "GACTAC"];

pub(crate) struct PrimerNormalizationRequest<'a> {
    pub r1: Vec<FastqRecord>,
    pub r2: Option<Vec<FastqRecord>>,
    pub params: &'a PrimerNormalizationEffectiveParams,
}

pub(crate) struct NormalizedPrimerReads {
    pub r1: Vec<FastqRecord>,
    pub r2: Option<Vec<FastqRecord>>,
    pub summary: PrimerNormalizationSummary,
}

#[derive(Debug, PartialEq)]
pub(crate) struct PrimerNormalizationSummary {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub primer_trimmed_reads: u64,
    pub primer_trimmed_fraction: Option<f64>,
    pub orientation_forward_fraction: Option<f64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PrimerNormalizationError {
    IncoherentMateCount { r1: usize, r2: usize },
}

impl std::fmt::Display for PrimerNormalizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncoherentMateCount { r1, r2 } => write!(
                formatter,
                "fastq.normalize_primers refused incoherent paired input: R1 count {r1} != R2 count {r2}"
            ),
        }
    }
}

impl std::error::Error for PrimerNormalizationError {}

pub(crate) fn validate_primer_read_counts(
    r1: usize,
    r2: Option<usize>,
) -> Result<(), PrimerNormalizationError> {
    if let Some(r2) = r2 {
        if r1 != r2 {
            return Err(PrimerNormalizationError::IncoherentMateCount { r1, r2 });
        }
    }
    Ok(())
}

fn trim_primer_prefix(sequence: &str, quality: &str, min_overlap: usize) -> (String, String, bool) {
    for motif in PRIMER_MOTIFS {
        if sequence.starts_with(motif) && motif.len() >= min_overlap {
            return (sequence[motif.len()..].to_string(), quality[motif.len()..].to_string(), true);
        }
    }
    (sequence.to_string(), quality.to_string(), false)
}

/// Normalize records already admitted by the FASTQ codec.
///
/// The record values retain the codec's sequence/quality validity preconditions.
/// This operation owns no path resolution, acquisition, publication or scheduling.
pub(crate) fn normalize_primer_records(
    request: PrimerNormalizationRequest<'_>,
) -> Result<NormalizedPrimerReads, PrimerNormalizationError> {
    let PrimerNormalizationRequest { mut r1, mut r2, params } = request;
    validate_primer_read_counts(r1.len(), r2.as_ref().map(Vec::len))?;
    let reads_in = (r1.len() + r2.as_ref().map_or(0, Vec::len)) as u64;
    let bases_in = r1.iter().map(|record| record.sequence.len() as u64).sum::<u64>()
        + r2.iter().flatten().map(|record| record.sequence.len() as u64).sum::<u64>();
    let mut trimmed_reads = 0_u64;
    let mut forward_oriented = 0_u64;
    for record in &mut r1 {
        let (sequence, quality, trimmed) =
            trim_primer_prefix(&record.sequence, &record.quality, params.min_overlap_bp as usize);
        if trimmed {
            trimmed_reads += 1;
            forward_oriented += 1;
            record.sequence = sequence;
            record.quality = quality;
        }
    }
    for record in r2.iter_mut().flatten() {
        let (sequence, quality, trimmed) =
            trim_primer_prefix(&record.sequence, &record.quality, params.min_overlap_bp as usize);
        if trimmed {
            trimmed_reads += 1;
            record.sequence = sequence;
            record.quality = quality;
        }
    }
    let reads_out = (r1.len() + r2.as_ref().map_or(0, Vec::len)) as u64;
    let bases_out = r1.iter().map(|record| record.sequence.len() as u64).sum::<u64>()
        + r2.iter().flatten().map(|record| record.sequence.len() as u64).sum::<u64>();
    let orientation_forward_fraction =
        if reads_in == 0 { Some(0.0) } else { Some(forward_oriented as f64 / reads_in as f64) };
    let primer_trimmed_fraction =
        if reads_in == 0 { Some(0.0) } else { Some(trimmed_reads as f64 / reads_in as f64) };
    Ok(NormalizedPrimerReads {
        r1,
        r2,
        summary: PrimerNormalizationSummary {
            reads_in,
            reads_out,
            bases_in,
            bases_out,
            primer_trimmed_reads: trimmed_reads,
            primer_trimmed_fraction,
            orientation_forward_fraction,
        },
    })
}
