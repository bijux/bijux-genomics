use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::input_assessment::{
    assess_input_dir, discover_fastq_files, is_fastq_path, is_gzip_path, write_input_assessment,
    FastqLayout,
};

fn temp_dir() -> Result<tempfile::TempDir> {
    Ok(tempfile::Builder::new().prefix("bijux-dna-core-test-").tempdir()?)
}

fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
}

#[test]
fn fastq_path_detection_accepts_common_extensions() {
    let cases = [
        "sample.fastq",
        "sample.fq",
        "sample.fastq.gz",
        "sample.fq.gz",
        "SAMPLE.FASTQ",
        "SAMPLE.FQ.GZ",
    ];
    for case in cases {
        let path = Path::new(case);
        assert!(is_fastq_path(path), "expected fastq path for {case}");
    }
    assert!(!is_fastq_path(Path::new("sample.txt")));
}

#[test]
fn gzip_detection_is_extension_based() {
    assert!(is_gzip_path(Path::new("sample.fastq.gz")));
    assert!(!is_gzip_path(Path::new("sample.fastq")));
}

#[test]
fn discover_fastq_files_finds_nested_inputs() -> Result<()> {
    let dir = temp_dir()?;
    let root = dir.path();
    write_file(&root.join("a.fastq"), b"@r1\nACGT\n+\n!!!!\n")?;
    write_file(&root.join("nested").join("b.fq.gz"), b"dummy")?;
    write_file(&root.join("notes.txt"), b"ignore")?;
    let files = discover_fastq_files(root);
    let expected = vec![root.join("a.fastq"), root.join("nested").join("b.fq.gz")];
    assert_eq!(files, expected);
    Ok(())
}

#[test]
fn assess_input_dir_groups_pairs() -> Result<()> {
    let dir = temp_dir()?;
    let root = dir.path();
    write_file(&root.join("sample_R1.fastq.gz"), b"r1")?;
    write_file(&root.join("sample_R2.fastq.gz"), b"r2")?;
    let assessment = assess_input_dir(root)?;
    assert_eq!(assessment.schema_version, 1);
    let sample = assessment
        .samples
        .iter()
        .find(|sample| sample.id.layout == FastqLayout::PairedEnd)
        .ok_or_else(|| anyhow!("paired sample entry"))?;
    assert_eq!(sample.id.layout, FastqLayout::PairedEnd);
    assert!(sample.r2.is_some());
    assert!(sample.naming_warnings.is_empty());
    Ok(())
}

#[test]
fn assess_input_dir_marks_single_end() -> Result<()> {
    let dir = temp_dir()?;
    let root = dir.path();
    write_file(&root.join("solo.fastq.gz"), b"r1")?;
    let assessment = assess_input_dir(root)?;
    assert_eq!(assessment.samples.len(), 1);
    let sample = &assessment.samples[0];
    assert_eq!(sample.id.layout, FastqLayout::SingleEnd);
    assert!(sample.r2.is_none());
    Ok(())
}

#[test]
fn assess_input_dir_tracks_orphan_r2_files() -> Result<()> {
    let dir = temp_dir()?;
    let root = dir.path();
    let orphan = root.join("sample_R2.fastq.gz");
    write_file(&orphan, b"r2")?;

    let assessment = assess_input_dir(root)?;

    assert!(assessment.samples.is_empty());
    assert_eq!(assessment.unpaired_files, vec![orphan]);
    assert_eq!(assessment.issues, vec!["sample sample missing R1"]);
    Ok(())
}

#[test]
fn assess_input_dir_keeps_first_duplicate_read_candidate() -> Result<()> {
    let dir = temp_dir()?;
    let root = dir.path();
    let preferred_r1 = root.join("sample_R1.fastq.gz");
    let duplicate_r1 = root.join("sample_R1.fq.gz");
    let r2 = root.join("sample_R2.fastq.gz");
    write_file(&preferred_r1, b"r1")?;
    write_file(&duplicate_r1, b"r1-dup")?;
    write_file(&r2, b"r2")?;

    let assessment = assess_input_dir(root)?;
    let sample = assessment
        .samples
        .iter()
        .find(|sample| sample.id.sample_name == "sample")
        .ok_or_else(|| anyhow!("sample entry"))?;

    assert_eq!(sample.id.r1_path, preferred_r1);
    assert_eq!(sample.naming_warnings, vec!["multiple R1 candidates for sample"]);
    Ok(())
}

#[test]
fn write_input_assessment_persists_typed_payload() -> Result<()> {
    let dir = temp_dir()?;
    let root = dir.path();
    write_file(&root.join("sample_R1.fastq.gz"), b"r1")?;
    write_file(&root.join("sample_R2.fastq.gz"), b"r2")?;
    let assessment = assess_input_dir(root)?;
    let output = root.join("reports").join("input-assessment.json");

    write_input_assessment(&output, &assessment)?;

    let encoded = std::fs::read_to_string(&output)?;
    let decoded: bijux_dna_core::prelude::input_assessment::InputAssessmentV1 =
        serde_json::from_str(&encoded)?;
    assert_eq!(decoded.schema_version, 1);
    assert_eq!(decoded.samples.len(), 1);
    assert!(decoded.samples[0].r2.is_some());
    Ok(())
}

#[test]
fn write_input_assessment_replaces_existing_payload() -> Result<()> {
    let dir = temp_dir()?;
    let root = dir.path();
    write_file(&root.join("sample.fastq"), b"r1")?;
    let assessment = assess_input_dir(root)?;
    let output = root.join("input-assessment.json");
    write_file(&output, br#"{"stale":true}"#)?;

    write_input_assessment(&output, &assessment)?;

    let encoded = std::fs::read_to_string(&output)?;
    assert!(encoded.contains("\"schema_version\""));
    assert!(!encoded.contains("stale"));
    Ok(())
}
