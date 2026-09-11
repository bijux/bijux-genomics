use super::declarations::EXPECTED_SMOKE_CASE_COUNT;
use super::declarations::FASTQ_VALIDATE_READS_SMOKE_PATH;
use super::declarations::FASTQ_VALIDATE_READS_STAGE_ID;
use super::declarations::REQUIRED_SMOKE_LAYOUTS;
use super::declarations::REQUIRED_SMOKE_SAMPLE_IDS;
use super::models::FastqValidateReadsSmokeEvidence;
use super::models::LocalValidateReadsSmokeReport;
use crate::proofs::inputs::ValidationContext;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_fastq::observer::parse_validated_reads_manifest;
use bijux_dna_domain_fastq::observer::parse_validation_report;
use bijux_dna_domain_fastq::params::PairedMode;
use std::fs;
pub(super) fn load_fastq_validate_reads_smoke_evidence(
    context: &ValidationContext<'_>,
) -> Result<FastqValidateReadsSmokeEvidence> {
    let repo_root = context.repo_root;
    let smoke_path = repo_root.join(FASTQ_VALIDATE_READS_SMOKE_PATH);
    if !smoke_path.is_file() {
        context
            .validation
            .materialize(repo_root, FASTQ_VALIDATE_READS_STAGE_ID)
            .context("materialize fastq.validate_reads local smoke report")?;
    }
    let payload = fs::read_to_string(&smoke_path)
        .with_context(|| format!("read {}", smoke_path.display()))?;
    let report: LocalValidateReadsSmokeReport =
        serde_json::from_str(&payload).context("parse fastq.validate_reads smoke report")?;

    let mut sample_ids = Vec::new();
    let mut layouts = Vec::new();
    let mut read_count_totals = Vec::new();
    let mut validation_statuses = Vec::new();
    let mut failure_classes = Vec::new();
    let mut parseable = true;
    let mut sample_id_normalized = true;
    let mut layout_normalized = true;
    let mut read_count_normalized = true;
    let mut validation_status_normalized = true;
    let mut failure_reason_normalized = true;

    for case in &report.cases {
        sample_id_normalized &= !case.sample_id.trim().is_empty();
        sample_ids.push(case.sample_id.clone());
        layouts.push(case.layout.clone());
        read_count_totals.push(case.input_read_count_total);
        validation_statuses.push(case.validation_status.clone());

        let validation_report_path = repo_root.join(&case.validation_report);
        let validation_report_json = fs::read_to_string(&validation_report_path)
            .with_context(|| format!("read {}", validation_report_path.display()))?;
        let validation_report = parse_validation_report(&validation_report_json)
            .context("parse smoke validation report")?;

        let validated_reads_manifest_path = repo_root.join(&case.validated_reads_manifest);
        let validated_reads_manifest_json = fs::read_to_string(&validated_reads_manifest_path)
            .with_context(|| format!("read {}", validated_reads_manifest_path.display()))?;
        let validated_reads_manifest =
            parse_validated_reads_manifest(&validated_reads_manifest_json)
                .context("parse smoke validated-reads manifest")?;

        let derived_layout = paired_mode_label(validated_reads_manifest.paired_mode).to_string();
        layout_normalized &= derived_layout == case.layout;

        let derived_total = validation_report.validated_reads_r1
            + validation_report.validated_reads_r2.unwrap_or(0);
        read_count_normalized &= derived_total == case.input_read_count_total;
        if case.layout == "paired_end" {
            read_count_normalized &= validation_report.validated_pairs == case.input_pair_count;
        }

        let derived_status = if validation_report.strict_pass { "pass" } else { "fail" };
        validation_status_normalized &= derived_status == case.validation_status;
        failure_reason_normalized &= true;
        failure_classes
            .push(validate_failure_class_label(&validation_report.failure_class).to_string());

        parseable &= validation_report.stage_id == FASTQ_VALIDATE_READS_STAGE_ID
            && validated_reads_manifest.stage_id == FASTQ_VALIDATE_READS_STAGE_ID
            && validation_report.input_r2.is_some() == (case.layout == "paired_end")
            && validated_reads_manifest.input_r2.is_some() == (case.layout == "paired_end");
    }

    sample_ids.sort();
    sample_ids.dedup();
    layouts.sort();
    layouts.dedup();
    read_count_totals.sort_unstable();
    validation_statuses.sort();
    validation_statuses.dedup();
    failure_classes.sort();
    failure_classes.dedup();

    sample_id_normalized &= sample_ids == REQUIRED_SMOKE_SAMPLE_IDS.map(str::to_string);
    layout_normalized &= layouts == REQUIRED_SMOKE_LAYOUTS.map(str::to_string);
    validation_status_normalized &= validation_statuses == vec!["pass".to_string()];
    parseable &= report.cases.len() == EXPECTED_SMOKE_CASE_COUNT;

    Ok(FastqValidateReadsSmokeEvidence {
        proof_path: FASTQ_VALIDATE_READS_SMOKE_PATH.to_string(),
        sample_case_count: report.cases.len(),
        sample_ids,
        layouts,
        read_count_totals,
        validation_statuses,
        failure_classes,
        parseable,
        sample_id_normalized,
        layout_normalized,
        read_count_normalized,
        validation_status_normalized,
        failure_reason_normalized,
    })
}

pub(super) fn paired_mode_label(paired_mode: PairedMode) -> &'static str {
    match paired_mode {
        PairedMode::SingleEnd => "single_end",
        PairedMode::PairedEnd => "paired_end",
        PairedMode::Unknown => "unknown",
    }
}

pub(super) fn validate_failure_class_label(
    value: &bijux_dna_domain_fastq::ValidateFailureClass,
) -> &'static str {
    match value {
        bijux_dna_domain_fastq::ValidateFailureClass::None => "none",
        bijux_dna_domain_fastq::ValidateFailureClass::UnsupportedCompression => {
            "unsupported_compression"
        }
        bijux_dna_domain_fastq::ValidateFailureClass::EmptyInput => "empty_input",
        bijux_dna_domain_fastq::ValidateFailureClass::MalformedRecord => "malformed_record",
        bijux_dna_domain_fastq::ValidateFailureClass::InvalidQualityEncoding => {
            "invalid_quality_encoding"
        }
        bijux_dna_domain_fastq::ValidateFailureClass::ValidatorError => "validator_error",
        bijux_dna_domain_fastq::ValidateFailureClass::PairCountMismatch => "pair_count_mismatch",
        bijux_dna_domain_fastq::ValidateFailureClass::HeaderSyncMismatch => "header_sync_mismatch",
    }
}
