use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_bench_corpus::bam::validate_adna_contamination_truth_manifest_path;

const DECLARATION: &str =
    "benchmarks/tests/fixtures/science/adna-contamination-truth/manifest.toml";

struct ContaminationTruthFixture {
    directory: tempfile::TempDir,
    manifest: PathBuf,
}

impl ContaminationTruthFixture {
    fn acquire(source: &Path) -> Result<Self> {
        let parent = std::env::var_os("TEST_TMP_DIR")
            .map_or_else(|| source.join("artifacts/testing"), PathBuf::from);
        fs::create_dir_all(&parent)?;
        let directory =
            tempfile::Builder::new().prefix("corpus-contamination-truth-").tempdir_in(parent)?;
        let manifest = directory.path().join(DECLARATION);
        let declaration: toml::Value =
            toml::from_str(&fs::read_to_string(source.join(DECLARATION))?)?;
        for path in declaration["source_paths"].as_array().context("contamination source paths")? {
            let relative = path.as_str().context("contamination source path")?;
            let destination = directory.path().join(relative);
            fs::create_dir_all(destination.parent().context("contamination input parent")?)?;
            fs::write(destination, fs::read(source.join(relative))?)?;
        }
        fs::create_dir_all(manifest.parent().context("contamination declaration parent")?)?;
        fs::write(&manifest, fs::read(source.join(DECLARATION))?)?;
        fs::write(
            manifest.with_file_name("expected.json"),
            fs::read(source.join(DECLARATION).with_file_name("expected.json"))?,
        )?;
        Ok(Self { directory, manifest })
    }
}

#[test]
fn contamination_truth_preserves_tool_context_and_interval_admission() -> Result<()> {
    let fixture = ContaminationTruthFixture::acquire(&crate::support::repo_root())?;
    let root = fixture.directory.path();
    let report = validate_adna_contamination_truth_manifest_path(root, &fixture.manifest)?;
    assert_eq!(report.validated_case_count, 6);
    assert_eq!(report.validated_insufficient_case_count, 3);
    assert_eq!(
        report.tool_ids,
        vec!["contammix".to_string(), "schmutzi".to_string(), "verifybamid2".to_string()]
    );
    assert_eq!(
        report.checked_cases,
        vec![
            "schmutzi_mitochondrial_ready".to_string(),
            "schmutzi_mitochondrial_insufficient".to_string(),
            "verifybamid2_nuclear_ready".to_string(),
            "verifybamid2_nuclear_insufficient".to_string(),
            "contammix_nuclear_ready".to_string(),
            "contammix_nuclear_insufficient".to_string(),
        ],
    );
    assert!(report.valid);
    let baseline = serde_json::to_vec(&report)?;
    let original = fs::read(&fixture.manifest)?;
    let expected_path = fixture.manifest.with_file_name("expected.json");
    let expected = fs::read(&expected_path)?;
    let mut declaration: toml::Value = toml::from_str(std::str::from_utf8(&original)?)?;
    fs::create_dir(root.join("declared-source-directory"))?;
    declaration["source_paths"]
        .as_array_mut()
        .context("contamination sources")?
        .push(toml::Value::String("declared-source-directory".to_string()));
    fs::write(&fixture.manifest, toml::to_string(&declaration)?)?;
    assert_eq!(
        serde_json::to_vec(&validate_adna_contamination_truth_manifest_path(
            root,
            &fixture.manifest
        )?)?,
        baseline,
    );
    let mut unsupported = declaration.clone();
    unsupported["cases"][0]["tool"] = toml::Value::String("unsupported".to_string());
    unsupported["expected_path"] = toml::Value::String("missing-expected.json".to_string());
    fs::write(&fixture.manifest, toml::to_string(&unsupported)?)?;
    let error = validate_adna_contamination_truth_manifest_path(root, &fixture.manifest)
        .err()
        .context("unsupported tool must reject before missing expected bundle")?;
    assert!(error.to_string().contains("must use schmutzi, verifybamid2, or contammix"));
    let mut inverted_interval = declaration;
    inverted_interval["cases"][0]["raw_ci_low"] = toml::Value::Float(0.025);
    fs::write(&fixture.manifest, toml::to_string(&inverted_interval)?)?;
    let error = validate_adna_contamination_truth_manifest_path(root, &fixture.manifest)
        .err()
        .context("inverted declared interval must reject")?;
    assert!(error.to_string().contains("must keep ci_low <= estimate <= ci_high"));
    fs::write(&fixture.manifest, &original)?;
    assert_eq!(
        serde_json::to_vec(&validate_adna_contamination_truth_manifest_path(
            root,
            &fixture.manifest
        )?)?,
        baseline,
    );
    assert_eq!(fs::read(&fixture.manifest)?, original);
    assert_eq!(fs::read(expected_path)?, expected);
    Ok(())
}
