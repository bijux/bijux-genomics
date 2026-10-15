use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::validate_bam_gc_coverage_truth_manifest_path;
use std::fs;
use std::path::{Path, PathBuf};

const DECLARATION: &str = "benchmarks/tests/fixtures/science/bam-gc-coverage-truth/manifest.toml";

struct CoverageGcTruthFixture {
    directory: tempfile::TempDir,
    manifest: PathBuf,
}

impl CoverageGcTruthFixture {
    fn acquire(source: &Path) -> Result<Self> {
        let parent = std::env::var_os("TEST_TMP_DIR")
            .map_or_else(|| source.join("artifacts/testing"), PathBuf::from);
        fs::create_dir_all(&parent)?;
        let directory = tempfile::Builder::new().prefix("coverage-gc-truth-").tempdir_in(parent)?;
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
fn coverage_gc_truth_preserves_weighted_regions_bins_and_admission() -> Result<()> {
    let root = crate::support::repo_root();
    let fixture = CoverageGcTruthFixture::acquire(&root)?;
    let expected_path = fixture.directory.path().join("expected.json");
    let original_manifest = fs::read(&fixture.manifest)?;
    let original_expected = fs::read(&expected_path)?;
    let report = validate_bam_gc_coverage_truth_manifest_path(&root, &fixture.manifest)?;
    assert_eq!(report.validated_coverage_sample_count, 1);
    assert_eq!(report.validated_gc_bias_sample_count, 1);
    assert_eq!(report.validated_coverage_region_count, 2);
    assert_eq!(report.validated_gc_bias_bin_count, 3);
    assert_eq!(
        report.checked_samples,
        vec![
            "human_like_target_window_coverage".to_string(),
            "human_like_gc_window_ladder".to_string()
        ]
    );
    assert!(report.valid);
    let baseline = serde_json::to_vec(&report)?;
    let expected: serde_json::Value = serde_json::from_slice(&original_expected)?;
    let mut wrong_depth = expected.clone();
    wrong_depth["coverage_truths"][0]["summary"]["mean_depth"] = serde_json::json!(2.0);
    fs::write(&expected_path, serde_json::to_vec(&wrong_depth)?)?;
    let error = validate_bam_gc_coverage_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("expected depth must match weighted region mean")?;
    assert!(error.to_string().contains("does not match weighted region mean"));
    let mut wrong_windows = expected;
    wrong_windows["gc_bias_truths"][0]["summary"]["windows"] = serde_json::json!(4);
    fs::write(&expected_path, serde_json::to_vec(&wrong_windows)?)?;
    let error = validate_bam_gc_coverage_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("expected windows must match GC bins")?;
    assert!(error.to_string().contains("does not match GC-bin total"));
    fs::write(&expected_path, &original_expected)?;
    let mut declaration: toml::Value = toml::from_str(std::str::from_utf8(&original_manifest)?)?;
    declaration["gc_bias_cases"][0]["window_size"] = toml::Value::Integer(0);
    declaration["expected_path"] = toml::Value::String("missing-expected.json".to_string());
    fs::write(&fixture.manifest, toml::to_string(&declaration)?)?;
    let error = validate_bam_gc_coverage_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("window admission must precede expected acquisition")?;
    assert!(error.to_string().contains("window_size greater than zero"));
    fs::write(&fixture.manifest, &original_manifest)?;
    fs::write(&expected_path, &original_expected)?;
    assert_eq!(
        serde_json::to_vec(&validate_bam_gc_coverage_truth_manifest_path(
            &root,
            &fixture.manifest
        )?)?,
        baseline
    );
    assert_eq!(fs::read(&fixture.manifest)?, original_manifest);
    assert_eq!(fs::read(&expected_path)?, original_expected);
    Ok(())
}
