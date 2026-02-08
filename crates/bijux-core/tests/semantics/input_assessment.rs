use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_core::prelude::input_assessment::{
    assess_input_dir, discover_fastq_files, is_fastq_path, is_gzip_path, FastqLayout,
};

fn temp_dir() -> Result<PathBuf> {
    let base = std::env::temp_dir().join(format!("bijux-core-test-{}", uuid::Uuid::new_v4()));
    bijux_infra::ensure_dir(&base)?;
    Ok(base)
}

fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_infra::ensure_dir(parent)?;
    }
    bijux_infra::write_bytes(path, contents)?;
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
    let root = temp_dir()?;
    write_file(&root.join("a.fastq"), b"@r1\nACGT\n+\n!!!!\n")?;
    write_file(&root.join("nested").join("b.fq.gz"), b"dummy")?;
    write_file(&root.join("notes.txt"), b"ignore")?;
    let mut files = discover_fastq_files(&root);
    files.sort();
    let expected = vec![root.join("a.fastq"), root.join("nested").join("b.fq.gz")];
    assert_eq!(files, expected);
    Ok(())
}

#[test]
fn assess_input_dir_groups_pairs() -> Result<()> {
    let root = temp_dir()?;
    write_file(&root.join("sample_R1.fastq.gz"), b"r1")?;
    write_file(&root.join("sample_R2.fastq.gz"), b"r2")?;
    let assessment = assess_input_dir(&root)?;
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
    let root = temp_dir()?;
    write_file(&root.join("solo.fastq.gz"), b"r1")?;
    let assessment = assess_input_dir(&root)?;
    assert_eq!(assessment.samples.len(), 1);
    let sample = &assessment.samples[0];
    assert_eq!(sample.id.layout, FastqLayout::SingleEnd);
    assert!(sample.r2.is_none());
    Ok(())
}
