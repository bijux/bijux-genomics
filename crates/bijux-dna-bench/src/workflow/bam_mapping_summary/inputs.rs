//! Owner: bijux-dna-bench
//! Input and mount closure for BAM mapping-summary attempts.

use bijux_dna_bench_model::{
    AdmittedBamCorpus, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
};

pub(super) fn execution_inputs(corpus: &AdmittedBamCorpus) -> Vec<BenchmarkExecutionInput> {
    vec![
        input("bam", &corpus.bam.path, "/input/ERR2112767.sorted.bam", &corpus.bam.sha256),
        input(
            "bam_index",
            &corpus.bam_index.path,
            "/input/ERR2112767.sorted.bam.bai",
            &corpus.bam_index.sha256,
        ),
        input(
            "corpus_data_index",
            &corpus.data_index_path,
            "/input/data-index.json",
            &corpus.data_index_sha256,
        ),
        input(
            "corpus_catalog",
            &corpus.catalog_path,
            "/input/corpus-catalog.json",
            &corpus.catalog_sha256,
        ),
    ]
}

fn input(id: &str, path: &str, mount_path: &str, sha256: &str) -> BenchmarkExecutionInput {
    BenchmarkExecutionInput {
        id: id.to_string(),
        kind: BenchmarkExecutionInputKind::Data,
        path: path.to_string(),
        mount_path: mount_path.to_string(),
        sha256: sha256.to_string(),
    }
}
