use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::{fs, path::Path};

/// Canonical BAM proof schema selected by the governed probe catalog.
#[derive(Debug, Clone, Copy)]
pub enum BamReportKind {
    ValidationSummary,
    AlignmentProvenance,
    ContaminationJson,
    OverlapCorrectionSummary,
    RecalibrationSummary,
    DamageEvidence,
    AuthenticityAdvisory,
    ComplexitySummary,
    KinshipSummary,
    SexJson,
    GenotypingJson,
}

/// Admit the canonical BAM report representation.
///
/// # Errors
/// Rejects unreadable or malformed report bytes.
pub fn parse_report(proof_path: &Path, kind: BamReportKind) -> Result<Value> {
    match kind {
        BamReportKind::ValidationSummary => {
            let _: bijux_dna_domain_bam::BamValidationSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::AlignmentProvenance => {
            let _: bijux_dna_domain_bam::BamAlignmentProvenanceV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::ContaminationJson => {
            let _ = bijux_dna_domain_bam::metrics::parse_contamination_json(proof_path)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::OverlapCorrectionSummary => {
            let _: bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::RecalibrationSummary => {
            let _: bijux_dna_domain_bam::BamRecalibrationSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::DamageEvidence => {
            let _: bijux_dna_domain_bam::BamDamageEvidenceV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::AuthenticityAdvisory => {
            let _: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::ComplexitySummary => {
            let _: bijux_dna_domain_bam::BamComplexitySummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::KinshipSummary => {
            let _: bijux_dna_domain_bam::BamKinshipSummaryV1 = serde_json::from_str(
                &fs::read_to_string(proof_path)
                    .with_context(|| format!("read {}", proof_path.display()))?,
            )
            .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::SexJson => {
            let _ = bijux_dna_domain_bam::metrics::parse_sex_json(proof_path)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::GenotypingJson => {
            let value = read_json_document(proof_path)?;
            if value.get("producer").is_none() {
                return Err(anyhow!("{} is missing `producer`", proof_path.display()));
            }
        }
    }
    read_json_document(proof_path)
}

fn read_json_document(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
