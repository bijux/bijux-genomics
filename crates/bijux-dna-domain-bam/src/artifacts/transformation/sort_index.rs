//! Coordinate ordering and sidecar publication for miniature alignment fixtures.
//! This compatibility operation retains file effects; scientific report evaluation is separate.
use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use bijux_dna_core::prelude::ArtifactId;

use crate::artifacts::alignment::{parse_tiny_sam, write_tiny_sam_from_document, TinySamDocument};
use crate::artifacts::bam_artifact_inventory_from_outputs;
use crate::types::artifact_reports::BamArtifactInventoryV1;

/// Sort tiny SAM/BAM fixtures by coordinate and emit a deterministic index sidecar.
///
/// # Errors
/// Returns an error if input parsing or output materialization fails.
pub fn sort_and_index_tiny_bam(
    input_bam: &Path,
    output_bam: &Path,
    output_bai: &Path,
) -> Result<BamArtifactInventoryV1> {
    let mut document = parse_tiny_sam(input_bam)?;
    sort_coordinates(&mut document);
    write_tiny_sam_from_document(output_bam, &document, "coordinate")?;

    let index_payload = index_payload(&document);
    std::fs::write(output_bai, index_payload)?;

    let output_root = output_bam.parent().unwrap_or_else(|| Path::new("."));
    let outputs = vec![
        ArtifactRef::required(
            ArtifactId::from_static("sorted_bam"),
            output_bam.to_path_buf(),
            ArtifactRole::Bam,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("sorted_bai"),
            output_bai.to_path_buf(),
            ArtifactRole::Index,
        ),
    ];
    Ok(bam_artifact_inventory_from_outputs("bam.sort_index", output_root, &outputs))
}

fn sort_coordinates(document: &mut TinySamDocument) {
    let mut reference_rank = HashMap::<String, usize>::new();
    for (index, reference) in document.references.iter().enumerate() {
        reference_rank.insert(reference.clone(), index);
    }
    let fallback_rank = reference_rank.len() + 1;
    document.records.sort_by(|left, right| {
        let left_rank = if left.is_mapped() {
            *reference_rank.get(&left.rname).unwrap_or(&fallback_rank)
        } else {
            fallback_rank
        };
        let right_rank = if right.is_mapped() {
            *reference_rank.get(&right.rname).unwrap_or(&fallback_rank)
        } else {
            fallback_rank
        };
        (left_rank, left.pos, left.qname.as_str()).cmp(&(
            right_rank,
            right.pos,
            right.qname.as_str(),
        ))
    });
}

fn index_payload(document: &TinySamDocument) -> String {
    let mut mapped_by_reference = HashMap::<String, u64>::new();
    for record in document.records.iter().filter(|entry| entry.is_mapped()) {
        *mapped_by_reference.entry(record.rname.clone()).or_insert(0) += 1;
    }
    let mut index_payload = String::new();
    for reference in &document.references {
        let count = mapped_by_reference.get(reference).copied().unwrap_or(0);
        index_payload.push_str(&format!("{reference}\t{count}\n"));
    }
    index_payload
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "sort_index/contracts.rs"]
mod contracts;
