#![cfg(test)]
//! Native membership compatibility is independent of filesystem and backend execution.

use super::cluster_otu_records;
use crate::params::edna::OtuClusteringEffectiveParams;
use crate::types::FastqRecord;

fn parameters(identity_threshold: f64) -> OtuClusteringEffectiveParams {
    OtuClusteringEffectiveParams {
        schema_version: "v1".to_string(),
        identity_threshold,
        threads: 2,
        output_table_kind: "otu_abundance_table".to_string(),
        report_artifact: "cluster_otus_report".to_string(),
        raw_backend_report_artifact: None,
        raw_backend_report_format: None,
    }
}

fn record(sequence: &str) -> FastqRecord {
    FastqRecord {
        header: "@read/1 retained description".to_string(),
        sequence: sequence.to_string(),
        plus: "+retained identity".to_string(),
        quality: "!".repeat(sequence.len()),
    }
}

#[test]
fn ambiguous_membership_uses_first_matching_representative_and_input_order() {
    let params = parameters(2.0 / 3.0);
    let first = cluster_otu_records(&[record("AAA"), record("ATT"), record("AAT")], &params);
    assert_eq!(first.representatives, ["AAA", "ATT"]);
    assert_eq!(first.counts, [2, 1]);
    let reversed = cluster_otu_records(&[record("ATT"), record("AAA"), record("AAT")], &params);
    assert_eq!(reversed.representatives, ["ATT", "AAA"]);
    assert_eq!(reversed.counts, [2, 1]);
    let chained = cluster_otu_records(&[record("AAA"), record("AAT"), record("ATT")], &params);
    assert_eq!(chained.representatives, ["AAA", "ATT"]);
    assert_eq!(chained.counts, [2, 1]);
}

#[test]
fn prefix_lengths_case_and_unicode_keep_existing_identity_arithmetic() {
    let prefix = cluster_otu_records(&[record("ACGT"), record("ACGTA")], &parameters(1.0));
    assert_eq!(prefix.representatives, ["ACGT"]);
    assert_eq!(prefix.counts, [2]);
    let case = cluster_otu_records(&[record("ACGT"), record("acgt")], &parameters(1.0));
    assert_eq!(case.representatives, ["ACGT", "acgt"]);
    assert_eq!(case.counts, [1, 1]);
    let unicode = cluster_otu_records(&[record("é"), record("é")], &parameters(0.5));
    assert_eq!(unicode.representatives, ["é"]);
    assert_eq!(unicode.counts, [2]);
    let above_unicode = cluster_otu_records(&[record("é"), record("é")], &parameters(0.5001));
    assert_eq!(above_unicode.representatives, ["é", "é"]);
    assert_eq!(above_unicode.counts, [1, 1]);
}

#[test]
fn threshold_equality_empty_sequences_and_nonfinite_values_keep_direct_call_semantics() {
    let reads = [record("ACGT"), record("ACGA")];
    assert_eq!(cluster_otu_records(&reads, &parameters(0.75)).counts, [2]);
    assert_eq!(cluster_otu_records(&reads, &parameters(0.7501)).counts, [1, 1]);
    let empty = cluster_otu_records(&[], &parameters(0.97));
    assert!(empty.representatives.is_empty());
    assert!(empty.counts.is_empty());
    let empty_reads = [record(""), record("A")];
    assert_eq!(cluster_otu_records(&empty_reads, &parameters(0.0)).counts, [2]);
    assert_eq!(cluster_otu_records(&empty_reads, &parameters(0.1)).counts, [1, 1]);
    for threshold in [f64::NAN, f64::INFINITY] {
        let clusters = cluster_otu_records(&[record("AAA"), record("AAA")], &parameters(threshold));
        assert_eq!(clusters.representatives, ["AAA", "AAA"]);
        assert_eq!(clusters.counts, [1, 1]);
    }
    for threshold in [f64::NEG_INFINITY, -0.5] {
        assert_eq!(cluster_otu_records(&empty_reads, &parameters(threshold)).counts, [2]);
    }
}

#[test]
fn membership_retains_record_identity_and_ignores_report_only_parameters() {
    let reads = [record("AAA"), record("AAA"), record("ATT")];
    let original = reads.clone();
    let params = parameters(0.97);
    let before = cluster_otu_records(&reads, &params);
    let mut metadata = params;
    metadata.threads = 99;
    metadata.output_table_kind = "retained-report-label".to_string();
    metadata.report_artifact = "not-a-path-to-read".to_string();
    metadata.raw_backend_report_artifact = Some("nonexistent-backend-evidence".to_string());
    metadata.raw_backend_report_format = Some("retained-format-label".to_string());
    let after = cluster_otu_records(&reads, &metadata);
    assert_eq!(before.representatives, ["AAA", "ATT"]);
    assert_eq!(before.counts, [2, 1]);
    assert_eq!(after.representatives, before.representatives);
    assert_eq!(after.counts, before.counts);
    assert_eq!(reads, original);
}
