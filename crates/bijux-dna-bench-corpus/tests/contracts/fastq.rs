use std::fs;
use std::path::PathBuf;

use bijux_dna_bench_corpus::fastq::{
    validate_fastq_corpus_fixture_manifest_path, DEFAULT_CORPUS_01_MINI_MANIFEST_PATH,
    FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn corpus_01_mini_fixture_manifest_validates_expected_se_and_pe_counts() {
    let root = repo_root();
    let report = validate_fastq_corpus_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_01_MINI_MANIFEST_PATH),
    )
    .expect("validate corpus-01 mini fixture manifest");

    assert_eq!(report.schema_version, FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.corpus_id, "corpus-01-mini");
    assert_eq!(report.compression, "gzip");
    assert_eq!(report.sample_count, 11);
    assert_eq!(report.single_end_sample_count, 6);
    assert_eq!(report.paired_end_sample_count, 5);
    assert!(report.valid);
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "adna_like_pe_trim_signals"
            && sample.layout == "pe"
            && sample.observed_read_count_total == 4
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_se_adapter_hit"
            && sample.layout == "se"
            && sample.observed_read_count_total == 2
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_se_filter_signals"
            && sample.layout == "se"
            && sample.observed_read_count_total == 3
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_se_polyg_trim_signals"
            && sample.layout == "se"
            && sample.observed_read_count_total == 3
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "human_like_se_overrepresented_signals"
            && sample.layout == "se"
            && sample.observed_read_count_total == 5
    }));
}

#[test]
fn corpus_01_mini_fixture_validation_refuses_read_count_drift() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("manifest.toml");
    let broken = fs::read_to_string(root.join(DEFAULT_CORPUS_01_MINI_MANIFEST_PATH))
        .expect("read governed corpus-01 mini manifest")
        .replacen("expected_read_count_total = 2", "expected_read_count_total = 3", 1);
    fs::write(&manifest_path, broken).expect("write broken manifest");

    let error = validate_fastq_corpus_fixture_manifest_path(&root, &manifest_path)
        .expect_err("manifest validation should reject read count drift");
    assert!(
        error.to_string().contains("single-end total must match `expected_read_count_r1`"),
        "validation error should explain total-read drift: {error:#}"
    );
}
