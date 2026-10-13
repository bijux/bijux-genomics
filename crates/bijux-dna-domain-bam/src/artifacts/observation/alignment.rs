//! Acquisition and deterministic projection of miniature alignment observations.
use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::artifacts::alignment::{parse_tiny_alignment, TinySamDocument};
use crate::types::artifact_reports::{
    BamAlignmentTruthCigarClassV1, BamAlignmentTruthMapqClassV1, BamAlignmentTruthPositionV1,
    BamAlignmentTruthRecordV1, BamAlignmentTruthSummaryV1, BamTinyAlignmentInspectionV1,
    BAM_ALIGNMENT_TRUTH_SUMMARY_SCHEMA_VERSION,
};

/// Inspect a tiny BAM/SAM fixture and expose the normalized alignment header footprint.
///
/// # Errors
/// Returns an error when the alignment fixture cannot be parsed.
pub fn inspect_tiny_alignment(path: &Path) -> Result<BamTinyAlignmentInspectionV1> {
    Ok(project_inspection(parse_tiny_alignment(path)?))
}

/// Summarize exact tiny-alignment truth for governed BAM/SAM fixtures.
///
/// # Errors
/// Returns an error when the alignment fixture cannot be parsed.
pub fn summarize_tiny_bam_alignment_truth(input_bam: &Path) -> Result<BamAlignmentTruthSummaryV1> {
    Ok(project_truth(&parse_tiny_alignment(input_bam)?))
}

fn project_inspection(document: TinySamDocument) -> BamTinyAlignmentInspectionV1 {
    let mut header_contigs = document.references.clone();
    header_contigs.sort();
    header_contigs.dedup();

    let mut header_sample_ids = document.read_group_samples.clone();
    header_sample_ids.sort();
    header_sample_ids.dedup();

    let mut read_group_ids = document.read_groups.clone();
    read_group_ids.sort();
    read_group_ids.dedup();

    let mut mapped_record_contigs = document
        .records
        .iter()
        .filter(|record| record.is_mapped())
        .map(|record| record.rname.clone())
        .collect::<Vec<_>>();
    mapped_record_contigs.sort();
    mapped_record_contigs.dedup();

    BamTinyAlignmentInspectionV1 {
        sort_order: document.sort_order,
        header_contigs,
        header_sample_ids,
        read_group_ids,
        mapped_record_contigs,
        record_count: document.records.len() as u64,
    }
}

fn project_truth(document: &TinySamDocument) -> BamAlignmentTruthSummaryV1 {
    let total_reads = document.records.len() as u64;
    let mapped_reads = document.records.iter().filter(|record| record.is_mapped()).count() as u64;
    let unmapped_reads = total_reads.saturating_sub(mapped_reads);

    let mut mapped_contigs = document
        .records
        .iter()
        .filter(|record| record.is_mapped())
        .map(|record| record.rname.clone())
        .collect::<Vec<_>>();
    mapped_contigs.sort();
    mapped_contigs.dedup();

    let mut mapped_positions = HashMap::<(String, u64), u64>::new();
    let mut mapped_mapq_classes = HashMap::<u8, u64>::new();
    let mut cigar_classes = HashMap::<String, u64>::new();
    let mut records = document
        .records
        .iter()
        .map(|record| {
            *cigar_classes.entry(record.cigar.clone()).or_insert(0) += 1;
            if record.is_mapped() {
                *mapped_positions.entry((record.rname.clone(), record.pos)).or_insert(0) += 1;
                *mapped_mapq_classes.entry(record.mapq).or_insert(0) += 1;
            }
            BamAlignmentTruthRecordV1 {
                qname: record.qname.clone(),
                mapped: record.is_mapped(),
                reference_name: record.is_mapped().then(|| record.rname.clone()),
                position: record.is_mapped().then_some(record.pos),
                mapq: record.mapq,
                cigar: record.cigar.clone(),
            }
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.mapped
            .cmp(&right.mapped)
            .reverse()
            .then(left.reference_name.cmp(&right.reference_name))
            .then(left.position.cmp(&right.position))
            .then(left.qname.cmp(&right.qname))
            .then(left.cigar.cmp(&right.cigar))
            .then(left.mapq.cmp(&right.mapq))
    });

    let mut mapped_positions = mapped_positions
        .into_iter()
        .map(|((reference_name, position), read_count)| BamAlignmentTruthPositionV1 {
            reference_name,
            position,
            read_count,
        })
        .collect::<Vec<_>>();
    mapped_positions.sort_by(|left, right| {
        left.reference_name
            .cmp(&right.reference_name)
            .then(left.position.cmp(&right.position))
            .then(left.read_count.cmp(&right.read_count))
    });

    let mut mapped_mapq_classes = mapped_mapq_classes
        .into_iter()
        .map(|(mapq, read_count)| BamAlignmentTruthMapqClassV1 { mapq, read_count })
        .collect::<Vec<_>>();
    mapped_mapq_classes.sort_by_key(|row| row.mapq);

    let mut cigar_classes = cigar_classes
        .into_iter()
        .map(|(cigar, read_count)| BamAlignmentTruthCigarClassV1 { cigar, read_count })
        .collect::<Vec<_>>();
    cigar_classes.sort_by(|left, right| left.cigar.cmp(&right.cigar));

    BamAlignmentTruthSummaryV1 {
        schema_version: BAM_ALIGNMENT_TRUTH_SUMMARY_SCHEMA_VERSION.to_string(),
        total_reads,
        mapped_reads,
        unmapped_reads,
        mapped_contigs,
        mapped_positions,
        mapped_mapq_classes,
        cigar_classes,
        records,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "alignment/contracts.rs"]
mod contracts;
