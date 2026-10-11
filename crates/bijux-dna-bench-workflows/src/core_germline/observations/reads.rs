//! Owner: bijux-dna-bench-workflows
//! Core germline observations reads.
use super::super::{
    anyhow, fs, BTreeMap, BufRead, BufReader, Context, FastqLengthBin, FastqRecord, Path, Result,
};
use std::io::Write as _;

pub(crate) fn read_fastq_records(path: &Path) -> Result<Vec<FastqRecord>> {
    let reader: Box<dyn BufRead> = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let file = fs::File::open(path)?;
        let decoder = flate2::read::MultiGzDecoder::new(file);
        Box::new(BufReader::new(decoder))
    } else {
        Box::new(BufReader::new(fs::File::open(path)?))
    };

    let mut lines = reader.lines();
    let mut records = Vec::new();
    while let Some(header) = lines.next() {
        let header = header?;
        let sequence =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        let plus =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        let quality =
            lines.next().ok_or_else(|| anyhow!("truncated FASTQ at {}", path.display()))??;
        records.push(FastqRecord { header, sequence, plus, quality });
    }
    Ok(records)
}

pub(crate) fn write_fastq_records(path: &Path, records: &[FastqRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let file =
        bijux_dna_infra::create_file(path).with_context(|| format!("create {}", path.display()))?;
    let gzip_output = path.extension().and_then(|ext| ext.to_str()) == Some("gz");
    if gzip_output {
        let mut writer = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        for record in records {
            writeln!(writer, "{}", record.header)?;
            writeln!(writer, "{}", record.sequence)?;
            writeln!(writer, "{}", record.plus)?;
            writeln!(writer, "{}", record.quality)?;
        }
        writer.flush()?;
        let _ = writer.finish()?;
    } else {
        let mut writer = std::io::BufWriter::new(file);
        for record in records {
            writeln!(writer, "{}", record.header)?;
            writeln!(writer, "{}", record.sequence)?;
            writeln!(writer, "{}", record.plus)?;
            writeln!(writer, "{}", record.quality)?;
        }
        writer.flush()?;
    }
    Ok(())
}

pub(crate) fn length_histogram(
    records_r1: &[FastqRecord],
    records_r2: Option<&[FastqRecord]>,
) -> Vec<FastqLengthBin> {
    let mut bins = BTreeMap::<u64, u64>::new();
    for record in records_r1 {
        *bins.entry(u64::try_from(record.sequence.len()).unwrap_or(u64::MAX)).or_insert(0) += 1;
    }
    if let Some(records_r2) = records_r2 {
        for record in records_r2 {
            *bins.entry(u64::try_from(record.sequence.len()).unwrap_or(u64::MAX)).or_insert(0) += 1;
        }
    }
    bins.into_iter().map(|(length, count)| FastqLengthBin { length, count }).collect()
}

pub(crate) fn total_bases(records: &[FastqRecord]) -> u64 {
    records.iter().map(|record| record.sequence.len() as u64).sum()
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Approximate normalized metrics retain the existing floating-point calculation; integer counts remain separate"
)]
pub(crate) fn combined_mean_quality(
    records_r1: &[FastqRecord],
    records_r2: Option<&[FastqRecord]>,
) -> f64 {
    let total_bases = total_bases(records_r1) + records_r2.map_or(0, total_bases);
    if total_bases == 0 {
        return 0.0;
    }
    let total_quality = quality_sum(records_r1) + records_r2.map_or(0, quality_sum);
    total_quality as f64 / total_bases as f64
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Approximate normalized metrics retain the existing floating-point calculation; integer counts remain separate"
)]
pub(crate) fn combined_gc_percent(
    records_r1: &[FastqRecord],
    records_r2: Option<&[FastqRecord]>,
) -> f64 {
    let total_bases = total_bases(records_r1) + records_r2.map_or(0, total_bases);
    if total_bases == 0 {
        return 0.0;
    }
    let gc_bases = gc_count(records_r1) + records_r2.map_or(0, gc_count);
    (gc_bases as f64 / total_bases as f64) * 100.0
}

pub(crate) fn quality_sum(records: &[FastqRecord]) -> u64 {
    records
        .iter()
        .flat_map(|record| record.quality.bytes())
        .map(|value| u64::from(value.saturating_sub(33)))
        .sum::<u64>()
}

pub(crate) fn gc_count(records: &[FastqRecord]) -> u64 {
    records
        .iter()
        .flat_map(|record| record.sequence.bytes())
        .filter(|base| matches!(*base, b'G' | b'g' | b'C' | b'c'))
        .count()
        .try_into()
        .unwrap_or(u64::MAX)
}
