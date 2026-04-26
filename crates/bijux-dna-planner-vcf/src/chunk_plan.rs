use anyhow::{bail, Result};
use serde::Serialize;

use crate::api::ChunkPlanSettings;
use bijux_dna_domain_vcf::contracts::SpeciesContext;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RegionChunkPlan {
    pub chunk_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
}

/// # Errors
/// Returns an error if the configured chunking rules are invalid.
pub fn plan_region_chunks(
    species: &SpeciesContext,
    chunking: &ChunkPlanSettings,
) -> Result<Vec<RegionChunkPlan>> {
    if chunking.window_size_bp == 0 {
        bail!("chunk window_size_bp must be > 0");
    }
    if chunking.overlap_bp >= chunking.window_size_bp {
        bail!("chunk overlap_bp must be < window_size_bp");
    }
    if chunking.max_parallel_chunks == 0 {
        bail!("chunk max_parallel_chunks must be > 0");
    }
    let mut chunks = Vec::new();
    let include_all = chunking.chr_include.is_empty();
    let step = chunking.window_size_bp - chunking.overlap_bp;
    for contig in &species.contigs {
        if !include_all && !chunking.chr_include.iter().any(|name| name == &contig.name) {
            continue;
        }
        if chunking.chr_exclude.iter().any(|name| name == &contig.name) {
            continue;
        }
        if contig.length_bp <= chunking.window_size_bp {
            chunks.push(RegionChunkPlan {
                chunk_id: format!("{}:whole", contig.name),
                contig: contig.name.clone(),
                start: 1,
                end: contig.length_bp,
            });
            continue;
        }
        let mut start = 1u64;
        let mut idx = 0usize;
        while start <= contig.length_bp {
            let end = std::cmp::min(start + chunking.window_size_bp - 1, contig.length_bp);
            chunks.push(RegionChunkPlan {
                chunk_id: format!("{}:{idx:05}", contig.name),
                contig: contig.name.clone(),
                start,
                end,
            });
            if end == contig.length_bp {
                break;
            }
            start = start.saturating_add(step);
            idx += 1;
        }
    }
    chunks.sort_by(|left, right| {
        left.contig
            .cmp(&right.contig)
            .then(left.start.cmp(&right.start))
            .then(left.end.cmp(&right.end))
            .then(left.chunk_id.cmp(&right.chunk_id))
    });
    Ok(chunks)
}
