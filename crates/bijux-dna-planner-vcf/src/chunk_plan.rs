use anyhow::{bail, Result};
use serde::Serialize;
use std::collections::BTreeSet;

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
    validate_unique_filter_names("chr_include", &chunking.chr_include)?;
    validate_unique_filter_names("chr_exclude", &chunking.chr_exclude)?;
    let include_names = chunking.chr_include.iter().collect::<BTreeSet<_>>();
    let exclude_names = chunking.chr_exclude.iter().collect::<BTreeSet<_>>();
    if let Some(name) = include_names.intersection(&exclude_names).next() {
        bail!("chunk contig `{name}` cannot appear in both chr_include and chr_exclude");
    }
    let known_contigs = species.contigs.iter().map(|contig| &contig.name).collect::<BTreeSet<_>>();
    if let Some(name) = include_names.difference(&known_contigs).next() {
        bail!("chunk chr_include contains unknown contig `{name}`");
    }
    if let Some(name) = exclude_names.difference(&known_contigs).next() {
        bail!("chunk chr_exclude contains unknown contig `{name}`");
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
            let end = start.saturating_add(chunking.window_size_bp - 1).min(contig.length_bp);
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
    if chunks.is_empty() {
        bail!("chunk filters produced no region chunks");
    }
    Ok(chunks)
}

fn validate_unique_filter_names(label: &str, names: &[String]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for name in names {
        if !seen.insert(name) {
            bail!("chunk {label} contains duplicate contig `{name}`");
        }
    }
    Ok(())
}

#[cfg(test)]
mod endpoint_tests {
    use super::*;
    use bijux_dna_domain_vcf::contracts::ContigSpec;

    #[test]
    fn bounded_two_chunk_plan_reaches_the_largest_contig_endpoint() -> Result<()> {
        let species = SpeciesContext {
            species_id: "Homo sapiens".into(),
            build_id: "GRCh38".into(),
            contig_set_digest: "a".repeat(64),
            contigs: vec![ContigSpec { name: "1".into(), length_bp: u64::MAX }],
            sex_system: "xy".into(),
            par_policy: "grch38_par".into(),
            default_coverage_regime: None,
        };
        let settings = ChunkPlanSettings {
            window_size_bp: u64::MAX - 1,
            overlap_bp: 0,
            chr_include: Vec::new(),
            chr_exclude: Vec::new(),
            max_parallel_chunks: 1,
        };
        let chunks = plan_region_chunks(&species, &settings)?;
        assert_eq!(
            chunks.iter().map(|chunk| (chunk.start, chunk.end)).collect::<Vec<_>>(),
            vec![(1, u64::MAX - 1), (u64::MAX, u64::MAX)]
        );
        Ok(())
    }
}
