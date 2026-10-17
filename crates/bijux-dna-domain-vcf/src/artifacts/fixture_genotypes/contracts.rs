#![cfg(test)]
//! Meaningful strict-fixture observations, independent of acquisition and publication.
use std::path::Path;

use anyhow::{Context, Result};

use super::{summarize_fixture_genotypes, VcfFixtureGenotypeInput, VcfFixtureGenotypeSummary};

fn document(samples: &str, records: &[&str]) -> String {
    format!(
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{samples}\n{}\n",
        records.join("\n")
    )
}

fn observe(text: &str) -> Result<VcfFixtureGenotypeSummary> {
    summarize_fixture_genotypes(VcfFixtureGenotypeInput {
        text,
        source_path: Path::new("fixture-observations.vcf"),
    })
}

#[test]
fn source_path_is_diagnostic_only_for_acquired_fixture_text() -> Result<()> {
    let text = document("sample-a", &["chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t0/1"]);
    // An empty path cannot name an acquired file. The supplied text remains authoritative.
    let report = summarize_fixture_genotypes(VcfFixtureGenotypeInput {
        text: &text,
        source_path: Path::new(""),
    })?;
    assert_eq!((report.sample_count, report.variant_count), (1, 1));
    let state = report.genotype_states.get("sample-a").context("sample observation")?;
    assert_eq!((state.het, state.missing), (1, 0));
    for source_path in [Path::new(""), Path::new("diagnostic-only.vcf")] {
        let error = summarize_fixture_genotypes(VcfFixtureGenotypeInput {
            text: "##fileformat=VCFv4.2\n",
            source_path,
        })
        .err()
        .context("sample-free text must refuse")?;
        assert_eq!(
            error.to_string(),
            format!("VCF truth parser found no samples in {}", source_path.display())
        );
    }
    Ok(())
}

