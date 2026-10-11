//! Owner: bijux-dna-bench-corpus
//! DAMAGE corpus alignment ownership.
use super::{anyhow, fs, Path, Result, TinySamHeaderSummary};
use anyhow::Context;

pub(super) fn ensure_fixture_file(path: &Path, suffix: &str, label: &str) -> Result<()> {
    if !path.is_file() {
        return Err(anyhow!("{label} is missing: {}", path.display()));
    }
    if !path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with(suffix)) {
        return Err(anyhow!("{label} must end with `{suffix}`"));
    }
    Ok(())
}

#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed corpus filenames have case-sensitive spelling."
)]
pub(super) fn ensure_reference_fasta(path: &Path) -> Result<()> {
    if !path.is_file() {
        return Err(anyhow!("BAM damage fixture reference FASTA is missing: {}", path.display()));
    }
    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".fasta") || name.ends_with(".fa"))
    {
        return Err(anyhow!("BAM damage fixture reference FASTA must end with `.fasta` or `.fa`"));
    }
    Ok(())
}

pub(super) fn parse_reference_contigs(reference_fasta: &Path) -> Result<Vec<String>> {
    let payload = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut contigs = Vec::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            let contig = header.split_whitespace().next().unwrap_or_default().trim();
            if !contig.is_empty() {
                contigs.push(contig.to_string());
            }
        }
    }
    contigs.sort();
    contigs.dedup();
    Ok(contigs)
}

pub(super) fn parse_tiny_sam_header(bam_path: &Path) -> Result<TinySamHeaderSummary> {
    let payload =
        fs::read_to_string(bam_path).with_context(|| format!("read {}", bam_path.display()))?;
    let mut header = TinySamHeaderSummary::default();
    for (line_index, line) in payload.lines().enumerate() {
        if line.starts_with("@SQ") {
            let contig =
                line.split('\t').find_map(|field| field.strip_prefix("SN:")).ok_or_else(|| {
                    anyhow!(
                        "malformed SAM header at line {}: `@SQ` is missing `SN:`",
                        line_index + 1
                    )
                })?;
            header.contigs.push(contig.to_string());
        } else if line.starts_with("@RG") {
            let sample_id =
                line.split('\t').find_map(|field| field.strip_prefix("SM:")).ok_or_else(|| {
                    anyhow!(
                        "malformed SAM header at line {}: `@RG` is missing `SM:`",
                        line_index + 1
                    )
                })?;
            header.sample_ids.push(sample_id.to_string());
        } else if !line.starts_with('@') {
            break;
        }
    }
    header.contigs.sort();
    header.contigs.dedup();
    header.sample_ids.sort();
    header.sample_ids.dedup();
    Ok(header)
}
