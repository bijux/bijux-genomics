use super::ParseKind;
use anyhow::Result;
use bijux_dna_bench_bam_proofs::parser_probes::reports::{parse_report, BamReportKind};
use serde_json::Value;
use std::path::Path;

pub(super) fn parse(path: &Path, kind: ParseKind) -> Result<Value> {
    let native = match kind {
        ParseKind::BamValidationSummary => BamReportKind::ValidationSummary,
        ParseKind::BamAlignmentProvenance => BamReportKind::AlignmentProvenance,
        ParseKind::BamContaminationJson => BamReportKind::ContaminationJson,
        ParseKind::BamOverlapCorrectionSummary => BamReportKind::OverlapCorrectionSummary,
        ParseKind::BamRecalibrationSummary => BamReportKind::RecalibrationSummary,
        ParseKind::BamDamageEvidence => BamReportKind::DamageEvidence,
        ParseKind::BamAuthenticityAdvisory => BamReportKind::AuthenticityAdvisory,
        ParseKind::BamComplexitySummary => BamReportKind::ComplexitySummary,
        ParseKind::BamKinshipSummary => BamReportKind::KinshipSummary,
        ParseKind::BamSexJson => BamReportKind::SexJson,
        ParseKind::BamGenotypingJson => BamReportKind::GenotypingJson,
        _ => return Err(anyhow::anyhow!("not a BAM parser kind")),
    };
    parse_report(path, native)
}
