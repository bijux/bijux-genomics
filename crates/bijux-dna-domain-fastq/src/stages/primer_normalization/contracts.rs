#![cfg(test)]
//! Scientific behavior of the native local-smoke primer transform.

use super::{normalize_primer_records, PrimerNormalizationError, PrimerNormalizationRequest};
use crate::params::edna::PrimerNormalizationEffectiveParams;
use crate::params::PairedMode;
use crate::types::FastqRecord;

fn params() -> PrimerNormalizationEffectiveParams {
    PrimerNormalizationEffectiveParams {
        schema_version: "v1".to_string(),
        paired_mode: PairedMode::SingleEnd,
        threads: Some(2),
        orientation_policy: "normalize_to_forward_primer".to_string(),
        primer_set_id: "16s".to_string(),
        marker_id: Some("16S".to_string()),
        primer_fasta: None,
        max_mismatch_rate: 0.1,
        min_overlap_bp: 6,
        strict_5p_anchor: true,
        allow_iupac_codes: true,
    }
}

fn record(sequence: &str) -> FastqRecord {
    FastqRecord {
        header: "@read/1 description".to_string(),
        sequence: sequence.to_string(),
        plus: "+retained identity".to_string(),
        quality: "I".repeat(sequence.len()),
    }
}

#[test]
fn prefix_trimming_preserves_record_identity_and_corresponding_quality() {
    let parameters = params();
    let mut input = record("CCTACGGGACGT");
    input.quality = "abcdefghIJKL".to_string();
    let normalized = normalize_primer_records(PrimerNormalizationRequest {
        r1: vec![input.clone(), record("GACTACGG"), record("cctacgggACGT"), record("ACCTACGGG")],
        r2: None,
        params: &parameters,
    })
    .expect("admitted single-end records");
    assert_eq!(normalized.r1[0].sequence, "ACGT");
    assert_eq!(normalized.r1[0].quality, "IJKL");
    assert_eq!(normalized.r1[0].header, input.header);
    assert_eq!(normalized.r1[0].plus, input.plus);
    assert_eq!(normalized.r1[0].base_name_and_mate(), ("read".to_string(), Some(1)));
    assert_eq!(normalized.r1[1].sequence, "GG");
    assert_eq!(normalized.r1[2], record("cctacgggACGT"));
    assert_eq!(normalized.r1[3], record("ACCTACGGG"));
    assert_eq!(normalized.summary.reads_in, 4);
    assert_eq!(normalized.summary.reads_out, 4);
    assert_eq!(normalized.summary.bases_in, 41);
    assert_eq!(normalized.summary.bases_out, 27);
    assert_eq!(normalized.summary.primer_trimmed_reads, 2);
    assert!(normalized.r2.is_none());
}

#[test]
fn overlap_threshold_uses_complete_motif_length() {
    for (overlap, sequences, trimmed) in [
        (0, ["ACGT", "GG"], 2),
        (6, ["ACGT", "GG"], 2),
        (7, ["ACGT", "GACTACGG"], 1),
        (8, ["ACGT", "GACTACGG"], 1),
        (9, ["CCTACGGGACGT", "GACTACGG"], 0),
    ] {
        let mut parameters = params();
        parameters.min_overlap_bp = overlap;
        let normalized = normalize_primer_records(PrimerNormalizationRequest {
            r1: vec![record("CCTACGGGACGT"), record("GACTACGG")],
            r2: None,
            params: &parameters,
        })
        .expect("admitted records");
        assert_eq!(normalized.r1[0].sequence, sequences[0], "overlap {overlap}");
        assert_eq!(normalized.r1[1].sequence, sequences[1], "overlap {overlap}");
        assert_eq!(normalized.summary.primer_trimmed_reads, trimmed, "overlap {overlap}");
    }
}

#[test]
fn paired_forward_fraction_counts_only_r1_over_all_mates() {
    let parameters = params();
    let normalized = normalize_primer_records(PrimerNormalizationRequest {
        r1: vec![record("CCTACGGGACGT"), record("AAAA")],
        r2: Some(vec![record("GACTACGG"), record("CCTACGGGTT")]),
        params: &parameters,
    })
    .expect("coherent mates");
    assert_eq!(normalized.summary.reads_in, 4);
    assert_eq!(normalized.summary.reads_out, 4);
    assert_eq!(normalized.summary.bases_in, 34);
    assert_eq!(normalized.summary.bases_out, 12);
    assert_eq!(normalized.summary.primer_trimmed_reads, 3);
    assert_eq!(normalized.summary.primer_trimmed_fraction, Some(0.75));
    assert_eq!(normalized.summary.orientation_forward_fraction, Some(0.25));
    assert_eq!(normalized.r2.expect("mate records")[1].sequence, "TT");
}

#[test]
fn metadata_controls_do_not_change_native_smoke_trimming() {
    let parameters = params();
    let mut metadata = parameters.clone();
    metadata.schema_version = "metadata-schema".to_string();
    metadata.paired_mode = PairedMode::PairedEnd;
    metadata.threads = Some(64);
    metadata.orientation_policy = "metadata-orientation".to_string();
    metadata.primer_set_id = "metadata-primer-set".to_string();
    metadata.marker_id = None;
    metadata.primer_fasta = Some("unread-primer-bank.fasta".to_string());
    metadata.max_mismatch_rate = 0.9;
    metadata.strict_5p_anchor = false;
    metadata.allow_iupac_codes = false;
    let normalize = |params| {
        normalize_primer_records(PrimerNormalizationRequest {
            r1: vec![record("CCTACGGGACGT"), record("CCTACGGAACGT")],
            r2: None,
            params,
        })
        .expect("admitted records")
    };
    let expected = normalize(&parameters);
    let observed = normalize(&metadata);
    assert_eq!(observed.r1, expected.r1);
    assert_eq!(observed.r2, expected.r2);
    assert_eq!(observed.summary, expected.summary);
}

#[test]
fn empty_layouts_and_incoherent_mates_preserve_admission_semantics() {
    let parameters = params();
    for mate in [None, Some(Vec::new())] {
        let paired = mate.is_some();
        let normalized = normalize_primer_records(PrimerNormalizationRequest {
            r1: Vec::new(),
            r2: mate,
            params: &parameters,
        })
        .expect("empty records");
        assert_eq!(normalized.r2.is_some(), paired);
        assert_eq!(normalized.summary.reads_in, 0);
        assert_eq!(normalized.summary.reads_out, 0);
        assert_eq!(normalized.summary.bases_out, 0);
        assert_eq!(normalized.summary.primer_trimmed_fraction, Some(0.0));
        assert_eq!(normalized.summary.orientation_forward_fraction, Some(0.0));
    }
    let error = normalize_primer_records(PrimerNormalizationRequest {
        r1: vec![record("ACGT")],
        r2: Some(Vec::new()),
        params: &parameters,
    })
    .err()
    .expect("incoherent mates rejected");
    assert_eq!(error, PrimerNormalizationError::IncoherentMateCount { r1: 1, r2: 0 });
    assert_eq!(
        error.to_string(),
        "fastq.normalize_primers refused incoherent paired input: R1 count 1 != R2 count 0"
    );
}