#[test]
fn strict_fixture_calls_preserve_phase_zygosity_and_called_allele_frequency() -> Result<()> {
    let text = document(
        "sample-a\tsample-b",
        &[
            "chr1\t1\t.\tA\tG,T\t.\tPASS\t.\tGT\t1/2\t0|0",
            "chr1\t2\t.\tA\tG,T\t.\tPASS\t.\tGT\t./.\t2",
        ],
    );
    let report = observe(&text)?;
    let left = report.genotype_states.get("sample-a").context("sample-a")?;
    let right = report.genotype_states.get("sample-b").context("sample-b")?;
    assert_eq!((left.hom_ref, left.het, left.hom_alt, left.missing), (0, 1, 0, 1));
    assert_eq!((left.phased_calls, left.unphased_calls), (0, 1));
    assert_eq!(left.dosages, [2, 0]);
    assert_eq!(left.contig_is_homozygous.get("chr1"), Some(&vec![false, false]));
    assert_eq!((right.hom_ref, right.het, right.hom_alt, right.missing), (1, 0, 1, 0));
    assert_eq!((right.phased_calls, right.unphased_calls), (1, 1));
    assert_eq!(right.dosages, [0, 1]);
    assert_eq!(right.contig_is_homozygous.get("chr1"), Some(&vec![true, true]));
    let first = report.allele_frequencies.get(&("chr1".to_string(), 1)).context("first site")?;
    let second = report.allele_frequencies.get(&("chr1".to_string(), 2)).context("second site")?;
    assert_eq!((first.alt_allele_count, first.called_allele_count), (2, 4));
    assert!((first.alt_allele_frequency - 0.5).abs() < f64::EPSILON);
    assert_eq!((second.alt_allele_count, second.called_allele_count), (1, 1));
    assert!((second.alt_allele_frequency - 1.0).abs() < f64::EPSILON);
    let missing = observe(&document("sample-a", &["chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t.|."]))?;
    let frequency =
        missing.allele_frequencies.get(&("chr1".to_string(), 1)).context("missing site")?;
    assert_eq!((frequency.alt_allele_count, frequency.called_allele_count), (0, 0));
    assert!(frequency.alt_allele_frequency.abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn invalid_fixture_calls_keep_original_error_priority() -> Result<()> {
    for (record, message) in [
        ("chr1\tinvalid\t.\tA\tG\t.\tPASS\t.\tDP\t7", "parse VCF position `invalid`"),
        ("chr1\t1\t.\tA\tG\t.\tPASS\t.\tDP\t7", "VCF truth parser requires GT in FORMAT"),
        ("chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t0/.", "parse genotype allele token `.`"),
        ("chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t0|1|2", "parse genotype allele token `1|2`"),
        ("chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t0/0\tinvalid", "parse genotype allele token `invalid`"),
    ] {
        let error = observe(&document("sample-a", &[record]))
            .err()
            .context("invalid fixture input must refuse")?;
        assert_eq!(error.to_string(), message);
    }
    Ok(())
}

#[test]
fn duplicate_sites_preserve_record_observations_and_last_frequency() -> Result<()> {
    let text = document(
        "sample-a",
        &[
            "chrZ\t5\t.\tA\tG\t.\tPASS\t.\tGT\t0/1",
            "chrA\t8\t.\tA\tG\t.\tPASS\t.\tGT\t0/0",
            "chrZ\t5\t.\tA\tG\t.\tPASS\t.\tGT\t1/1",
        ],
    );
    let report = observe(&text)?;
    assert_eq!(report.variant_count, 3);
    assert_eq!(report.contigs, ["chrA", "chrZ"]);
    assert_eq!(report.allele_frequencies.len(), 2);
    let state = report.genotype_states.get("sample-a").context("sample-a")?;
    assert_eq!(state.dosages, [1, 0, 2]);
    assert_eq!(state.contig_is_homozygous.get("chrZ"), Some(&vec![false, true]));
    assert_eq!(state.contig_is_homozygous.get("chrA"), Some(&vec![true]));
    let site = report.allele_frequencies.get(&("chrZ".to_string(), 5)).context("duplicate site")?;
    assert_eq!((site.alt_allele_count, site.called_allele_count), (2, 2));
    assert!((site.alt_allele_frequency - 1.0).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn header_sample_identity_and_short_rows_keep_existing_projection() -> Result<()> {
    let text = document(
        "duplicate\tduplicate",
        &["chr1\t1\t.\tA\tG,T\t.\tPASS\t.\tGT\t0/0\t1/2", "chr1\t2\t.\tA\tG\t.\tPASS\t.\tGT\t./."],
    );
    let report = observe(&text)?;
    assert_eq!(report.sample_ids, ["duplicate", "duplicate"]);
    assert_eq!((report.sample_count, report.variant_count), (2, 2));
    assert_eq!(report.genotype_states.len(), 1);
    let state = report.genotype_states.get("duplicate").context("duplicate sample")?;
    assert_eq!((state.hom_ref, state.het, state.hom_alt, state.missing), (1, 1, 0, 1));
    assert_eq!(state.dosages, [0, 2, 0]);
    let repeated = format!(
        "{}{}",
        document("old-sample", &["chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t1/1"]),
        document("new-sample", &["chr1\t2\t.\tA\tG\t.\tPASS\t.\tGT\t0/0"])
    );
    let report = observe(&repeated)?;
    assert_eq!(report.sample_ids, ["new-sample"]);
    assert_eq!((report.sample_count, report.variant_count), (1, 2));
    assert_eq!(report.genotype_states.len(), 2);
    assert_eq!(
        report.genotype_states.get("old-sample").context("retained old sample")?.dosages,
        [2]
    );
    assert_eq!(report.genotype_states.get("new-sample").context("current sample")?.dosages, [0]);
    for (text, message) in [
        ("##fileformat=VCFv4.2\n".to_string(), "VCF truth parser found no samples"),
        (document("sample-a", &[]), "VCF truth parser found no variant records"),
        (
            "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\n".to_string(),
            "VCF truth parser expected sample columns",
        ),
        (document("sample-a", &["chr1\t1"]), "VCF truth parser line 2"),
    ] {
        let error = observe(&text).err().context("inadmissible fixture shape must refuse")?;
        assert!(error.to_string().starts_with(message));
    }
    Ok(())
}
