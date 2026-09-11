use super::ParseKind;
use anyhow::Result;
use bijux_dna_bench_fastq::probes::reports::{parse_report, FastqReportKind};
use serde_json::Value;
use std::path::Path;

pub(super) fn parse(path: &Path, kind: ParseKind) -> Result<Value> {
    let native = match kind {
        ParseKind::FastqIndexReference => FastqReportKind::IndexReference,
        ParseKind::FastqCorrectErrors => FastqReportKind::CorrectErrors,
        ParseKind::FastqDepleteHost => FastqReportKind::DepleteHost,
        ParseKind::FastqDepleteRrna => FastqReportKind::DepleteRrna,
        ParseKind::FastqTrimReads => FastqReportKind::TrimReads,
        ParseKind::FastqMergePairs => FastqReportKind::MergePairs,
        ParseKind::FastqFilterReads => FastqReportKind::FilterReads,
        ParseKind::FastqProfileReads => FastqReportKind::ProfileReads,
        ParseKind::FastqReportQc => FastqReportKind::ReportQc,
        ParseKind::FastqRemoveDuplicates => FastqReportKind::RemoveDuplicates,
        ParseKind::FastqExtractUmis => FastqReportKind::ExtractUmis,
        ParseKind::FastqScreenTaxonomy => FastqReportKind::ScreenTaxonomy,
        ParseKind::FastqInferAsvs => FastqReportKind::InferAsvs,
        ParseKind::FastqNormalizeAbundance => FastqReportKind::NormalizeAbundance,
        ParseKind::FastqDetectDuplicatesPremerge => FastqReportKind::DetectDuplicatesPremerge,
        _ => return Err(anyhow::anyhow!("not a FASTQ parser kind")),
    };
    parse_report(path, native)
}
