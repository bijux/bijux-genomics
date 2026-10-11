use std::io::Write;

use bijux_dna_science::evidence::{derive_fastq_read_length_truth, FastqInputLayout};
use flate2::write::GzEncoder;
use flate2::Compression;

const READS_ONE: &[u8] = b"@r1 first\nAC\n+\n!!\n@r2\nACGT\n+\n!!!!\n";
const READS_TWO: &[u8] = b"@r1 second\r\nACG\r\n+\r\n!!!\r\n@r2\r\nACGTA\r\n+\r\n!!!!!\r\n";

#[test]
fn fastq_truth_streams_plain_and_gzip_inputs_in_declared_order() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("science-fastq-read-length-truth")?;
    let first = temporary.path().join("reads_R1.fastq");
    let second = temporary.path().join("reads_R2.fastq.gz");
    std::fs::write(&first, READS_ONE)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(READS_TWO)?;
    std::fs::write(&second, encoder.finish()?)?;

    let truth = derive_fastq_read_length_truth(
        "ERR-science",
        FastqInputLayout::PairedEnd,
        &[&first, &second],
    )?;

    assert_eq!(truth.file_count, 2);
    assert_eq!(truth.read_count, 4);
    assert_eq!(truth.minimum_read_length, 2);
    assert!((truth.mean_read_length - 3.5).abs() < f64::EPSILON);
    assert!((truth.median_read_length - 3.5).abs() < f64::EPSILON);
    assert_eq!(truth.maximum_read_length, 5);
    assert_eq!(truth.distinct_read_lengths, 4);
    assert_eq!(truth.histogram, [(2, 1), (3, 1), (4, 1), (5, 1)].into());
    assert_eq!(
        truth.exact_rows_sha256,
        bijux_dna_infra::hash_bytes_sha256(b"r1\t2\nr2\t4\nr1\t3\nr2\t5\n")
    );
    Ok(())
}

#[test]
fn fastq_truth_rejects_truncation_framing_and_quality_drift() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("science-fastq-read-length-invalid")?;
    let path = temporary.path().join("reads.fastq");

    std::fs::write(&path, b"@r1\nAC\n+\n")?;
    assert!(derive_fastq_read_length_truth("sample", FastqInputLayout::SingleEnd, &[&path])
        .is_err_and(|error| error.to_string().contains("truncated")));

    std::fs::write(&path, b"r1\nAC\n+\n!!\n")?;
    assert!(derive_fastq_read_length_truth("sample", FastqInputLayout::SingleEnd, &[&path])
        .is_err_and(|error| error.to_string().contains("header is invalid")));

    std::fs::write(&path, b"@r1\nAC\n+\n!\n")?;
    assert!(derive_fastq_read_length_truth("sample", FastqInputLayout::SingleEnd, &[&path])
        .is_err_and(|error| error.to_string().contains("quality lengths differ")));
    Ok(())
}

#[test]
fn fastq_truth_rejects_layout_mismatch_and_empty_evidence() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("science-fastq-read-length-closure")?;
    let path = temporary.path().join("reads.fastq");
    std::fs::write(&path, b"")?;

    assert!(derive_fastq_read_length_truth("sample", FastqInputLayout::PairedEnd, &[&path])
        .is_err_and(|error| error.to_string().contains("file closure")));
    assert!(derive_fastq_read_length_truth("sample", FastqInputLayout::SingleEnd, &[&path])
        .is_err_and(|error| error.to_string().contains("evidence is empty")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn fastq_truth_rejects_symlinked_inputs() -> anyhow::Result<()> {
    let temporary = bijux_dna_infra::temp_dir("science-fastq-read-length-symlink")?;
    let target = temporary.path().join("actual.fastq");
    let link = temporary.path().join("reads.fastq");
    std::fs::write(&target, READS_ONE)?;
    std::os::unix::fs::symlink(target, &link)?;

    assert!(derive_fastq_read_length_truth("sample", FastqInputLayout::SingleEnd, &[&link])
        .is_err_and(|error| error.to_string().contains("non-symlink")));
    Ok(())
}
