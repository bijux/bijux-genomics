use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{DepleteRrnaReportV1, DEPLETE_RRNA_REPORT_SCHEMA_VERSION};
use crate::params::screen::RrnaEffectiveParams;
use crate::params::PairedMode;

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

const RRNA_MOTIFS: &[&str] = &["GCGGCGTGCCTA", "ACCTGGTTGATCCTGCCAG"]; // representative rRNA-like patterns

fn rrna_like(record: &FastqRecord) -> bool {
    let seq = record.sequence.to_ascii_uppercase();
    RRNA_MOTIFS.iter().any(|motif| seq.contains(motif))
}

/// Deplete rRNA-like reads and emit retained outputs with DB provenance.
///
/// # Errors
/// Returns an error when paired inputs or outputs are incoherent.
pub fn deplete_rrna(
    r1: &Path,
    r2: Option<&Path>,
    params: &RrnaEffectiveParams,
    output_r1: &Path,
    output_r2: Option<&Path>,
    rrna_report_tsv: &Path,
    rrna_report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<DepleteRrnaReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 { read_fastq_records(path)? } else { Vec::new() };

    let paired = r2.is_some();
    if paired && left.len() != right.len() {
        return Err(anyhow!(
            "fastq.deplete_rrna refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }
    if paired && output_r2.is_none() {
        return Err(anyhow!("fastq.deplete_rrna requires output_r2 for paired input"));
    }

    let reads_in = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_in = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let mut retained_left = Vec::<FastqRecord>::new();
    let mut retained_right = Vec::<FastqRecord>::new();
    let mut removed_reads = 0_u64;

    if paired {
        for (l, r) in left.iter().cloned().zip(right.iter().cloned()) {
            if rrna_like(&l) || rrna_like(&r) {
                removed_reads += 2;
            } else {
                retained_left.push(l);
                retained_right.push(r);
            }
        }
    } else {
        for l in left.iter().cloned() {
            if rrna_like(&l) {
                removed_reads += 1;
            } else {
                retained_left.push(l);
            }
        }
    }

    write_fastq_records(output_r1, &retained_left)?;
    if paired {
        write_fastq_records(output_r2.expect("validated above"), &retained_right)?;
    }

    let reads_out = if paired {
        (retained_left.len() + retained_right.len()) as u64
    } else {
        retained_left.len() as u64
    };
    let bases_out = retained_left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + retained_right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let rrna_fraction_removed =
        if reads_in == 0 { 0.0 } else { removed_reads as f64 / reads_in as f64 };

    std::fs::write(
        rrna_report_tsv,
        format!("metric\tvalue\nreads_in\t{reads_in}\nreads_removed\t{removed_reads}\n"),
    )?;
    std::fs::write(
        rrna_report_json,
        serde_json::to_string_pretty(&serde_json::json!({
            "reads_in": reads_in,
            "reads_removed": removed_reads,
            "rrna_fraction_removed": rrna_fraction_removed,
        }))?,
    )?;

    Ok(DepleteRrnaReportV1 {
        schema_version: DEPLETE_RRNA_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.deplete_rrna".to_string(),
        stage_id: "fastq.deplete_rrna".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        rrna_db: params.contaminant_db.clone(),
        database_artifact_id: params.database_artifact_id.clone(),
        database_build_id: params.database_build_id.clone(),
        database_digest: None,
        screening_engine: params.screening_engine.clone(),
        report_format: params.report_format.clone(),
        emit_removed_reads: params.emit_removed_reads,
        min_identity: None,
        retained_read_role: "rrna_filtered_reads".to_string(),
        rejected_read_role: "removed_rrna_reads".to_string(),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        rrna_report_tsv: rrna_report_tsv.display().to_string(),
        rrna_report_json: rrna_report_json.display().to_string(),
        reads_in,
        reads_out,
        reads_removed: removed_reads,
        bases_in,
        bases_out,
        bases_removed: bases_in.saturating_sub(bases_out),
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(retained_left.len() as u64),
        rrna_fraction_removed,
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report.map(|_| "sortmerna_log".to_string()),
        backend_metrics: Some(serde_json::json!({
            "removed_rrna_reads": removed_reads,
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::deplete_rrna;
    use crate::params::screen::{RrnaEffectiveParams, RrnaReportFormat, RrnaScreeningEngine};
    use crate::params::PairedMode;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn deplete_rrna_reports_depletion_rate() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-deplete-rrna")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(
            &r1,
            &[("rrna", "GCGGCGTGCCTA", "IIIIIIIIIIII"), ("clean", "ACGTACGTACGT", "IIIIIIIIIIII")],
        )?;

        let params = RrnaEffectiveParams {
            schema_version: "bijux.fastq.params.deplete_rrna.v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            contaminant_db: Some("silva".to_string()),
            database_artifact_id: "silva_nr99".to_string(),
            database_build_id: Some("2026.04".to_string()),
            screening_engine: RrnaScreeningEngine::Sortmerna,
            report_format: RrnaReportFormat::SummaryTsvAndJson,
            emit_removed_reads: false,
        };

        let report = deplete_rrna(
            &r1,
            None,
            &params,
            &temp.path().join("retained.fastq"),
            None,
            &temp.path().join("rrna.tsv"),
            &temp.path().join("rrna.json"),
            None,
        )?;

        assert_eq!(report.reads_removed, 1);
        assert!(report.rrna_fraction_removed > 0.0);
        Ok(())
    }
}
