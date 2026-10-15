use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::validate_adna_damage_truth_manifest_path;

const DECLARATION: &str = "benchmarks/tests/fixtures/science/adna-damage-truth/manifest.toml";

struct DamageTruthFixture {
    directory: tempfile::TempDir,
    manifest: PathBuf,
    expected: PathBuf,
    alignment: PathBuf,
}

impl DamageTruthFixture {
    fn acquire(source: &Path) -> Result<Self> {
        let parent = std::env::var_os("TEST_TMP_DIR")
            .map_or_else(|| source.join("artifacts/testing"), PathBuf::from);
        fs::create_dir_all(&parent)?;
        let directory =
            tempfile::Builder::new().prefix("corpus-damage-truth-").tempdir_in(parent)?;
        let manifest = directory.path().join(DECLARATION);
        let declaration: toml::Value =
            toml::from_str(&fs::read_to_string(source.join(DECLARATION))?)?;
        let paths = declaration["source_paths"].as_array().context("damage source paths")?;
        for path in paths {
            let relative = path.as_str().context("damage source path")?;
            let destination = directory.path().join(relative);
            fs::create_dir_all(destination.parent().context("damage input parent")?)?;
            fs::write(destination, fs::read(source.join(relative))?)?;
        }
        fs::create_dir_all(manifest.parent().context("damage declaration parent")?)?;
        fs::write(&manifest, fs::read(source.join(DECLARATION))?)?;
        let expected = manifest.parent().context("damage bundle parent")?.join("expected.json");
        fs::write(&expected, fs::read(source.join(DECLARATION).with_file_name("expected.json"))?)?;
        let alignment = directory.path().join(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam",
        );
        Ok(Self { directory, manifest, expected, alignment })
    }
}

#[test]
fn damage_truth_preserves_projection_and_admission_order() -> Result<()> {
    let fixture = DamageTruthFixture::acquire(&crate::support::repo_root())?;
    let root = fixture.directory.path();
    let report = validate_adna_damage_truth_manifest_path(root, &fixture.manifest)?;
    assert_eq!(report.validated_case_count, 2);
    assert_eq!(report.validated_insufficient_case_count, 1);
    assert_eq!(
        report.checked_cases,
        vec!["non_udg_terminal_damage".to_string(), "half_udg_insufficient_signal".to_string()],
    );
    assert_eq!(
        report.terminal_pattern_classes,
        vec!["ct5p_dominant".to_string(), "insufficient_terminal_damage".to_string()],
    );
    assert!(report.valid);
    let baseline = serde_json::to_vec(&report)?;
    let expected = fs::read(&fixture.expected)?;
    let alignment = fs::read(&fixture.alignment)?;
    let original = std::str::from_utf8(&alignment)?;
    assert!(original.contains("SM:adna_damage_non_udg"));
    fs::write(&fixture.alignment, original.replace("SM:adna_damage_non_udg", "SM:foreign_sample"))?;
    let mut changed: serde_json::Value = serde_json::from_slice(&expected)?;
    changed["fixture_id"] = serde_json::json!("foreign-damage-truth");
    fs::write(&fixture.expected, serde_json::to_vec(&changed)?)?;
    let error = validate_adna_damage_truth_manifest_path(root, &fixture.manifest)
        .err()
        .context("foreign expected identity must reject before alignment observation")?;
    assert!(error.to_string().contains("does not match manifest fixture_id"));
    fs::write(&fixture.expected, &expected)?;
    let error = validate_adna_damage_truth_manifest_path(root, &fixture.manifest)
        .err()
        .context("foreign header sample must reject after expected admission")?;
    assert!(error.to_string().contains("is not present in alignment header samples"));
    fs::write(&fixture.alignment, &alignment)?;
    assert_eq!(
        serde_json::to_vec(&validate_adna_damage_truth_manifest_path(root, &fixture.manifest)?)?,
        baseline,
    );
    assert_eq!(fs::read(&fixture.expected)?, expected);
    assert_eq!(fs::read(&fixture.alignment)?, alignment);
    Ok(())
}
