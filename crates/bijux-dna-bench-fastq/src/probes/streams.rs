use super::model::FastqReadStats;
use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::Path,
};
pub(super) fn count_fastq_stats(path: &Path) -> Result<FastqReadStats> {
    let reader: Box<dyn Read> = match path.extension().and_then(|value| value.to_str()) {
        Some("gz") => Box::new(MultiGzDecoder::new(
            fs::File::open(path).with_context(|| format!("open {}", path.display()))?,
        )),
        _ => Box::new(fs::File::open(path).with_context(|| format!("open {}", path.display()))?),
    };
    observe_records(BufReader::new(reader), path)
}
fn observe_records(reader: impl BufRead, path: &Path) -> Result<FastqReadStats> {
    let mut lines = reader.lines();
    let mut reads = 0_u64;
    let mut bases = 0_u64;
    while let Some(header) = lines.next() {
        let header = header.with_context(|| format!("read header from {}", path.display()))?;
        let sequence = lines
            .next()
            .ok_or_else(|| anyhow!("malformed FASTQ {}", path.display()))?
            .with_context(|| format!("read sequence from {}", path.display()))?;
        let plus = lines
            .next()
            .ok_or_else(|| anyhow!("malformed FASTQ {}", path.display()))?
            .with_context(|| format!("read plus from {}", path.display()))?;
        let quality = lines
            .next()
            .ok_or_else(|| anyhow!("malformed FASTQ {}", path.display()))?
            .with_context(|| format!("read quality from {}", path.display()))?;
        anyhow::ensure!(
            header.starts_with('@') && plus.starts_with('+') && sequence.len() == quality.len(),
            "malformed FASTQ record in {}",
            path.display()
        );
        reads += 1;
        bases += sequence.len() as u64;
    }
    Ok(FastqReadStats { reads, bases })
}
#[cfg(test)]
#[path = "../../tests/probes/streams.rs"]
mod tests;
