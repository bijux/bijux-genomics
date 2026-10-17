#![cfg(test)]
//! Run-boundary and paired-observation contracts for retained fixture truth.
use std::path::Path;

use anyhow::{Context, Result};

use super::super::{
    summarize_fixture_genotypes, VcfFixtureGenotypeInput, VcfFixtureSampleGenotypes,
};

fn observations(contigs: &[(&str, &[bool])]) -> VcfFixtureSampleGenotypes {
    VcfFixtureSampleGenotypes {
        contig_is_homozygous: contigs
            .iter()
            .map(|(contig, flags)| ((*contig).to_string(), flags.to_vec()))
            .collect(),
        ..VcfFixtureSampleGenotypes::default()
    }
}

#[test]
fn homozygous_runs_require_two_adjacent_observations_and_flush_at_end() {
    let vectors: &[(&[bool], u64)] = &[
        (&[], 0),
        (&[false, false], 0),
        (&[true], 0),
        (&[false, true, false], 0),
        (&[true, true], 1),
        (&[true, true, false], 1),
        (&[true, true, false, true, true, true], 2),
    ];
    for (flags, expected) in vectors {
        assert_eq!(observations(&[("chr1", flags)]).homozygous_run_count(), *expected);
    }
}

#[test]
fn contig_boundaries_and_input_order_do_not_create_runs() {
    let separate = observations(&[("chrA", &[true]), ("chrB", &[true])]);
    assert_eq!(separate.homozygous_run_count(), 0);
    let alternating = observations(&[("chr1", &[true, false, true])]);
    assert_eq!(alternating.homozygous_run_count(), 0);
    let two_contigs = observations(&[("chrA", &[true, true]), ("chrB", &[false, true, true])]);
    assert_eq!(two_contigs.homozygous_run_count(), 2);
    assert_eq!(VcfFixtureSampleGenotypes::default().homozygous_run_count(), 0);
}

#[test]
fn shared_runs_observe_homozygosity_not_dosage_identity() -> Result<()> {
    let different_alleles = concat!(
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample-a\tsample-b\n",
        "chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t0/0\t1/1\n",
        "chr1\t2\t.\tA\tG\t.\tPASS\t.\tGT\t0/0\t1/1\n",
    );
    let report = summarize_fixture_genotypes(VcfFixtureGenotypeInput {
        text: different_alleles,
        source_path: Path::new(""),
    })?;
    let left = report.genotype_states.get("sample-a").context("left homozygous sample")?;
    let right = report.genotype_states.get("sample-b").context("right homozygous sample")?;
    assert_eq!(left.dosages, [0, 0]);
    assert_eq!(right.dosages, [2, 2]);
    assert_eq!(left.shared_homozygous_run_count(right), 1);
    let matching_dosages = concat!(
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample-a\tsample-b\n",
        "chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t0/1\t0/1\n",
        "chr1\t2\t.\tA\tG\t.\tPASS\t.\tGT\t0/1\t0/1\n",
    );
    let report = summarize_fixture_genotypes(VcfFixtureGenotypeInput {
        text: matching_dosages,
        source_path: Path::new(""),
    })?;
    let left = report.genotype_states.get("sample-a").context("left heterozygous sample")?;
    let right = report.genotype_states.get("sample-b").context("right heterozygous sample")?;
    assert_eq!(left.dosages, right.dosages);
    assert_eq!(left.shared_homozygous_run_count(right), 0);
    Ok(())
}

#[test]
fn shared_runs_keep_missing_contig_and_shorter_vector_semantics() {
    let left = observations(&[("chr1", &[true, true]), ("left-only", &[true, true])]);
    let disjoint = observations(&[("right-only", &[true, true])]);
    assert_eq!(left.shared_homozygous_run_count(&disjoint), 0);
    let longer = observations(&[("chr1", &[true, true, true, true])]);
    let shorter = observations(&[("chr1", &[true, true]), ("right-only", &[true, true])]);
    assert_eq!(longer.shared_homozygous_run_count(&shorter), 1);
    assert_eq!(shorter.shared_homozygous_run_count(&longer), 1);
    let empty = observations(&[("chr1", &[])]);
    assert_eq!(longer.shared_homozygous_run_count(&empty), 0);
    let trailing = observations(&[("chr1", &[false, true, true])]);
    let all_true = observations(&[("chr1", &[true, true, true])]);
    assert_eq!(trailing.shared_homozygous_run_count(&all_true), 1);
    let alternating = observations(&[("chr1", &[true, false, true])]);
    assert_eq!(alternating.shared_homozygous_run_count(&all_true), 0);
}
