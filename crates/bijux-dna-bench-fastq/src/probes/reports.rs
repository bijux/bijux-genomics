use anyhow::{Context, Result};
use serde_json::Value;
use std::{fs, path::Path};
/// Canonical FASTQ report parser selected by stage contract.
#[derive(Debug, Clone, Copy)]
pub enum FastqReportKind {
    IndexReference,
    CorrectErrors,
    DepleteHost,
    DepleteRrna,
    TrimReads,
    MergePairs,
    FilterReads,
    ProfileReads,
    ReportQc,
    RemoveDuplicates,
    ExtractUmis,
    ScreenTaxonomy,
    InferAsvs,
    NormalizeAbundance,
    DetectDuplicatesPremerge,
}
/// Admit report bytes with the canonical domain parser and return that same observation.
///
/// # Errors
/// Rejects unreadable reports, invalid report contracts or malformed JSON.
pub fn parse_report(path: &Path, kind: FastqReportKind) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    match kind {
        FastqReportKind::IndexReference => {
            bijux_dna_domain_fastq::observer::parse_index_reference_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::CorrectErrors => {
            bijux_dna_domain_fastq::observer::parse_correct_errors_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::DepleteHost => {
            bijux_dna_domain_fastq::observer::parse_deplete_host_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::DepleteRrna => {
            bijux_dna_domain_fastq::observer::parse_deplete_rrna_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::TrimReads => {
            bijux_dna_domain_fastq::observer::parse_trim_reads_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::MergePairs => {
            bijux_dna_domain_fastq::observer::parse_merge_pairs_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::FilterReads => {
            bijux_dna_domain_fastq::observer::parse_filter_reads_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::ProfileReads => {
            bijux_dna_domain_fastq::observer::parse_profile_reads_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::ReportQc => {
            bijux_dna_domain_fastq::observer::parse_report_qc_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::RemoveDuplicates => {
            bijux_dna_domain_fastq::observer::parse_remove_duplicates_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::ExtractUmis => {
            bijux_dna_domain_fastq::observer::parse_extract_umis_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::ScreenTaxonomy => {
            bijux_dna_domain_fastq::observer::parse_screen_taxonomy_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::InferAsvs => {
            bijux_dna_domain_fastq::observer::parse_infer_asvs_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::NormalizeAbundance => {
            bijux_dna_domain_fastq::observer::parse_normalize_abundance_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
        FastqReportKind::DetectDuplicatesPremerge => {
            bijux_dna_domain_fastq::observer::parse_detect_duplicates_premerge_report(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
        }
    }
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
