use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    DepleteReferenceContaminantsReportV1, DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION,
};
use crate::params::screen::ReferenceContaminantEffectiveParams;
use crate::params::PairedMode;

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

const CONTAMINANT_MOTIFS: &[&str] = &["GCGTAATACGACTCACTATAGGG", "CAGCGGCATCAGCACCTTG"]; // phix/spike-in like motifs

fn contaminant_like(record: &FastqRecord) -> bool {
    let seq = record.sequence.to_ascii_uppercase();
    CONTAMINANT_MOTIFS.iter().any(|motif| seq.contains(motif))
}

/// Deplete contaminant-reference reads from FASTQ streams.
///
/// # Errors
/// Returns an error when paired inputs or outputs are incoherent.
pub fn deplete_reference_contaminants(
    r1: &Path,
    r2: Option<&Path>,
    params: &ReferenceContaminantEffectiveParams,
    output_r1: &Path,
    output_r2: Option<&Path>,
    removed_reads_r1: &Path,
    removed_reads_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<DepleteReferenceContaminantsReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 { read_fastq_records(path)? } else { Vec::new() };

    let paired = r2.is_some();
    if paired && left.len() != right.len() {
        return Err(anyhow!(
            "fastq.deplete_reference_contaminants refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }
    if paired && (output_r2.is_none() || removed_reads_r2.is_none()) {
        return Err(anyhow!(
            "fastq.deplete_reference_contaminants requires output_r2 and removed_reads_r2 for paired input"
        ));
    }

    let reads_in = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_in = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let mut retained_left = Vec::<FastqRecord>::new();
    let mut retained_right = Vec::<FastqRecord>::new();
    let mut removed_left = Vec::<FastqRecord>::new();
    let mut removed_right = Vec::<FastqRecord>::new();

    if paired {
        for (l, r) in left.iter().cloned().zip(right.iter().cloned()) {
            if contaminant_like(&l) || contaminant_like(&r) {
                removed_left.push(l);
                removed_right.push(r);
            } else {
                retained_left.push(l);
                retained_right.push(r);
            }
        }
    } else {
        for l in left.iter().cloned() {
            if contaminant_like(&l) {
                removed_left.push(l);
            } else {
                retained_left.push(l);
            }
        }
    }

    write_fastq_records(output_r1, &retained_left)?;
    if paired {
        write_fastq_records(output_r2.expect("validated above"), &retained_right)?;
    }
    write_fastq_records(removed_reads_r1, &removed_left)?;
    if paired {
        write_fastq_records(removed_reads_r2.expect("validated above"), &removed_right)?;
    }

    let reads_out = if paired {
        (retained_left.len() + retained_right.len()) as u64
    } else {
        retained_left.len() as u64
    };
    let removed_read_count = reads_in.saturating_sub(reads_out);
    let bases_out = retained_left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + retained_right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    Ok(DepleteReferenceContaminantsReportV1 {
        schema_version: DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.deplete_reference_contaminants".to_string(),
        stage_id: "fastq.deplete_reference_contaminants".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        reference_catalog_id: params.reference_catalog_id.clone(),
        contaminant_reference: params.contaminant_reference.clone(),
        reference_index_artifact_id: params.index_artifact.clone(),
        reference_index_backend: params.reference_index_backend.clone(),
        reference_build_id: params.reference_build_id.clone(),
        reference_digest: params.reference_digest.clone(),
        match_threshold: Some(0.90),
        retained_read_role: "contaminant_screened_reads".to_string(),
        rejected_read_role: "removed_contaminant_reads".to_string(),
        retain_unmapped_pairs: params.retain_unmapped_pairs,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        removed_reads_r1: removed_reads_r1.display().to_string(),
        removed_reads_r2: removed_reads_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        reads_in,
        reads_out,
        reads_removed: removed_read_count,
        bases_in,
        bases_out,
        bases_removed: bases_in.saturating_sub(bases_out),
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(retained_left.len() as u64),
        contaminant_fraction_removed: if reads_in == 0 {
            0.0
        } else {
            removed_read_count as f64 / reads_in as f64
        },
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report.map(|_| "bowtie2_met_file".to_string()),
        backend_metrics: Some(serde_json::json!({
            "removed_contaminant_reads": removed_read_count,
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::deplete_reference_contaminants;
    use crate::params::screen::ReferenceContaminantEffectiveParams;
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
    fn deplete_reference_contaminants_removes_contaminant_like_reads() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-deplete-contaminants")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(
            &r1,
            &[
                ("cont", "GCGTAATACGACTCACTATAGGG", "IIIIIIIIIIIIIIIIIIIIIII"),
                ("clean", "ACGTACGTACGT", "IIIIIIIIIIII"),
            ],
        )?;

        let params = ReferenceContaminantEffectiveParams {
            schema_version: "bijux.fastq.params.deplete_reference_contaminants.v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            reference_catalog_id: "contaminants".to_string(),
            contaminant_reference: "phix".to_string(),
            index_artifact: "contaminant_index".to_string(),
            reference_index_backend: "bowtie2_build".to_string(),
            reference_build_id: None,
            reference_digest: None,
            retain_unmapped_pairs: false,
        };

        let report = deplete_reference_contaminants(
            &r1,
            None,
            &params,
            &temp.path().join("retained.fastq"),
            None,
            &temp.path().join("removed.fastq"),
            None,
            &temp.path().join("report.json"),
            None,
        )?;

        assert_eq!(report.reads_removed, 1);
        assert_eq!(
            report.removed_reads_r1,
            temp.path().join("removed.fastq").display().to_string()
        );
        assert_eq!(report.reads_out, 1);
        Ok(())
    }
}
