use bijux_domain_fastq::contracts::{
    contract_for_stage, FastqArtifactKind, FastqPE, FastqSE, FastqStats,
};
use std::path::PathBuf;

#[test]
fn forbidden_transitions_are_rejected() {
    let Some(merge) = contract_for_stage("fastq.merge") else {
        panic!("merge contract");
    };
    assert_eq!(merge.input_kind, FastqArtifactKind::PairedEnd);
    let Some(stats) = contract_for_stage("fastq.stats_neutral") else {
        panic!("stats contract");
    };
    assert_eq!(stats.input_kind, FastqArtifactKind::SingleEnd);
    assert_ne!(merge.output_kind, stats.input_kind);
}

#[test]
fn optional_branches_are_explicit() {
    let Some(umi) = contract_for_stage("fastq.umi") else {
        panic!("umi contract");
    };
    let Some(preprocess) = contract_for_stage("fastq.preprocess") else {
        panic!("preprocess contract");
    };
    assert_eq!(umi.input_kind, FastqArtifactKind::PairedEnd);
    assert_eq!(preprocess.input_kind, FastqArtifactKind::PairedEnd);
}

#[test]
fn type_level_artifacts_are_public() {
    let se = FastqSE {
        r1: PathBuf::from("reads.fastq.gz"),
    };
    let pe = FastqPE {
        r1: PathBuf::from("reads_r1.fastq.gz"),
        r2: PathBuf::from("reads_r2.fastq.gz"),
    };
    let stats = FastqStats {
        report: PathBuf::from("stats.json"),
    };
    assert!(se.r1.to_string_lossy().contains("reads"));
    assert!(pe.r2.to_string_lossy().contains("r2"));
    assert!(stats.report.to_string_lossy().contains("stats"));
}
