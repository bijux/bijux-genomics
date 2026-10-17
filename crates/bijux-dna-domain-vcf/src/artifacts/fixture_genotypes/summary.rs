//! Pure accumulation over acquired mini-fixture text; input paths are diagnostic-only.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};

use super::genotypes::{parse_variant_record, ParsedGenotype};
use super::{
    VcfFixtureAlleleFrequency, VcfFixtureGenotypeInput, VcfFixtureGenotypeSummary,
    VcfFixtureSampleGenotypes,
};

/// Observe strict mini-fixture genotypes without reading the diagnostic source path.
///
/// Sample and record order, duplicate identity aggregation and last-seen site
/// frequencies are preserved. This observation does not validate a general VCF.
///
/// # Errors
/// Returns the existing mini-fixture header, record, position, FORMAT or allele
/// error, including the supplied diagnostic path where that context was present.
///
/// # Panics
/// Panics when called or alternate allele counts exceed the supported u32 fixture range.
#[allow(clippy::too_many_lines)]
pub fn summarize_fixture_genotypes(
    input: VcfFixtureGenotypeInput<'_>,
) -> Result<VcfFixtureGenotypeSummary> {
    let VcfFixtureGenotypeInput { text: raw, source_path: vcf_path } = input;
    let mut sample_ids = Vec::new();
    let mut sample_index = BTreeMap::new();
    let mut contigs = BTreeSet::new();
    let mut variant_count = 0u64;
    let mut genotype_states = BTreeMap::<String, VcfFixtureSampleGenotypes>::new();
    let mut allele_frequencies = BTreeMap::<(String, u64), VcfFixtureAlleleFrequency>::new();

    for (line_number, line) in raw.lines().enumerate() {
        if let Some(payload) = line.strip_prefix("#CHROM\t") {
            let fields = payload.split('\t').collect::<Vec<_>>();
            if fields.len() < 9 {
                return Err(anyhow!(
                    "VCF truth parser expected sample columns in {}",
                    vcf_path.display()
                ));
            }
            sample_ids = fields[8..].iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
            sample_index = sample_ids
                .iter()
                .enumerate()
                .map(|(index, sample_id)| (sample_id.clone(), index))
                .collect();
            for sample_id in &sample_ids {
                genotype_states.insert(sample_id.clone(), VcfFixtureSampleGenotypes::default());
            }
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            return Err(anyhow!(
                "VCF truth parser line {} in {} must contain at least 10 columns",
                line_number + 1,
                vcf_path.display()
            ));
        }
        let record = parse_variant_record(&fields)?;
        contigs.insert(record.contig.clone());
        variant_count += 1;
        let mut alt_allele_count = 0u64;
        let mut called_allele_count = 0u64;
        for (sample_id, genotype) in sample_ids.iter().zip(record.sample_genotypes.iter()) {
            let state = genotype_states
                .get_mut(sample_id)
                .ok_or_else(|| anyhow!("missing genotype accumulator for `{sample_id}`"))?;
            match genotype {
                ParsedGenotype::Missing => {
                    state.missing += 1;
                }
                ParsedGenotype::Unphased(alleles) => {
                    state.unphased_calls += 1;
                    called_allele_count += alleles.len() as u64;
                    alt_allele_count +=
                        alleles.iter().map(|allele| u64::from(*allele > 0)).sum::<u64>();
                    if genotype.is_hom_ref() {
                        state.hom_ref += 1;
                    } else if genotype.is_hom_alt() {
                        state.hom_alt += 1;
                    } else if genotype.is_het() {
                        state.het += 1;
                    }
                }
                ParsedGenotype::Phased(alleles) => {
                    state.phased_calls += 1;
                    called_allele_count += alleles.len() as u64;
                    alt_allele_count +=
                        alleles.iter().map(|allele| u64::from(*allele > 0)).sum::<u64>();
                    if genotype.is_hom_ref() {
                        state.hom_ref += 1;
                    } else if genotype.is_hom_alt() {
                        state.hom_alt += 1;
                    } else if genotype.is_het() {
                        state.het += 1;
                    }
                }
            }
            state.dosages.push(genotype.dosage().unwrap_or(0));
            state
                .contig_is_homozygous
                .entry(record.contig.clone())
                .or_default()
                .push(genotype.is_homozygous());
        }
        let allele_frequency = if called_allele_count == 0 {
            0.0
        } else {
            checked_f64_from_u64(alt_allele_count, "alternate allele count")
                / checked_f64_from_u64(called_allele_count, "called allele count")
        };
        allele_frequencies.insert(
            (record.contig, record.position),
            VcfFixtureAlleleFrequency {
                alt_allele_count,
                called_allele_count,
                alt_allele_frequency: allele_frequency,
            },
        );
    }

    if sample_ids.is_empty() || sample_index.is_empty() {
        return Err(anyhow!("VCF truth parser found no samples in {}", vcf_path.display()));
    }
    if variant_count == 0 {
        return Err(anyhow!("VCF truth parser found no variant records in {}", vcf_path.display()));
    }

    Ok(VcfFixtureGenotypeSummary {
        sample_count: sample_ids.len(),
        sample_ids,
        variant_count,
        contigs: contigs.into_iter().collect(),
        genotype_states,
        allele_frequencies,
    })
}

fn checked_f64_from_u64(value: u64, context: &str) -> f64 {
    u32::try_from(value)
        .map_or_else(|_| panic!("{context} exceeds the supported fixture range"), f64::from)
}
