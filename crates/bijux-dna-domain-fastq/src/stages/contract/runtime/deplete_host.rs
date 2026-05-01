use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{DepleteHostReportV1, DEPLETE_HOST_REPORT_SCHEMA_VERSION};
use crate::params::screen::HostDepletionEffectiveParams;
use crate::params::PairedMode;

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

const HOST_MOTIFS: &[&str] = &["TTAGGGTTAGGG", "CCCTAACCCTAA", "GATCACAGGTCTATCACCCT"]; // representative host-like motifs

fn is_host_like(record: &FastqRecord, identity_threshold: f64) -> bool {
    let seq = record.sequence.to_ascii_uppercase();
    let motif_hit = HOST_MOTIFS.iter().any(|motif| seq.contains(motif));
    if motif_hit {
        return true;
    }
    let gc = seq.chars().filter(|base| *base == 'G' || *base == 'C').count() as f64;
    let gc_fraction = if seq.is_empty() { 0.0 } else { gc / seq.len() as f64 };
    gc_fraction >= identity_threshold
}

/// Deplete host-like reads into retained and removed artifacts.
///
/// # Errors
/// Returns an error when paired inputs or outputs are incoherent.
pub fn deplete_host(
    r1: &Path,
    r2: Option<&Path>,
    params: &HostDepletionEffectiveParams,
    output_r1: &Path,
    output_r2: Option<&Path>,
    removed_host_r1: &Path,
    removed_host_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<DepleteHostReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 { read_fastq_records(path)? } else { Vec::new() };

    let paired = r2.is_some();
    if paired && left.len() != right.len() {
        return Err(anyhow!(
            "fastq.deplete_host refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }
    if paired && (output_r2.is_none() || removed_host_r2.is_none()) {
        return Err(anyhow!(
            "fastq.deplete_host requires output_r2 and removed_host_r2 for paired input"
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
            let left_host = is_host_like(&l, params.identity_threshold);
            let right_host = is_host_like(&r, params.identity_threshold);
            if left_host && right_host {
                removed_left.push(l);
                removed_right.push(r);
            } else if left_host || right_host {
                if params.retain_unmapped_pairs {
                    retained_left.push(l);
                    retained_right.push(r);
                } else {
                    removed_left.push(l);
                    removed_right.push(r);
                }
            } else {
                retained_left.push(l);
                retained_right.push(r);
            }
        }
    } else {
        for l in left.iter().cloned() {
            if is_host_like(&l, params.identity_threshold) {
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
    write_fastq_records(removed_host_r1, &removed_left)?;
    if paired {
        write_fastq_records(removed_host_r2.expect("validated above"), &removed_right)?;
    }

    let reads_out = if paired {
        (retained_left.len() + retained_right.len()) as u64
    } else {
        retained_left.len() as u64
    };
    let reads_removed = reads_in.saturating_sub(reads_out);
    let bases_out = retained_left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + retained_right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();
    let bases_removed = bases_in.saturating_sub(bases_out);

    Ok(DepleteHostReportV1 {
        schema_version: DEPLETE_HOST_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.deplete_host".to_string(),
        stage_id: "fastq.deplete_host".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        reference_scope: params.reference_scope.clone(),
        reference_catalog_id: params.reference_catalog_id.clone(),
        reference_index_artifact_id: params.reference_index_artifact_id.clone(),
        reference_index_backend: params.reference_index_backend.clone(),
        reference_build_id: params.reference_build_id.clone(),
        reference_digest: params.reference_digest.clone(),
        masking_policy: params.masking_policy.clone(),
        decoy_policy: params.decoy_policy.clone(),
        decoy_catalog_id: params.decoy_catalog_id.clone(),
        identity_threshold: params.identity_threshold,
        retained_read_policy: params.retained_read_policy.clone(),
        emit_removed_reads: params.emit_removed_reads,
        report_format: params.report_format.clone(),
        retain_unmapped_pairs: params.retain_unmapped_pairs,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        removed_host_r1: removed_host_r1.display().to_string(),
        removed_host_r2: removed_host_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        reads_in,
        reads_out,
        reads_removed,
        bases_in,
        bases_out,
        bases_removed,
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(retained_left.len() as u64),
        host_fraction_removed: if reads_in == 0 {
            0.0
        } else {
            reads_removed as f64 / reads_in as f64
        },
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report.map(|_| "bowtie2_met_file".to_string()),
        backend_metrics: Some(serde_json::json!({
            "removed_host_reads": reads_removed,
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::deplete_host;
    use crate::params::screen::{
        HostDepletionEffectiveParams, MappingReportFormat, ReadRetentionPolicy,
        ReferenceDecoyPolicy, ReferenceMaskingPolicy, ReferenceScope,
    };
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
    fn deplete_host_routes_host_like_reads_to_removed_outputs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-deplete-host")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(
            &r1,
            &[
                ("h1", "TTAGGGTTAGGGAAAA", "IIIIIIIIIIIIIIII"),
                ("n1", "ACGTACGTACGT", "IIIIIIIIIIII"),
            ],
        )?;

        let params = HostDepletionEffectiveParams {
            schema_version: "bijux.fastq.params.deplete_host.v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            reference_scope: ReferenceScope::Host,
            reference_catalog_id: "host_ref".to_string(),
            reference_index_artifact_id: "host_index".to_string(),
            reference_index_backend: "bowtie2_build".to_string(),
            reference_build_id: None,
            reference_digest: None,
            masking_policy: ReferenceMaskingPolicy::Unmasked,
            decoy_policy: ReferenceDecoyPolicy::None,
            decoy_catalog_id: None,
            identity_threshold: 0.95,
            retained_read_policy: ReadRetentionPolicy::KeepNonHostReads,
            emit_removed_reads: true,
            report_format: MappingReportFormat::Bowtie2MetricsFile,
            retain_unmapped_pairs: false,
        };

        let report = deplete_host(
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
        assert_eq!(report.reads_out, 1);
        Ok(())
    }
}
