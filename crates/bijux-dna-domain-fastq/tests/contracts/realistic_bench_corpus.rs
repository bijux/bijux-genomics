use bijux_dna_domain_fastq::{
    bench_corpus_manifest, required_bench_corpus_scenarios, BenchCorpusId,
};

#[test]
fn realistic_regression_corpus_covers_required_fastq_scenarios() {
    let manifest = bench_corpus_manifest(BenchCorpusId::FastqRealisticRegression);
    let required = required_bench_corpus_scenarios();
    assert_eq!(
        manifest.scenarios_covered, required,
        "realistic FASTQ regression corpus must cover all governed scenario classes"
    );
    assert_eq!(manifest.datasets.len(), required.len());
}

#[test]
fn realistic_regression_corpus_keeps_scope_and_layout_explicit() {
    let manifest = bench_corpus_manifest(BenchCorpusId::FastqRealisticRegression);
    for dataset in &manifest.datasets {
        assert!(
            !dataset.scientific_scope.is_empty(),
            "{} missing scientific scope",
            dataset.dataset_id
        );
        assert!(!dataset.scenarios.is_empty(), "{} missing scenario tags", dataset.dataset_id);
    }
}
