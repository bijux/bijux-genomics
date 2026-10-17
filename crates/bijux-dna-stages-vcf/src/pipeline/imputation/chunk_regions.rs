use std::collections::BTreeSet;

use anyhow::Result;
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};

use crate::pipeline::{plan_regions_deterministic, ChunkingPlanParams};

/// Plan imputation region identifiers from already admitted contigs.
pub(super) fn plan_imputation_chunk_regions(
    contig_seen: &BTreeSet<String>,
    species_context: &SpeciesContext,
    chunk_window_bp: Option<u64>,
    chunk_overlap_bp: u64,
    include_chr: Option<&BTreeSet<String>>,
    exclude_chr: &BTreeSet<String>,
) -> Result<Vec<(String, String)>> {
    let window = chunk_window_bp.unwrap_or(0);
    let selected = contig_seen
        .iter()
        .filter(|contig| include_chr.is_none_or(|allow| allow.contains(*contig)))
        .filter(|contig| !exclude_chr.contains(*contig))
        .collect::<Vec<_>>();
    if window == 0 {
        let mut chunks = selected
            .into_iter()
            .map(|contig| (format!("{contig}:whole"), contig.clone()))
            .collect::<Vec<_>>();
        chunks.sort();
        return Ok(chunks);
    }
    let mut observed_species = species_context.clone();
    observed_species.contigs = selected
        .into_iter()
        .map(|contig| {
            species_context
                .contigs
                .iter()
                .find(|declared| declared.name == *contig)
                .cloned()
                .unwrap_or_else(|| ContigSpec { name: contig.clone(), length_bp: window })
        })
        .filter(|contig| contig.length_bp > 0)
        .collect();
    let regions = plan_regions_deterministic(
        &observed_species,
        &ChunkingPlanParams {
            window_size_bp: window,
            overlap_bp: chunk_overlap_bp,
            chr_level_threshold_bp: 0,
            ..ChunkingPlanParams::default()
        },
    )?;
    let mut chunks = regions
        .into_iter()
        .map(|region| {
            (format!("{}:{}-{}", region.chunk_id, region.start, region.end), region.contig)
        })
        .collect::<Vec<_>>();
    chunks.sort();
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context as _;

    fn species_with_contigs(contigs: &[(&str, u64)]) -> SpeciesContext {
        SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "owned-window-contract".to_string(),
            contigs: contigs
                .iter()
                .map(|(name, length_bp)| ContigSpec {
                    name: (*name).to_string(),
                    length_bp: *length_bp,
                })
                .collect(),
            sex_system: "xy".to_string(),
            par_policy: "unsupported".to_string(),
            default_coverage_regime: None,
        }
    }

    fn assert_finite_shared_endpoints() -> Result<()> {
        let species = species_with_contigs(&[("2", u64::MAX)]);
        for (window, expected) in [
            (u64::MAX, vec![(1, u64::MAX)]),
            (u64::MAX - 2, vec![(1, u64::MAX - 2), (u64::MAX - 1, u64::MAX)]),
        ] {
            let regions = plan_regions_deterministic(
                &species,
                &ChunkingPlanParams {
                    window_size_bp: window,
                    overlap_bp: 0,
                    chr_level_threshold_bp: 0,
                    ..ChunkingPlanParams::default()
                },
            )?;
            assert_eq!(
                regions.iter().map(|region| (region.start, region.end)).collect::<Vec<_>>(),
                expected
            );
        }
        Ok(())
    }

    #[test]
    fn imputation_regions_use_admitted_windows_and_preserve_selection() -> Result<()> {
        let species = species_with_contigs(&[("2", 1000), ("2", 5), ("10", 3), ("0", 0)]);
        let seen = BTreeSet::from(["0", "10", "2", "missing"].map(str::to_string));
        let empty = BTreeSet::new();
        for overlap in [500, 501] {
            // Empty selections establish admission before any nonempty overlap observation.
            for (observed, included, excluded) in [
                (&empty, None, &empty),
                (&seen, Some(&empty), &empty),
                (&seen, None, &seen),
                (&seen, None, &empty),
            ] {
                let error = plan_imputation_chunk_regions(
                    observed,
                    &species,
                    Some(500),
                    overlap,
                    included,
                    excluded,
                )
                .err()
                .context("fixed windows must refuse a nonprogress overlap")?;
                assert_eq!(error.to_string(), "overlap_bp must be less than window_size_bp");
            }
        }
        let include = BTreeSet::from(["0", "2", "missing"].map(str::to_string));
        let exclude = BTreeSet::from(["2".to_string()]);
        for window in [None, Some(0)] {
            assert_eq!(
                plan_imputation_chunk_regions(&seen, &species, window, u64::MAX, None, &empty)?,
                [
                    ("0:whole", "0"),
                    ("10:whole", "10"),
                    ("2:whole", "2"),
                    ("missing:whole", "missing")
                ]
                .map(|(id, contig)| (id.to_string(), contig.to_string()))
            );
            assert_eq!(
                plan_imputation_chunk_regions(
                    &seen,
                    &species,
                    window,
                    u64::MAX,
                    Some(&include),
                    &exclude,
                )?,
                [("0:whole", "0"), ("missing:whole", "missing")]
                    .map(|(id, contig)| (id.to_string(), contig.to_string()))
            );
        }
        assert_eq!(
            plan_imputation_chunk_regions(&seen, &species, Some(500), 50, None, &empty)?,
            [
                ("10:00000:1-3", "10"),
                ("2:00000:1-500", "2"),
                ("2:00001:451-950", "2"),
                ("2:00002:901-1000", "2"),
                ("missing:00000:1-500", "missing"),
            ]
            .map(|(id, contig)| (id.to_string(), contig.to_string()))
        );
        assert_eq!(
            plan_imputation_chunk_regions(
                &seen,
                &species,
                Some(500),
                50,
                Some(&include),
                &exclude,
            )?,
            [("missing:00000:1-500".to_string(), "missing".to_string())]
        );
        assert_eq!(
            plan_imputation_chunk_regions(
                &BTreeSet::from(["2".to_string()]),
                &species,
                Some(u64::MAX),
                0,
                None,
                &empty,
            )?,
            [("2:00000:1-1000".to_string(), "2".to_string())]
        );
        assert_finite_shared_endpoints()?;
        Ok(())
    }
}
