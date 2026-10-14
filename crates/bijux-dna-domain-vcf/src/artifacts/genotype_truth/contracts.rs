#![cfg(test)]
//! Pure owned truth invariants over typed observations, independent of local acquisition.
use anyhow::Context;

use super::super::{TinyVcfDocument, TinyVcfRecord};
use super::{summarize, GenotypeTruthInput};
use std::collections::BTreeMap;
use std::path::Path;

fn record(format: Option<&str>, samples: &[&str]) -> TinyVcfRecord {
    TinyVcfRecord {
        chrom: "chr1".to_string(),
        pos: 1,
        ref_allele: "A".to_string(),
        alt_alleles: vec!["G".to_string()],
        qual: None,
        filter: "PASS".to_string(),
        info: ".".to_string(),
        format: format.map(str::to_string),
        samples: samples.iter().map(|value| (*value).to_string()).collect(),
    }
}

#[test]
fn nonexistent_path_and_caller_labels_are_metadata_without_acquisition() {
    let document = TinyVcfDocument::default();
    let path = Path::new("/missing-genotype-observations/not-an-acquired-file.vcf");
    let report = summarize(GenotypeTruthInput {
        document: &document,
        input_vcf: path,
        stage_id: "",
        tool_id: "caller-owned-label",
        per_sample_missingness: BTreeMap::new(),
    });
    assert_eq!(report.input_vcf, path);
    assert_eq!(report.stage_id, "");
    assert_eq!(report.tool_id, "caller-owned-label");
    assert_eq!(
        (report.sample_count, report.variant_count, report.called_calls, report.missing_calls),
        (0, 0, 0, 0)
    );
    assert_eq!(report.schema_version, super::VCF_GENOTYPE_TRUTH_SCHEMA_VERSION);
}

#[test]
fn mixed_ploidy_invalid_calls_absent_gt_and_likelihood_values_keep_distinct_classifications() {
    let document = TinyVcfDocument {
        samples: vec!["sample-a".to_string(), "sample-b".to_string()],
        records: vec![
            record(Some("GT:GL"), &["0/0:.", "1|2:-1,0,-1"]),
            record(Some("GT:GP:PL"), &["./.:.:.", "0/1:.:1,2,3"]),
            record(Some("GT"), &["0", "0/1/2"]),
            record(Some("GT:GL"), &["nonsense:.,.,.", "0/.:"]),
            record(None, &["0/1", "0/1"]),
            record(Some("DP:GL"), &["8:.", "4:0"]),
        ],
        ..TinyVcfDocument::default()
    };
    let report = summarize(GenotypeTruthInput {
        document: &document,
        input_vcf: Path::new("metadata.vcf"),
        stage_id: "vcf.call",
        tool_id: "tool",
        per_sample_missingness: BTreeMap::new(),
    });
    assert_eq!(report.observed_ploidy_widths, [1, 2, 3]);
    assert_eq!((report.called_calls, report.missing_calls), (5, 3));
    assert_eq!(
        (report.reference_only_calls, report.mixed_allele_calls, report.alternate_only_calls),
        (2, 2, 1)
    );
    assert_eq!((report.phased_calls, report.unphased_calls), (1, 4));
    assert_eq!((report.sample_count, report.variant_count), (2, 6));
    assert_eq!(report.likelihood_fields_present, ["GL", "GP", "PL"]);
    assert_eq!(
        report.sites_with_likelihood_values,
        BTreeMap::from([("GL".to_string(), 2), ("PL".to_string(), 1)])
    );
    assert_eq!(
        report.sites_missing_likelihood_values,
        BTreeMap::from([("GL".to_string(), 1), ("GP".to_string(), 1)])
    );
}

#[test]
fn precomputed_missingness_is_preserved_instead_of_recomputed_or_rebound() {
    let document = TinyVcfDocument {
        samples: vec!["first-snapshot-sample".to_string()],
        records: vec![record(Some("GT"), &["./."])],
        ..TinyVcfDocument::default()
    };
    let missingness = BTreeMap::from([("stats-snapshot-sample".to_string(), 0.25)]);
    let report = summarize(GenotypeTruthInput {
        document: &document,
        input_vcf: Path::new("snapshot.vcf"),
        stage_id: "vcf.call",
        tool_id: "tool",
        per_sample_missingness: missingness.clone(),
    });
    assert_eq!(report.missing_calls, 1);
    assert_eq!(report.per_sample_missingness, missingness);
}

#[test]
fn path_facade_preserves_acquisition_and_decoder_error_priority() -> anyhow::Result<()> {
    let scratch = bijux_dna_testkit::tempdir_for("vcf-genotype-acquisition-contract");
    let path = scratch.path().join("declared.vcf");
    let error = super::super::summarize_vcf_genotype_truth(&path, "", "")
        .err()
        .context("missing input acquisition precedes scientific projection")?;
    assert_eq!(
        error.downcast_ref::<std::io::Error>().context("original IO error")?.kind(),
        std::io::ErrorKind::NotFound
    );
    std::fs::write(&path, "malformed-before-header\n")?;
    let error = super::super::summarize_vcf_genotype_truth(&path, "", "")
        .err()
        .context("malformed record precedes missing header refusal")?;
    assert_eq!(error.to_string(), "malformed VCF record at line 1: expected at least 8 fields");
    std::fs::write(&path, "##fileformat=VCFv4.3\n")?;
    let error = super::super::summarize_vcf_genotype_truth(&path, "", "")
        .err()
        .context("missing header rejects before projection")?;
    assert_eq!(error.to_string(), "missing #CHROM header line");
    Ok(())
}
