//! Owner: bijux-dna-bench-workflows
//! Amplicon observations reads.
use super::super::{
    bail, fs, write_text_file, BufRead, BufReader, Context, MultiGzDecoder, Path, Result,
};

pub(crate) fn write_fastq_records(path: &Path, records: &[(&str, &str)]) -> Result<()> {
    let mut rendered = String::new();
    for (id, sequence) in records {
        rendered.push('@');
        rendered.push_str(id);
        rendered.push('\n');
        rendered.push_str(sequence);
        rendered.push_str("\n+\n");
        rendered.push_str(&"I".repeat(sequence.len()));
        rendered.push('\n');
    }
    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        if let Some(parent) = path.parent() {
            bijux_dna_infra::ensure_dir(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let file = bijux_dna_infra::create_file(path)
            .with_context(|| format!("create {}", path.display()))?;
        let mut writer = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        std::io::Write::write_all(&mut writer, rendered.as_bytes())?;
        std::io::Write::flush(&mut writer)?;
        let _ = writer.finish()?;
        Ok(())
    } else {
        write_text_file(path, &rendered)
    }
}

pub(crate) fn count_gz_fastq_reads(path: &Path) -> Result<u64> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(MultiGzDecoder::new(file));
    let line_count = reader.lines().try_fold(0u64, |count, line| {
        let _ = line.with_context(|| format!("read {}", path.display()))?;
        Ok::<u64, anyhow::Error>(count + 1)
    })?;
    if !line_count.is_multiple_of(4) {
        bail!("fastq line count is not divisible by four in {}", path.display());
    }
    Ok(line_count / 4)
}
