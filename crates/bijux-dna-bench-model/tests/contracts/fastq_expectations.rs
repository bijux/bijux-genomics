use bijux_dna_bench_model::{
    validate_fastq_read_length_expectations, BenchmarkFastqLayout,
    FastqReadLengthExpectationSample, FastqReadLengthExpectations,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1,
};

#[test]
fn exact_fastq_expectation_contract_accepts_canonical_closure() {
    assert!(validate_fastq_read_length_expectations(&valid_expectations()).is_ok());
}

#[test]
fn exact_fastq_expectation_contract_rejects_path_and_count_drift() {
    let mut expectations = valid_expectations();
    expectations.samples[0].summary_path = "../escaped.tsv".to_string();
    assert!(validate_fastq_read_length_expectations(&expectations)
        .is_err_and(|error| error.to_string().contains("artifact path")));

    let mut expectations = valid_expectations();
    expectations.file_count = 2;
    assert!(validate_fastq_read_length_expectations(&expectations)
        .is_err_and(|error| error.to_string().contains("file closure")));
}

#[test]
fn exact_fastq_expectation_contract_rejects_order_and_digest_drift() {
    let mut expectations = valid_expectations();
    expectations.samples.push(expectations.samples[0].clone());
    expectations.sample_count = 2;
    expectations.file_count = 2;
    assert!(validate_fastq_read_length_expectations(&expectations)
        .is_err_and(|error| error.to_string().contains("strictly ordered")));

    let mut expectations = valid_expectations();
    expectations.catalog_sha256 = "0".repeat(64);
    assert!(validate_fastq_read_length_expectations(&expectations)
        .is_err_and(|error| error.to_string().contains("catalog digest")));
}

fn valid_expectations() -> FastqReadLengthExpectations {
    FastqReadLengthExpectations {
        schema_version: FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1.to_string(),
        corpus_id: "exact-fastq-corpus".to_string(),
        sample_count: 1,
        file_count: 1,
        data_index_sha256: "a".repeat(64),
        catalog_sha256: "b".repeat(64),
        samples: vec![FastqReadLengthExpectationSample {
            sample_id: "ERR000001".to_string(),
            input_layout: BenchmarkFastqLayout::SingleEnd,
            file_count: 1,
            read_count: 2,
            minimum_read_length: 2,
            mean_read_length: 3.0,
            median_read_length: 3.0,
            maximum_read_length: 4,
            distinct_read_lengths: 2,
            summary_path: "summaries/ERR000001.summary.tsv".to_string(),
            summary_sha256: "c".repeat(64),
            histogram_path: "histograms/ERR000001.histogram.tsv".to_string(),
            histogram_sha256: "d".repeat(64),
            exact_rows_sha256: "e".repeat(64),
        }],
    }
}
