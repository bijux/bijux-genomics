use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::validate_bam_endogenous_truth_manifest_path;
use std::fs;
use std::path::{Path, PathBuf};

const DECLARATION: &str = "benchmarks/tests/fixtures/science/endogenous-truth/manifest.toml";

struct EndogenousTruthFixture {
    directory: tempfile::TempDir,
    manifest: PathBuf,
}

impl EndogenousTruthFixture {
    fn acquire(source: &Path) -> Result<Self> {
        let parent = std::env::var_os("TEST_TMP_DIR")
            .map_or_else(|| source.join("artifacts/testing"), PathBuf::from);
        fs::create_dir_all(&parent)?;
        let directory = tempfile::Builder::new().prefix("endogenous-truth-").tempdir_in(parent)?;
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
fn endogenous_truth_preserves_count_fraction_and_admission_order() -> Result<()> {
    let root = crate::support::repo_root();
    let fixture = EndogenousTruthFixture::acquire(&root)?;
    let expected_path = fixture.directory.path().join("expected.json");
    let original_manifest = fs::read(&fixture.manifest)?;
    let original_expected = fs::read(&expected_path)?;
    let report = validate_bam_endogenous_truth_manifest_path(&root, &fixture.manifest)?;
    assert_eq!(report.validated_sample_count, 2);
    assert_eq!(report.validated_total_reads, 8);
    assert_eq!(report.validated_contaminant_reads, 2);
    assert_eq!(report.validated_retained_reads, 6);
    assert_eq!(
        report.checked_samples,
        vec![
            "human_like_endogenous_partial_mapping".to_string(),
            "human_like_target_window_coverage".to_string()
        ]
    );
    assert!(report.valid);
    let baseline = serde_json::to_vec(&report)?;
    let expected: serde_json::Value = serde_json::from_slice(&original_expected)?;
    let mut inconsistent = expected;
    inconsistent["sample_truths"][0]["summary"]["retained_fraction"] = serde_json::json!(0.7);
    fs::write(&expected_path, serde_json::to_vec(&inconsistent)?)?;
    let error = validate_bam_endogenous_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("expected retained fraction must match declared counts")?;
    assert!(error.to_string().contains("retained_fraction drifted"));
    fs::write(&expected_path, &original_expected)?;
    let mut declaration: toml::Value = toml::from_str(std::str::from_utf8(&original_manifest)?)?;
    declaration["cases"][0]["prealignment_fraction"] = toml::Value::Float(1.1);
    declaration["expected_path"] = toml::Value::String("missing-expected.json".to_string());
    fs::write(&fixture.manifest, toml::to_string(&declaration)?)?;
    let error = validate_bam_endogenous_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("fraction admission must precede missing expected acquisition")?;
    assert!(error.to_string().contains("must be within [0, 1]"));
    fs::write(&fixture.manifest, &original_manifest)?;
    fs::write(&expected_path, &original_expected)?;
    assert_eq!(
        serde_json::to_vec(&validate_bam_endogenous_truth_manifest_path(
            &root,
            &fixture.manifest
        )?)?,
        baseline
    );
    assert_eq!(fs::read(&fixture.manifest)?, original_manifest);
    assert_eq!(fs::read(&expected_path)?, original_expected);
    Ok(())
}
