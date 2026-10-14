use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::metrics::downstream::{
    contamination::parse_contamination_json_contents, sex::parse_sex_json_contents,
};
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
/// Rejects unreadable or malformed report bytes. Validation and the returned JSON use
/// the same acquisition; this does not make concurrent in-place writes atomic.
pub fn parse_report(proof_path: &Path, kind: BamReportKind) -> Result<Value> {
    let acquired = fs::read_to_string(proof_path);
    let raw = match kind {
        BamReportKind::ContaminationJson => acquired
            .context("read contamination json")
            .with_context(|| format!("parse {}", proof_path.display()))?,
        BamReportKind::SexJson => acquired
            .context("read sex json")
            .with_context(|| format!("parse {}", proof_path.display()))?,
        _ => acquired.with_context(|| format!("read {}", proof_path.display()))?,
    };
    parse_report_contents(proof_path, &raw, kind)
}

fn parse_report_contents(proof_path: &Path, raw: &str, kind: BamReportKind) -> Result<Value> {
    match kind {
        BamReportKind::ValidationSummary => {
            let _: bijux_dna_domain_bam::BamValidationSummaryV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::AlignmentProvenance => {
            let _: bijux_dna_domain_bam::BamAlignmentProvenanceV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::ContaminationJson => {
            let _ = parse_contamination_json_contents(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::OverlapCorrectionSummary => {
            let _: bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::RecalibrationSummary => {
            let _: bijux_dna_domain_bam::BamRecalibrationSummaryV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::DamageEvidence => {
            let _: bijux_dna_domain_bam::BamDamageEvidenceV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::AuthenticityAdvisory => {
            let _: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::ComplexitySummary => {
            let _: bijux_dna_domain_bam::BamComplexitySummaryV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::KinshipSummary => {
            let _: bijux_dna_domain_bam::BamKinshipSummaryV1 = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::SexJson => {
            let _ = parse_sex_json_contents(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
        }
        BamReportKind::GenotypingJson => {
            let value: Value = serde_json::from_str(raw)
                .with_context(|| format!("parse {}", proof_path.display()))?;
            if value.get("producer").is_none() {
                return Err(anyhow!("{} is missing `producer`", proof_path.display()));
            }
            return Ok(value);
        }
    }
    serde_json::from_str(raw).with_context(|| format!("parse {}", proof_path.display()))
}

#[cfg(test)]
#[path = "reports/tests.rs"]
mod tests;
