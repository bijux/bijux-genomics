use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::validate_bam_haplogroup_truth_manifest_path;
use std::fs;
use std::path::{Path, PathBuf};

const DECLARATION: &str = "benchmarks/tests/fixtures/science/haplogroup-truth/manifest.toml";

struct HaplogroupTruthFixture {
    directory: tempfile::TempDir,
    manifest: PathBuf,
}

impl HaplogroupTruthFixture {
    fn acquire(source: &Path) -> Result<Self> {
        let parent = std::env::var_os("TEST_TMP_DIR")
            .map_or_else(|| source.join("artifacts/testing"), PathBuf::from);
        fs::create_dir_all(&parent)?;
        let directory = tempfile::Builder::new().prefix("haplogroup-truth-").tempdir_in(parent)?;
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
fn haplogroup_truth_preserves_marker_status_and_coverage_refusals() -> Result<()> {
    let root = crate::support::repo_root();
    let fixture = HaplogroupTruthFixture::acquire(&root)?;
    let expected_path = fixture.directory.path().join("expected.json");
    let original_manifest = fs::read(&fixture.manifest)?;
    let original_expected = fs::read(&expected_path)?;
    let report = validate_bam_haplogroup_truth_manifest_path(&root, &fixture.manifest)?;
    assert_eq!(report.validated_case_count, 3);
    assert_eq!(report.validated_ready_case_count, 1);
    assert_eq!(report.validated_uncertain_case_count, 1);
    assert_eq!(report.validated_coverage_gate_case_count, 1);
    assert_eq!(
        report.validated_statuses,
        vec![
            "coverage_gate_not_met".to_string(),
            "ready".to_string(),
            "uncertain_marker_support".to_string()
        ]
    );
    assert_eq!(
        report.checked_cases,
        vec![
            "screening_panel_ready".to_string(),
            "screening_panel_partial_support".to_string(),
            "screening_panel_coverage_gate".to_string()
        ]
    );
    assert!(report.valid);
    let baseline = serde_json::to_vec(&report)?;
    let expected: serde_json::Value = serde_json::from_slice(&original_expected)?;
    let mut wrong_ready = expected.clone();
    wrong_ready["case_truths"][0]["summary"]["ready"] = serde_json::json!(false);
    fs::write(&expected_path, serde_json::to_vec(&wrong_ready)?)?;
    let error = validate_bam_haplogroup_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("ready status must retain a ready call")?;
    assert!(error.to_string().contains("ready haplogroup call without refusal codes"));
    let mut wrong_gate = expected;
    wrong_gate["case_truths"][2]["summary"]["confidence"] = serde_json::json!(0.1);
    fs::write(&expected_path, serde_json::to_vec(&wrong_gate)?)?;
    let error = validate_bam_haplogroup_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("coverage-gated confidence must remain zero")?;
    assert!(error.to_string().contains("coverage-gated refusal semantics"));
    fs::write(&expected_path, &original_expected)?;
    let mut declaration: toml::Value = toml::from_str(std::str::from_utf8(&original_manifest)?)?;
    declaration["cases"][0]["minimum_coverage"] = toml::Value::Float(-1.0);
    declaration["expected_path"] = toml::Value::String("missing-expected.json".to_string());
    fs::write(&fixture.manifest, toml::to_string(&declaration)?)?;
    let error = validate_bam_haplogroup_truth_manifest_path(&root, &fixture.manifest)
        .err()
        .context("coverage declaration must precede expected acquisition")?;
    assert!(error.to_string().contains("non-negative minimum_coverage"));
    fs::write(&fixture.manifest, &original_manifest)?;
    fs::write(&expected_path, &original_expected)?;
    assert_eq!(
        serde_json::to_vec(&validate_bam_haplogroup_truth_manifest_path(
            &root,
            &fixture.manifest
        )?)?,
        baseline
    );
    assert_eq!(fs::read(&fixture.manifest)?, original_manifest);
    assert_eq!(fs::read(&expected_path)?, original_expected);
    Ok(())
}
