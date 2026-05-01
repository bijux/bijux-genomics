use std::path::Path;

use anyhow::Result;

use crate::artifacts::{
    DetectInstrumentArtifactsReportV1, DETECT_INSTRUMENT_ARTIFACTS_REPORT_SCHEMA_VERSION,
};

use super::fastq_io::read_fastq_records;

fn has_low_quality_tail(quality: &str) -> bool {
    let tail = quality.chars().rev().take(10).collect::<Vec<_>>();
    if tail.len() < 10 {
        return false;
    }
    tail.into_iter().all(|ch| (ch as u8) <= b'#')
}

fn has_patterned_flowcell_signature(header: &str) -> bool {
    let token = header.trim_start_matches('@').split_whitespace().next().unwrap_or(header);
    let fields = token.split(':').collect::<Vec<_>>();
    if fields.len() < 7 {
        return false;
    }
    fields.get(4).and_then(|tile| tile.parse::<u32>().ok()).is_some_and(|tile| tile >= 2000)
}

/// Detect instrument-associated FASTQ artifacts as advisory evidence.
///
/// # Errors
/// Returns an error when input cannot be parsed.
pub fn detect_instrument_artifacts(
    r1: &Path,
    r2: Option<&Path>,
) -> Result<DetectInstrumentArtifactsReportV1> {
    let mut records = read_fastq_records(r1)?;
    if let Some(path) = r2 {
        records.extend(read_fastq_records(path)?);
    }

    let mut poly_g_reads = 0_u64;
    let mut poly_n_reads = 0_u64;
    let mut quality_tail_reads = 0_u64;
    let mut patterned_flowcell_suspects = 0_u64;

    for record in &records {
        let seq = record.sequence.to_ascii_uppercase();
        let g_fraction = if seq.is_empty() {
            0.0
        } else {
            (seq.chars().filter(|ch| *ch == 'G').count() as f64) / (seq.len() as f64)
        };
        if seq.ends_with("GGGGGGGGGGGG") || g_fraction >= 0.80 {
            poly_g_reads += 1;
        }
        if seq.contains("NNNNNN") {
            poly_n_reads += 1;
        }
        if has_low_quality_tail(&record.quality) {
            quality_tail_reads += 1;
        }
        if has_patterned_flowcell_signature(&record.header) {
            patterned_flowcell_suspects += 1;
        }
    }

    let reads_in = records.len() as u64;
    let mut warnings = Vec::<String>::new();
    if reads_in > 0 {
        let poly_g_fraction = (poly_g_reads as f64) / (reads_in as f64);
        let poly_n_fraction = (poly_n_reads as f64) / (reads_in as f64);
        let low_qual_fraction = (quality_tail_reads as f64) / (reads_in as f64);
        if poly_g_fraction >= 0.10 {
            warnings.push("poly_g_fraction_high".to_string());
        }
        if poly_n_fraction >= 0.02 {
            warnings.push("poly_n_fraction_high".to_string());
        }
        if low_qual_fraction >= 0.10 {
            warnings.push("quality_tail_fraction_high".to_string());
        }
    }
    if patterned_flowcell_suspects > 0 {
        warnings.push("patterned_flowcell_signature_detected".to_string());
    }

    Ok(DetectInstrumentArtifactsReportV1 {
        schema_version: DETECT_INSTRUMENT_ARTIFACTS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.detect_instrument_artifacts".to_string(),
        stage_id: "fastq.detect_instrument_artifacts".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if r2.is_some() { "paired_end" } else { "single_end" }.to_string(),
        advisory_only: true,
        reads_in,
        poly_g_reads,
        poly_n_reads,
        quality_tail_reads,
        patterned_flowcell_suspects,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::detect_instrument_artifacts;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn detect_instrument_artifacts_reports_expected_signals() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-instrument-artifacts")?;
        let r1 = temp.path().join("r1.fastq");

        write_fastq(
            &r1,
            &[
                ("INST:1:FC:1:2101:100:100 1:N:0:ACGT", "GGGGGGGGGGGG", "!!!!!!!!!!!!"),
                ("INST:1:FC:1:1101:100:100 1:N:0:ACGT", "ACNNNNNNTTTT", "IIIIIIII####"),
                ("plain", "ACGTACGTACGT", "IIIIIIIIIIII"),
            ],
        )?;

        let report = detect_instrument_artifacts(&r1, None)?;
        assert_eq!(report.reads_in, 3);
        assert_eq!(report.poly_g_reads, 1);
        assert_eq!(report.poly_n_reads, 1);
        assert_eq!(report.patterned_flowcell_suspects, 1);
        Ok(())
    }
}
