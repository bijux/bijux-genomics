use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::validate_bam_duplicate_insert_truth_manifest_path;
use std::fs;
use std::path::{Path, PathBuf};

const DECLARATION: &str =
    "benchmarks/tests/fixtures/science/bam-duplicate-insert-truth/manifest.toml";

struct DuplicateInsertFixture {
    directory: tempfile::TempDir,
    manifest: PathBuf,
}

impl DuplicateInsertFixture {
    fn acquire(source: &Path) -> Result<Self> {
        let parent = std::env::var_os("TEST_TMP_DIR")
            .map_or_else(|| source.join("artifacts/testing"), PathBuf::from);
        fs::create_dir_all(&parent)?;
        let directory =
            tempfile::Builder::new().prefix("duplicate-insert-truth-").tempdir_in(parent)?;
        let manifest = directory.path().join("manifest.toml");
        fs::write(&manifest, fs::read(source.join(DECLARATION))?)?;
        fs::write(
            directory.path().join("expected.json"),
            fs::read(source.join(DECLARATION).with_file_name("expected.json"))?,
        )?;
        Ok(Self { directory, manifest })
    }
}

#[test]
fn duplicate_insert_truth_preserves_histogram_and_sample_order_admission() -> Result<()> {
    let root = crate::support::repo_root();
    let fixture = DuplicateInsertFixture::acquire(&root)?;
    let expected_path = fixture.directory.path().join("expected.json");
    let original_manifest = fs::read(&fixture.manifest)?;
    let original_expected = fs::read(&expected_path)?;
    let report = validate_bam_duplicate_insert_truth_manifest_path(&root, &fixture.manifest)?;
    assert_eq!(report.validated_sample_count, 2);
    assert_eq!(report.validated_duplicate_family_bin_count, 3);
    assert_eq!(report.validated_insert_size_bin_count, 3);
    assert_eq!(
        report.checked_samples,
        vec![
            "human_like_duplicate_cluster".to_string(),
            "human_like_insert_size_triplet".to_string()
        ]
    );
    assert!(report.valid);
    let baseline = serde_json::to_vec(&report)?;
    let expected: serde_json::Value = serde_json::from_slice(&original_expected)?;
    let mut inconsistent = expected.clone();
    inconsistent["sample_truths"][0]["mapped_reads"] = serde_json::json!(4);
    fs::write(&expected_path, serde_json::to_vec(&inconsistent)?)?;
    let error = validate_bam_duplicate_insert_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("expected count inconsistency must reject")?;
    assert!(error.to_string().contains("does not equal mapped+unmapped"));
    let mut reordered = expected;
    reordered["sample_truths"].as_array_mut().context("expected samples")?.reverse();
    fs::write(&expected_path, serde_json::to_vec(&reordered)?)?;
    let error = validate_bam_duplicate_insert_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("expected sample order must remain exact")?;
    assert!(error.to_string().contains("must contain samples"));
    fs::write(&expected_path, &original_expected)?;
    let mut declaration: toml::Value = toml::from_str(std::str::from_utf8(&original_manifest)?)?;
    declaration["corpus_manifest_path"] = toml::Value::String("missing-corpus.toml".to_string());
    declaration["expected_path"] = toml::Value::String("missing-expected.json".to_string());
    fs::write(&fixture.manifest, toml::to_string(&declaration)?)?;
    let error = validate_bam_duplicate_insert_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("corpus absence must reject before expected acquisition")?;
    assert!(error.to_string().contains("corpus manifest is missing"));
    fs::write(&fixture.manifest, &original_manifest)?;
    assert_eq!(
        serde_json::to_vec(&validate_bam_duplicate_insert_truth_manifest_path(
            &root,
            &fixture.manifest
        )?)?,
        baseline
    );
    assert_eq!(fs::read(&fixture.manifest)?, original_manifest);
    assert_eq!(fs::read(&expected_path)?, original_expected);
    Ok(())
}
