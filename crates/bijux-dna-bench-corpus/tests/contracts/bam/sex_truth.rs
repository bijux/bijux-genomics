use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::validate_bam_sex_truth_manifest_path;
use std::fs;
use std::path::{Path, PathBuf};

const DECLARATION: &str = "benchmarks/tests/fixtures/science/sex-inference-truth/manifest.toml";

struct SexTruthFixture {
    directory: tempfile::TempDir,
    manifest: PathBuf,
}

impl SexTruthFixture {
    fn acquire(source: &Path) -> Result<Self> {
        let parent = std::env::var_os("TEST_TMP_DIR")
            .map_or_else(|| source.join("artifacts/testing"), PathBuf::from);
        fs::create_dir_all(&parent)?;
        let directory = tempfile::Builder::new().prefix("sex-truth-").tempdir_in(parent)?;
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
fn sex_truth_preserves_call_classes_and_insufficiency_admission() -> Result<()> {
    let root = crate::support::repo_root();
    let fixture = SexTruthFixture::acquire(&root)?;
    let expected_path = fixture.directory.path().join("expected.json");
    let original_manifest = fs::read(&fixture.manifest)?;
    let original_expected = fs::read(&expected_path)?;
    let report = validate_bam_sex_truth_manifest_path(&root, &fixture.manifest)?;
    assert_eq!(report.validated_case_count, 4);
    assert_eq!(report.validated_ok_case_count, 3);
    assert_eq!(report.validated_insufficient_case_count, 1);
    assert_eq!(
        report.validated_call_classes,
        vec![
            "ambiguous".to_string(),
            "female".to_string(),
            "insufficient".to_string(),
            "male".to_string()
        ]
    );
    assert_eq!(
        report.checked_samples,
        vec![
            "human_like_xy_autosome_coverage".to_string(),
            "human_like_x_autosome_coverage".to_string(),
            "human_like_x_y_ambiguous_coverage".to_string(),
            "human_like_y_haplogroup_panel".to_string()
        ]
    );
    assert!(report.valid);
    let baseline = serde_json::to_vec(&report)?;
    let expected: serde_json::Value = serde_json::from_slice(&original_expected)?;
    let mut missing_reason = expected.clone();
    missing_reason["sample_truths"][3]["summary"]["insufficiency_reason"] = serde_json::Value::Null;
    fs::write(&expected_path, serde_json::to_vec(&missing_reason)?)?;
    let error = validate_bam_sex_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("insufficient call must retain its reason")?;
    assert!(error
        .to_string()
        .contains("must declare insufficiency_reason when call is insufficient"));
    let mut false_reason = expected;
    false_reason["sample_truths"][0]["summary"]["insufficiency_reason"] =
        serde_json::json!("unexpected");
    fs::write(&expected_path, serde_json::to_vec(&false_reason)?)?;
    let error = validate_bam_sex_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("ok status must retain absent insufficiency reason")?;
    assert!(error.to_string().contains("must not declare insufficiency_reason when status is ok"));
    fs::write(&expected_path, &original_expected)?;
    let mut declaration: toml::Value = toml::from_str(std::str::from_utf8(&original_manifest)?)?;
    declaration["cases"][0]["chromosome_system"] = toml::Value::String(String::new());
    declaration["expected_path"] = toml::Value::String("missing-expected.json".to_string());
    fs::write(&fixture.manifest, toml::to_string(&declaration)?)?;
    let error = validate_bam_sex_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("chromosome declaration must precede expected acquisition")?;
    assert!(error.to_string().contains("non-empty chromosome_system"));
    fs::write(&fixture.manifest, &original_manifest)?;
    fs::write(&expected_path, &original_expected)?;
    assert_eq!(
        serde_json::to_vec(&validate_bam_sex_truth_manifest_path(&root, &fixture.manifest)?)?,
        baseline
    );
    assert_eq!(fs::read(&fixture.manifest)?, original_manifest);
    assert_eq!(fs::read(&expected_path)?, original_expected);
    Ok(())
}
