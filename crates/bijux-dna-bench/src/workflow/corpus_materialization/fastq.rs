//! Owner: bijux-dna-bench
//! Bounded gzip and FASTQ record validation for corpus admission.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use flate2::read::MultiGzDecoder;
use rayon::prelude::*;

const MAX_FASTQ_LINE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FastqIdentity {
    pub(super) read_count: u64,
    pub(super) base_count: u64,
}

pub(super) fn validate_fastq_files(
    paths: &[PathBuf],
    maximum_parallelism: usize,
) -> Result<Vec<FastqIdentity>> {
    if paths.is_empty() {
        bail!("FASTQ validation batch is empty");
    }
    if maximum_parallelism == 0 {
        bail!("FASTQ validation parallelism must be positive");
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(maximum_parallelism)
        .build()
        .context("create FASTQ validation pool")?;
    pool.install(|| paths.par_iter().map(|path| validate_fastq(path)).collect())
}

fn validate_fastq(path: &Path) -> Result<FastqIdentity> {
    let file = File::open(path).with_context(|| format!("open FASTQ {}", path.display()))?;
    let mut reader = BufReader::new(MultiGzDecoder::new(file));
    let mut lines = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut read_count = 0_u64;
    let mut base_count = 0_u64;
    loop {
        if read_bounded_line(&mut reader, &mut lines[0], "header", path)? == 0 {
            break;
        }
        for (offset, line) in lines.iter_mut().enumerate().skip(1) {
            if read_bounded_line(&mut reader, line, "record", path)? == 0 {
                bail!("incomplete FASTQ record {} in {}", read_count + 1, path.display());
            }
            if offset == 3 && line.is_empty() {
                bail!("FASTQ quality is empty in {}", path.display());
            }
        }
        let header = trim_line_ending(&lines[0]);
        let sequence = trim_line_ending(&lines[1]);
        let separator = trim_line_ending(&lines[2]);
        let quality = trim_line_ending(&lines[3]);
        if !header.starts_with(b"@")
            || !separator.starts_with(b"+")
            || sequence.is_empty()
            || sequence.len() != quality.len()
        {
            bail!("invalid FASTQ record {} in {}", read_count + 1, path.display());
        }
        read_count = read_count.checked_add(1).context("FASTQ read count overflow")?;
        base_count = base_count
            .checked_add(u64::try_from(sequence.len()).context("FASTQ sequence length overflow")?)
            .context("FASTQ base count overflow")?;
    }
    if read_count == 0 {
        bail!("FASTQ contains no records: {}", path.display());
    }
    Ok(FastqIdentity { read_count, base_count })
}

fn read_bounded_line(
    reader: &mut impl BufRead,
    output: &mut Vec<u8>,
    label: &str,
    path: &Path,
) -> Result<usize> {
    output.clear();
    loop {
        let available = reader
            .fill_buf()
            .with_context(|| format!("read FASTQ {label} from {}", path.display()))?;
        if available.is_empty() {
            return Ok(output.len());
        }
        let consumed = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if output.len().saturating_add(consumed) > MAX_FASTQ_LINE_BYTES {
            bail!("FASTQ {label} exceeds {MAX_FASTQ_LINE_BYTES} bytes in {}", path.display());
        }
        output.extend_from_slice(&available[..consumed]);
        let complete = available[consumed - 1] == b'\n';
        reader.consume(consumed);
        if complete {
            return Ok(output.len());
        }
    }
}

fn trim_line_ending(mut value: &[u8]) -> &[u8] {
    if value.ends_with(b"\n") {
        value = &value[..value.len() - 1];
    }
    if value.ends_with(b"\r") {
        value = &value[..value.len() - 1];
    }
    value
}
