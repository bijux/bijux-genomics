use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    ClassifyLayoutReportV1, FastqLayoutClassV1, CLASSIFY_LAYOUT_REPORT_SCHEMA_VERSION,
};

const SAMPLE_RECORDS_PER_FILE: usize = 512;

#[derive(Debug, Clone)]
struct HeaderProbe {
    base: String,
    mate: Option<u8>,
    seq_len: usize,
}

/// Classify FASTQ layout before downstream planning.
///
/// # Errors
/// Returns an error when inputs cannot be read.
pub fn classify_layout(reads: &[&Path]) -> Result<ClassifyLayoutReportV1> {
    if reads.is_empty() {
        return Err(anyhow!("fastq.classify_layout requires at least one input FASTQ"));
    }
    if reads.len() > 2 {
        return Ok(invalid_report(
            reads.len() as u32,
            0,
            vec!["expected one or two FASTQ inputs for layout classification".to_string()],
        ));
    }

    let mut sampled = Vec::with_capacity(reads.len());
    for path in reads {
        sampled.push(sample_fastq_headers(path, SAMPLE_RECORDS_PER_FILE)?);
    }

    if reads.len() == 1 {
        classify_single_file_layout(&sampled[0])
    } else {
        classify_dual_file_layout(&sampled[0], &sampled[1])
    }
}

/// Refuse incoherent layout output using stable reasons.
///
/// # Errors
/// Returns an error for invalid layout classifications.
pub fn ensure_layout_is_coherent(report: &ClassifyLayoutReportV1) -> Result<()> {
    if report.layout == FastqLayoutClassV1::Invalid {
        let reason = report
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "invalid layout".to_string());
        return Err(anyhow!("fastq.classify_layout rejected input: {reason}"));
    }
    Ok(())
}

fn classify_single_file_layout(probes: &[HeaderProbe]) -> Result<ClassifyLayoutReportV1> {
    if probes.is_empty() {
        return Ok(invalid_report(
            1,
            0,
            vec!["input FASTQ has zero readable records".to_string()],
        ));
    }

    let mate1 = probes.iter().filter(|probe| probe.mate == Some(1)).count();
    let mate2 = probes.iter().filter(|probe| probe.mate == Some(2)).count();

    if mate1 > 0 || mate2 > 0 {
        let alternating_pairs = probes
            .chunks_exact(2)
            .filter(|pair| pair.len() == 2)
            .all(|pair| {
                pair[0].mate == Some(1)
                    && pair[1].mate == Some(2)
                    && pair[0].base == pair[1].base
            });

        if alternating_pairs && mate1 == mate2 {
            return Ok(layout_report(
                FastqLayoutClassV1::Interleaved,
                0.95,
                1,
                probes.len() as u64,
                Some(true),
                vec!["alternating mate tags and synchronized pair ids detected".to_string()],
            ));
        }

        if mate1 == 0 || mate2 == 0 {
            return Ok(layout_report(
                FastqLayoutClassV1::Singleton,
                0.9,
                1,
                probes.len() as u64,
                Some(false),
                vec!["only one mate tag family detected in a single FASTQ".to_string()],
            ));
        }

        return Ok(layout_report(
            FastqLayoutClassV1::Singleton,
            0.75,
            1,
            probes.len() as u64,
            Some(false),
            vec!["mixed mate tags detected without interleaved pair synchronization".to_string()],
        ));
    }

    let mean_len = probes.iter().map(|probe| probe.seq_len).sum::<usize>() / probes.len();
    if mean_len >= 220 {
        return Ok(layout_report(
            FastqLayoutClassV1::Merged,
            0.7,
            1,
            probes.len() as u64,
            None,
            vec![format!(
                "single FASTQ without mate tags and long mean read length ({mean_len})"
            )],
        ));
    }

    Ok(layout_report(
        FastqLayoutClassV1::SingleEnd,
        0.9,
        1,
        probes.len() as u64,
        None,
        vec!["single FASTQ without mate tags".to_string()],
    ))
}

fn classify_dual_file_layout(
    left: &[HeaderProbe],
    right: &[HeaderProbe],
) -> Result<ClassifyLayoutReportV1> {
    if left.is_empty() || right.is_empty() {
        return Ok(invalid_report(
            2,
            (left.len() + right.len()) as u64,
            vec!["paired-end classification requires non-empty R1 and R2 inputs".to_string()],
        ));
    }

    let pair_count = left.len().min(right.len());
    let synchronized = left
        .iter()
        .zip(right.iter())
        .take(pair_count)
        .filter(|(l, r)| l.base == r.base)
        .count();
    let sync_ratio = synchronized as f64 / pair_count as f64;

    let left_mate_consistent = left.iter().take(pair_count).all(|probe| probe.mate != Some(2));
    let right_mate_consistent = right.iter().take(pair_count).all(|probe| probe.mate != Some(1));

    if sync_ratio < 0.98 {
        return Ok(invalid_report(
            2,
            (left.len() + right.len()) as u64,
            vec![format!(
                "paired files are not synchronized (pair id sync ratio {sync_ratio:.3})"
            )],
        ));
    }

    if !left_mate_consistent || !right_mate_consistent {
        return Ok(invalid_report(
            2,
            (left.len() + right.len()) as u64,
            vec!["paired files contain inconsistent mate tags".to_string()],
        ));
    }

    let reasons = if left.len() != right.len() {
        vec![
            "paired files are synchronized for sampled records but record counts differ; downstream repair is required"
                .to_string(),
        ]
    } else {
        vec!["paired files are synchronized and mate tags are coherent".to_string()]
    };

    Ok(layout_report(
        FastqLayoutClassV1::PairedEnd,
        if left.len() == right.len() { 0.97 } else { 0.83 },
        2,
        (left.len() + right.len()) as u64,
        Some(true),
        reasons,
    ))
}

