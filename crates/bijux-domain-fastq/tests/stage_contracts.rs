use bijux_domain_fastq::core::{contract_for_stage, FastqArtifactKind};

#[test]
fn forbidden_transitions_are_rejected() {
    let Some(merge) = contract_for_stage("fastq.merge") else {
        panic!("merge contract");
    };
    assert_eq!(merge.input_kind, FastqArtifactKind::PairedEnd);
    let Some(stats) = contract_for_stage("fastq.stats") else {
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
