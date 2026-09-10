//! Owner: bijux-dna-bench-workflows
//! Ancient DNA observations truth.
use super::super::{
    anyhow, bail, read_fastq_records, read_named_reference_contig, BTreeMap, DamageMetricsV1,
    FastqRecord, Path, Result, SyntheticReadPlacement,
};

pub(crate) fn trim_terminal_damage_record(record: &FastqRecord) -> Result<FastqRecord> {
    if record.sequence.len() <= 2 || record.quality.len() <= 2 {
        bail!("terminal damage trim requires reads longer than two bases");
    }
    Ok(FastqRecord {
        header: record.header.clone(),
        sequence: record.sequence[1..record.sequence.len() - 1].to_string(),
        plus: record.plus.clone(),
        quality: record.quality[1..record.quality.len() - 1].to_string(),
    })
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Approximate normalized metrics retain the existing floating-point calculation; integer counts remain separate"
)]
pub(crate) fn derive_damage_metrics(
    reference_fasta: &Path,
    reference_name: &str,
    trimmed_r1: &Path,
    trimmed_r2: &Path,
    placements: &BTreeMap<String, SyntheticReadPlacement>,
) -> Result<DamageMetricsV1> {
    let (_, reference_sequence) = read_named_reference_contig(reference_fasta, reference_name)?;
    let mut records = read_fastq_records(trimmed_r1)?;
    records.extend(read_fastq_records(trimmed_r2)?);
    let mut ct_events = 0_u64;
    let mut ga_events = 0_u64;
    let mut eligible_5p = 0_u64;
    let mut eligible_3p = 0_u64;
    for record in records {
        let qname = record.header.trim_start_matches('@');
        let placement = placements
            .get(qname)
            .ok_or_else(|| anyhow!("missing synthetic placement for `{qname}`"))?;
        let start_index = usize::try_from(placement.position.saturating_sub(1)).unwrap_or(0) + 1;
        let end_index = start_index + record.sequence.len() - 1;
        let ref_start = reference_sequence
            .as_bytes()
            .get(start_index)
            .copied()
            .ok_or_else(|| anyhow!("missing reference start base for `{qname}`"))?;
        let ref_end = reference_sequence
            .as_bytes()
            .get(end_index)
            .copied()
            .ok_or_else(|| anyhow!("missing reference end base for `{qname}`"))?;
        let read_start = record.sequence.as_bytes()[0].to_ascii_uppercase();
        let read_end = record.sequence.as_bytes()[record.sequence.len() - 1].to_ascii_uppercase();
        if ref_start == b'C' {
            eligible_5p += 1;
            if read_start == b'T' {
                ct_events += 1;
            }
        }
        if ref_end == b'G' {
            eligible_3p += 1;
            if read_end == b'A' {
                ga_events += 1;
            }
        }
    }
    Ok(DamageMetricsV1 {
        c_to_t_5p: if eligible_5p == 0 { 0.0 } else { ct_events as f64 / eligible_5p as f64 },
        g_to_a_3p: if eligible_3p == 0 { 0.0 } else { ga_events as f64 / eligible_3p as f64 },
        pmd_score_histogram: vec![(0, 1), (3, ct_events.max(ga_events))],
    })
}

pub(crate) fn count_json_u64(value: &serde_json::Value, pointer: &str) -> u64 {
    value.pointer(pointer).and_then(serde_json::Value::as_u64).unwrap_or(0)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Approximate normalized metrics retain the existing floating-point calculation; integer counts remain separate"
)]
pub(crate) fn terminal_transition_asymmetry(left: &[FastqRecord], right: &[FastqRecord]) -> f64 {
    let ct = left.iter().filter(|record| record.sequence.starts_with('T')).count() as f64;
    let ga = right.iter().filter(|record| record.sequence.ends_with('A')).count() as f64;
    if (ct + ga).abs() < f64::EPSILON {
        0.0
    } else {
        (ct - ga).abs() / (ct + ga)
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Approximate normalized metrics retain the existing floating-point calculation; integer counts remain separate"
)]
pub(crate) fn terminal_transition_asymmetry_one_end(
    records: &[FastqRecord],
    five_prime: bool,
) -> f64 {
    let target = if five_prime {
        records.iter().filter(|record| record.sequence.starts_with('T')).count() as f64
    } else {
        records.iter().filter(|record| record.sequence.ends_with('A')).count() as f64
    };
    if records.is_empty() {
        0.0
    } else {
        target / records.len() as f64
    }
}

pub(crate) fn first_base_histogram(records: &[FastqRecord]) -> BTreeMap<String, u64> {
    let mut histogram = BTreeMap::<String, u64>::new();
    for record in records {
        if let Some(base) = record.sequence.chars().next() {
            *histogram.entry(base.to_string()).or_insert(0) += 1;
        }
    }
    histogram
}

pub(crate) fn last_base_histogram(records: &[FastqRecord]) -> BTreeMap<String, u64> {
    let mut histogram = BTreeMap::<String, u64>::new();
    for record in records {
        if let Some(base) = record.sequence.chars().last() {
            *histogram.entry(base.to_string()).or_insert(0) += 1;
        }
    }
    histogram
}
