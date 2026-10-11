//! Owner: bijux-dna-bench-workflows
//! Ancient DNA observations reads.
use super::super::{bail, fs, BufRead, BufReader, Context, FastqRecord, Path, Result};

pub(crate) fn read_fastq_records(path: &Path) -> Result<Vec<FastqRecord>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut lines = Vec::<String>::new();
    loop {
        let mut line = String::new();
        let bytes =
            reader.read_line(&mut line).with_context(|| format!("read {}", path.display()))?;
        if bytes == 0 {
            break;
        }
        lines.push(line.trim_end().to_string());
    }
    if !lines.len().is_multiple_of(4) {
        bail!("FASTQ record count is incomplete in {}", path.display());
    }
    let mut records = Vec::new();
    for chunk in lines.chunks(4) {
        records.push(FastqRecord {
            header: chunk[0].clone(),
            sequence: chunk[1].clone(),
            plus: chunk[2].clone(),
            quality: chunk[3].clone(),
        });
    }
    Ok(records)
}

pub(crate) fn write_fastq_records(path: &Path, records: &[FastqRecord]) -> Result<()> {
    let mut payload = String::new();
    for record in records {
        payload.push_str(&record.header);
        payload.push('\n');
        payload.push_str(&record.sequence);
        payload.push('\n');
        payload.push_str(&record.plus);
        payload.push('\n');
        payload.push_str(&record.quality);
        payload.push('\n');
    }
    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        let file = bijux_dna_infra::create_file(path)
            .with_context(|| format!("create {}", path.display()))?;
        let mut writer = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        std::io::Write::write_all(&mut writer, payload.as_bytes())?;
        std::io::Write::flush(&mut writer)?;
        let _ = writer.finish()?;
    } else {
        bijux_dna_infra::write_bytes(path, payload)?;
    }
    Ok(())
}

pub(crate) fn total_fastq_bases(records: &[FastqRecord]) -> u64 {
    records.iter().map(|record| u64::try_from(record.sequence.len()).unwrap_or(0)).sum()
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Approximate normalized metrics retain the existing floating-point calculation; integer counts remain separate"
)]
pub(crate) fn mean_quality(left: &[FastqRecord], right: &[FastqRecord]) -> f64 {
    let mut total = 0_u64;
    let mut count = 0_u64;
    for record in left.iter().chain(right.iter()) {
        for ch in record.quality.bytes() {
            total += u64::from(ch.saturating_sub(33));
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}
