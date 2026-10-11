use std::fs;
use std::path::PathBuf;

use bijux_dna_bench_corpus::edna::{
    load_edna_corpus_fixture_manifest_path, load_validated_edna_expected_taxa_rows,
    resolve_edna_expected_taxa_path, validate_edna_corpus_fixture_manifest_path,
    DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH, EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn corpus_02_edna_fixture_manifest_validates_expected_taxa_and_sample_counts() {
    let root = repo_root();
    let report = validate_edna_corpus_fixture_manifest_path(
        &root,
        &root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH),
    )
    .expect("validate corpus-02 edna fixture manifest");

    assert_eq!(report.schema_version, EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION);
    assert_eq!(report.corpus_id, "corpus-02-edna-mini");
    assert_eq!(report.community_id, "mock_community_taxonomy");
    assert_eq!(report.compression, "gzip");
    assert_eq!(report.sample_count, 2);
    assert_eq!(report.expected_taxa_count, 3);
    assert_eq!(
        report.expected_taxa_path,
        "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv"
    );
    assert_eq!(report.expected_taxa_output_row_count, 6);
    assert_eq!(report.expected_present_row_count, 3);
    assert_eq!(report.expected_absent_row_count, 3);
    assert!(report.valid);
    assert!(report.expected_taxa.iter().any(|taxon| {
        taxon.taxon_id == 28890
            && taxon.name == "Halobacterium salinarum"
            && taxon.rank == "species"
    }));
    assert!(report.samples.iter().any(|sample| {
        sample.sample_id == "mock_community_sample_b"
            && sample.community_label == "mixed_microbiome"
            && sample.observed_read_count == 2
    }));
}

#[test]
fn corpus_02_edna_fixture_validation_refuses_duplicate_expected_taxa() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("manifest.toml");
    let broken = fs::read_to_string(root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH))
        .expect("read governed corpus-02 edna manifest")
        .replacen("taxon_id = 28901", "taxon_id = 561", 1);
    fs::write(&manifest_path, broken).expect("write broken manifest");

    let error = validate_edna_corpus_fixture_manifest_path(&root, &manifest_path)
        .expect_err("manifest validation should reject duplicate expected taxa");
    assert!(
        error.to_string().contains("eDNA corpus fixture repeats expected taxon_id `561`"),
        "validation error should explain duplicate expected taxa: {error:#}"
    );
}

#[test]
fn corpus_02_edna_fixture_validation_refuses_invalid_expected_presence_value() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_dir = temp.path();
    let manifest_path = manifest_dir.join("manifest.toml");
    let expected_taxa_path = manifest_dir.join("expected_taxa.tsv");
    let sample_a = manifest_dir.join("mock_community_sample_a.fastq.gz");
    let sample_b = manifest_dir.join("mock_community_sample_b.fastq.gz");

    let manifest = fs::read_to_string(root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH))
        .expect("read governed corpus-02 edna manifest")
        .replace("normalized/mock_community_sample_a.fastq.gz", "mock_community_sample_a.fastq.gz")
        .replace("normalized/mock_community_sample_b.fastq.gz", "mock_community_sample_b.fastq.gz");
    fs::write(&manifest_path, manifest).expect("write manifest");

    let expected_taxa = fs::read_to_string(
        root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv"),
    )
    .expect("read governed expected taxa")
    .replacen("\tpresent", "\tmaybe", 1);
    fs::write(&expected_taxa_path, expected_taxa).expect("write expected taxa");

    fs::copy(
        root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/normalized/mock_community_sample_a.fastq.gz"),
        &sample_a,
    )
    .expect("copy sample a");
    fs::copy(
        root.join("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/normalized/mock_community_sample_b.fastq.gz"),
        &sample_b,
    )
    .expect("copy sample b");

    let error = validate_edna_corpus_fixture_manifest_path(&root, &manifest_path)
        .expect_err("manifest validation should reject invalid expected presence");
    assert!(
        error
            .to_string()
            .contains("eDNA expected taxonomy output presence must be `present` or `absent`"),
        "validation error should explain invalid expected presence value: {error:#}"
    );
}

#[test]
fn corpus_02_edna_expected_taxa_rows_cover_each_manifest_sample_taxon_pair() {
    let root = repo_root();
    let manifest_path = root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH);
    let manifest = load_edna_corpus_fixture_manifest_path(&manifest_path)
        .expect("read governed corpus-02 edna manifest");
    let expected_taxa_path = resolve_edna_expected_taxa_path(&manifest_path, &manifest)
        .expect("resolve expected taxa path");

    let rows = load_validated_edna_expected_taxa_rows(&manifest, &expected_taxa_path)
        .expect("load governed expected taxa rows");

    assert_eq!(rows.len(), 6);
    assert!(rows.iter().any(|row| {
        row.sample_id == "mock_community_sample_a"
            && row.taxon_id == 561
            && row.name == "Escherichia coli"
            && row.rank == "species"
    }));
    assert!(rows.iter().any(|row| {
        row.sample_id == "mock_community_sample_b"
            && row.taxon_id == 28890
            && row.name == "Halobacterium salinarum"
            && row.rank == "species"
    }));
}
