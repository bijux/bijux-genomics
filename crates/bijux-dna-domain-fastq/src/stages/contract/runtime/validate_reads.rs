use std::path::Path;

use anyhow::Result;

use crate::artifacts::{
    ValidateFailureClass, ValidatedReadsManifestV1, ValidationReportV1,
    VALIDATED_READS_MANIFEST_SCHEMA_VERSION, VALIDATION_REPORT_SCHEMA_VERSION,
};
use crate::params::validate::{PairSyncPolicy, ValidationMode};
use crate::params::PairedMode;

use super::fastq_io::{parse_header_pairing, read_fastq_records, FastqRecord};

fn invalid_quality_encoding(record: &FastqRecord) -> bool {
    record.quality.bytes().any(|byte| !(33..=74).contains(&byte))
}

/// Validate FASTQ inputs with explicit malformed-read and pair-sync failure taxonomy.
///
/// # Errors
/// Returns an error when validation logs/manifests cannot be written.
pub fn validate_reads(
    r1: &Path,
    r2: Option<&Path>,
    validation_mode: ValidationMode,
    pair_sync_policy: PairSyncPolicy,
    validation_log_r1: &Path,
    validation_log_r2: Option<&Path>,
    validation_report_path: &Path,
) -> Result<(ValidationReportV1, ValidatedReadsManifestV1)> {
    if let Some(parent) = validation_log_r1.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut failure_class = ValidateFailureClass::None;
    let mut status_r1 = 0;
    let mut status_r2 = 0;

    let left = match read_fastq_records(r1) {
        Ok(records) => records,
        Err(err) => {
            status_r1 = 1;
            let message = err.to_string();
            failure_class = if message.contains("truncated")
                || message.contains("invalid FASTQ")
                || message.contains("length mismatch")
            {
                ValidateFailureClass::MalformedRecord
            } else if message.contains("gz") || message.contains("gzip") {
                ValidateFailureClass::UnsupportedCompression
            } else {
                ValidateFailureClass::ValidatorError
            };
            std::fs::write(validation_log_r1, format!("validation_error: {message}\n"))?;
            Vec::new()
        }
    };

    if status_r1 == 0 {
        let invalid_quality = left.iter().any(invalid_quality_encoding);
        if invalid_quality {
            status_r1 = 1;
            failure_class = ValidateFailureClass::InvalidQualityEncoding;
            std::fs::write(validation_log_r1, "invalid_quality_encoding_detected\n")?;
        } else if left.is_empty() {
            status_r1 = 1;
            failure_class = ValidateFailureClass::EmptyInput;
            std::fs::write(validation_log_r1, "empty_input\n")?;
        } else {
            std::fs::write(
                validation_log_r1,
                format!("validated_reads: {}\nstatus: ok\n", left.len()),
            )?;
        }
    }

    let right = if let Some(r2_path) = r2 {
        let Some(log_r2) = validation_log_r2 else {
            return Err(anyhow::anyhow!(
                "fastq.validate_reads requires validation_log_r2 for paired input"
            ));
        };
        let records = match read_fastq_records(r2_path) {
            Ok(records) => records,
            Err(err) => {
                status_r2 = 1;
                if matches!(failure_class, ValidateFailureClass::None) {
                    let message = err.to_string();
                    failure_class = if message.contains("truncated")
                        || message.contains("invalid FASTQ")
                        || message.contains("length mismatch")
                    {
                        ValidateFailureClass::MalformedRecord
                    } else if message.contains("gz") || message.contains("gzip") {
                        ValidateFailureClass::UnsupportedCompression
                    } else {
                        ValidateFailureClass::ValidatorError
                    };
                }
                std::fs::write(log_r2, format!("validation_error: {}\n", err))?;
                Vec::new()
            }
        };
        if status_r2 == 0 {
            let invalid_quality = records.iter().any(invalid_quality_encoding);
            if invalid_quality {
                status_r2 = 1;
                if matches!(failure_class, ValidateFailureClass::None) {
                    failure_class = ValidateFailureClass::InvalidQualityEncoding;
                }
                std::fs::write(log_r2, "invalid_quality_encoding_detected\n")?;
            } else if records.is_empty() {
                status_r2 = 1;
                if matches!(failure_class, ValidateFailureClass::None) {
                    failure_class = ValidateFailureClass::EmptyInput;
                }
                std::fs::write(log_r2, "empty_input\n")?;
            } else {
                std::fs::write(
                    log_r2,
                    format!("validated_reads: {}\nstatus: ok\n", records.len()),
                )?;
            }
        }
        records
    } else {
        Vec::new()
    };

    let paired = r2.is_some();
    let pair_sync_checked = paired
        && !matches!(pair_sync_policy, PairSyncPolicy::NotApplicable | PairSyncPolicy::SkipHeaderSync);

    let mut pair_sync_pass = None;
    let mut pair_count_match = None;
    let mut validated_pairs = None;

    if paired {
        let count_match = left.len() == right.len();
        pair_count_match = Some(count_match);
        validated_pairs = Some(left.len().min(right.len()) as u64);
        if !count_match {
            status_r1 = 1;
            status_r2 = 1;
            if matches!(failure_class, ValidateFailureClass::None) {
                failure_class = ValidateFailureClass::PairCountMismatch;
            }
        }

        if pair_sync_checked && status_r1 == 0 && status_r2 == 0 {
            let mut sync_ok = true;
            for (l, r) in left.iter().zip(right.iter()) {
                let (left_base, left_mate) = parse_header_pairing(l.header.trim_start_matches('@'));
                let (right_base, right_mate) = parse_header_pairing(r.header.trim_start_matches('@'));
                if left_base != right_base
                    || matches!(left_mate, Some(2))
                    || matches!(right_mate, Some(1))
                {
                    sync_ok = false;
                    break;
                }
            }
            pair_sync_pass = Some(sync_ok);
            if !sync_ok {
                status_r1 = 1;
                status_r2 = 1;
                if matches!(failure_class, ValidateFailureClass::None) {
                    failure_class = ValidateFailureClass::HeaderSyncMismatch;
                }
            }
        }
    }

    let strict_pass = status_r1 == 0
        && status_r2 == 0
        && matches!(failure_class, ValidateFailureClass::None)
        && pair_sync_pass.unwrap_or(true)
        && pair_count_match.unwrap_or(true);

    let exit_code = if strict_pass || matches!(validation_mode, ValidationMode::ReportOnly) {
        0
    } else {
        96
    };

    let report = ValidationReportV1 {
        schema_version: VALIDATION_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.validate_reads".to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        tool_id: "bijux".to_string(),
        validation_mode: validation_mode.clone(),
        pair_sync_policy: pair_sync_policy.clone(),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        validation_log_r1: validation_log_r1.display().to_string(),
        validation_log_r2: validation_log_r2.map(|path| path.display().to_string()),
        validated_inputs: if paired { 2 } else { 1 },
        validated_reads_r1: left.len() as u64,
        validated_reads_r2: paired.then_some(right.len() as u64),
        validated_pairs,
        status_r1,
        status_r2,
        pair_sync_checked,
        pair_sync_pass,
        pair_count_match,
        failure_class,
        strict_pass,
        exit_code,
    };

    let manifest = ValidatedReadsManifestV1 {
        schema_version: VALIDATED_READS_MANIFEST_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        tool_id: "bijux".to_string(),
        validation_mode,
        pair_sync_policy,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        validation_report: validation_report_path.display().to_string(),
        paired_mode: if paired {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        validated_stream_ids: if paired {
            vec!["reads_r1".to_string(), "reads_r2".to_string()]
        } else {
            vec!["reads_r1".to_string()]
        },
        pair_sync_checked,
        pair_sync_pass,
        validated_pairs,
    };

    Ok((report, manifest))
}

#[cfg(test)]
mod tests {
    use super::validate_reads;
    use crate::artifacts::ValidateFailureClass;
    use crate::params::validate::{PairSyncPolicy, ValidationMode};

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn validate_reads_reports_pair_mismatch_failure() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-validate-reads")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        write_fastq(&r1, &[("A/1", "AAAA", "!!!!"), ("B/1", "CCCC", "####")])?;
        write_fastq(&r2, &[("A/2", "TTTT", "!!!!")])?;

        let (report, _manifest) = validate_reads(
            &r1,
            Some(&r2),
            ValidationMode::Strict,
            PairSyncPolicy::RequireHeaderSync,
            &temp.path().join("r1.log"),
            Some(&temp.path().join("r2.log")),
            &temp.path().join("validation.json"),
        )?;

        assert_eq!(report.failure_class, ValidateFailureClass::PairCountMismatch);
        assert!(!report.strict_pass);
        Ok(())
    }

    #[test]
    fn validate_reads_reports_invalid_quality_encoding() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-validate-reads")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(&r1, &[("A", "AAAA", "~~~~")])?;

        let (report, _manifest) = validate_reads(
            &r1,
            None,
            ValidationMode::Strict,
            PairSyncPolicy::NotApplicable,
            &temp.path().join("r1.log"),
            None,
            &temp.path().join("validation.json"),
        )?;

        assert_eq!(report.failure_class, ValidateFailureClass::InvalidQualityEncoding);
        Ok(())
    }
}
