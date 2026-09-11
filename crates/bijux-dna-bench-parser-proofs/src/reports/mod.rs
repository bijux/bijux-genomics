#[cfg(feature = "bam")]
mod bam;
#[cfg(feature = "fastq")]
mod fastq;
mod kind;
mod surface;
mod vcf;
use crate::values::json_snapshot;
use anyhow::Result;
pub use kind::ParseKind;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};
pub use surface::parse_surface_label;

/// Admit a proof through its declared parser and project the governed snapshot.
///
/// # Errors
/// Rejects unsupported feature families, malformed reports or parser failures.
pub fn parse_proof(
    proof_path: &Path,
    parse_kind: ParseKind,
    snapshot_keys: &[&str],
) -> Result<(Value, BTreeMap<String, Value>)> {
    let value = match parse_kind {
        ParseKind::FastqIndexReference
        | ParseKind::FastqCorrectErrors
        | ParseKind::FastqDepleteHost
        | ParseKind::FastqDepleteRrna
        | ParseKind::FastqTrimReads
        | ParseKind::FastqMergePairs
        | ParseKind::FastqFilterReads
        | ParseKind::FastqProfileReads
        | ParseKind::FastqReportQc
        | ParseKind::FastqRemoveDuplicates
        | ParseKind::FastqExtractUmis
        | ParseKind::FastqScreenTaxonomy
        | ParseKind::FastqInferAsvs
        | ParseKind::FastqNormalizeAbundance
        | ParseKind::FastqDetectDuplicatesPremerge => {
            #[cfg(feature = "fastq")]
            {
                fastq::parse(proof_path, parse_kind)?
            }
            #[cfg(not(feature = "fastq"))]
            {
                return Err(anyhow::anyhow!("FASTQ proof parser feature is unavailable"));
            }
        }
        ParseKind::BamValidationSummary
        | ParseKind::BamAlignmentProvenance
        | ParseKind::BamContaminationJson
        | ParseKind::BamOverlapCorrectionSummary
        | ParseKind::BamRecalibrationSummary
        | ParseKind::BamDamageEvidence
        | ParseKind::BamAuthenticityAdvisory
        | ParseKind::BamComplexitySummary
        | ParseKind::BamKinshipSummary
        | ParseKind::BamSexJson
        | ParseKind::BamGenotypingJson => {
            #[cfg(feature = "bam")]
            {
                bam::parse(proof_path, parse_kind)?
            }
            #[cfg(not(feature = "bam"))]
            {
                return Err(anyhow::anyhow!("BAM proof parser feature is unavailable"));
            }
        }
        ParseKind::VcfCallMetrics
        | ParseKind::VcfDemographyReport
        | ParseKind::VcfIbdReport
        | ParseKind::VcfImputeMetrics
        | ParseKind::VcfPhasingMetrics
        | ParseKind::VcfPopulationStructureReport
        | ParseKind::VcfQcMetrics => vcf::parse(proof_path, parse_kind)?,
    };
    let snapshot = json_snapshot(&value, snapshot_keys);
    Ok((value, snapshot))
}

/// Verify parser availability before invoking a materializer.
///
/// # Errors
/// Rejects families whose native parser feature is not compiled.
pub fn require_available(kind: ParseKind) -> Result<()> {
    match kind {
        ParseKind::FastqIndexReference
        | ParseKind::FastqCorrectErrors
        | ParseKind::FastqDepleteHost
        | ParseKind::FastqDepleteRrna
        | ParseKind::FastqTrimReads
        | ParseKind::FastqMergePairs
        | ParseKind::FastqFilterReads
        | ParseKind::FastqProfileReads
        | ParseKind::FastqReportQc
        | ParseKind::FastqRemoveDuplicates
        | ParseKind::FastqExtractUmis
        | ParseKind::FastqScreenTaxonomy
        | ParseKind::FastqInferAsvs
        | ParseKind::FastqNormalizeAbundance
        | ParseKind::FastqDetectDuplicatesPremerge
            if !cfg!(feature = "fastq") =>
        {
            Err(anyhow::anyhow!("FASTQ proof parser feature is unavailable"))
        }
        ParseKind::BamValidationSummary
        | ParseKind::BamAlignmentProvenance
        | ParseKind::BamContaminationJson
        | ParseKind::BamOverlapCorrectionSummary
        | ParseKind::BamRecalibrationSummary
        | ParseKind::BamDamageEvidence
        | ParseKind::BamAuthenticityAdvisory
        | ParseKind::BamComplexitySummary
        | ParseKind::BamKinshipSummary
        | ParseKind::BamSexJson
        | ParseKind::BamGenotypingJson
            if !cfg!(feature = "bam") =>
        {
            Err(anyhow::anyhow!("BAM proof parser feature is unavailable"))
        }
        _ => Ok(()),
    }
}