fn invalid_report(files_examined: u32, records_examined: u64, reasons: Vec<String>) -> ClassifyLayoutReportV1 {
    layout_report(
        FastqLayoutClassV1::Invalid,
        1.0,
        files_examined,
        records_examined,
        None,
        reasons,
    )
}

fn layout_report(
    layout: FastqLayoutClassV1,
    confidence: f64,
    files_examined: u32,
    records_examined: u64,
    pair_sync_observed: Option<bool>,
    reasons: Vec<String>,
) -> ClassifyLayoutReportV1 {
    ClassifyLayoutReportV1 {
        schema_version: CLASSIFY_LAYOUT_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.classify_layout".to_string(),
        stage_id: "fastq.classify_layout".to_string(),
        tool_id: "bijux".to_string(),
        layout,
        confidence,
        files_examined,
        records_examined,
        pair_sync_observed,
        reasons,
    }
}

fn sample_fastq_headers(path: &Path, max_records: usize) -> Result<Vec<HeaderProbe>> {
    let data = super::read_fastq_text(path)?;
    let mut probes = Vec::new();
    let mut line_buffer = [""; 4];

    for (line_index, line) in data.lines().enumerate() {
        line_buffer[line_index % 4] = line;
        if line_index % 4 != 3 {
            continue;
        }

        let header = line_buffer[0];
        let sequence = line_buffer[1];
        let plus = line_buffer[2];
        let quality = line_buffer[3];

        if !header.starts_with('@') {
            return Err(anyhow!("invalid FASTQ header at record {} in {}", probes.len(), path.display()));
        }
        if !plus.starts_with('+') {
            return Err(anyhow!("invalid FASTQ plus-line at record {} in {}", probes.len(), path.display()));
        }
        if sequence.len() != quality.len() {
            return Err(anyhow!(
                "sequence/quality length mismatch at record {} in {}",
                probes.len(),
                path.display()
            ));
        }

        let body = header.trim_start_matches('@').trim();
        let (base, mate) = parse_header_pairing(body);
        probes.push(HeaderProbe { base, mate, seq_len: sequence.len() });
        if probes.len() >= max_records {
            break;
        }
    }

    Ok(probes)
}

fn parse_header_pairing(header: &str) -> (String, Option<u8>) {
    let token = header.split_whitespace().next().unwrap_or(header);
    if let Some(base) = token.strip_suffix("/1") {
        return (base.to_string(), Some(1));
    }
    if let Some(base) = token.strip_suffix("/2") {
        return (base.to_string(), Some(2));
    }

    let mut fields = header.split_whitespace();
    let first = fields.next().unwrap_or(header);
    let second = fields.next();
    if let Some(descriptor) = second {
        if descriptor.starts_with("1:") {
            return (first.to_string(), Some(1));
        }
        if descriptor.starts_with("2:") {
            return (first.to_string(), Some(2));
        }
    }

    (first.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::{classify_layout, ensure_layout_is_coherent, FastqLayoutClassV1};

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn classify_interleaved_layout() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-layout")?;
        let reads = temp.path().join("reads.fastq");
        write_fastq(
            &reads,
            &[
                ("read1/1", "ACGT", "!!!!"),
                ("read1/2", "TGCA", "!!!!"),
                ("read2/1", "AAAA", "####"),
                ("read2/2", "TTTT", "####"),
            ],
        )?;

        let report = classify_layout(&[reads.as_path()])?;
        assert_eq!(report.layout, FastqLayoutClassV1::Interleaved);
        ensure_layout_is_coherent(&report)?;
        Ok(())
    }

    #[test]
    fn classify_paired_end_layout() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-layout")?;
        let r1 = temp.path().join("reads_R1.fastq");
        let r2 = temp.path().join("reads_R2.fastq");
        write_fastq(
            &r1,
            &[("read1/1", "ACGT", "!!!!"), ("read2/1", "CCCC", "####")],
        )?;
        write_fastq(
            &r2,
            &[("read1/2", "TGCA", "!!!!"), ("read2/2", "GGGG", "####")],
        )?;

        let report = classify_layout(&[r1.as_path(), r2.as_path()])?;
        assert_eq!(report.layout, FastqLayoutClassV1::PairedEnd);
        assert_eq!(report.pair_sync_observed, Some(true));
        Ok(())
    }

    #[test]
    fn classify_invalid_when_pair_sync_breaks() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-layout")?;
        let r1 = temp.path().join("reads_R1.fastq");
        let r2 = temp.path().join("reads_R2.fastq");
        write_fastq(
            &r1,
            &[("read1/1", "ACGT", "!!!!"), ("read2/1", "CCCC", "####")],
        )?;
        write_fastq(
            &r2,
            &[("readX/2", "TGCA", "!!!!"), ("readY/2", "GGGG", "####")],
        )?;

        let report = classify_layout(&[r1.as_path(), r2.as_path()])?;
        assert_eq!(report.layout, FastqLayoutClassV1::Invalid);
        assert!(ensure_layout_is_coherent(&report).is_err());
        Ok(())
    }
}
